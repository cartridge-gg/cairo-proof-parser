use cairo_proof_parser::output::{extract_output, ExtractOutputResult};
use cairo_proof_parser::parse;
use cairo_proof_parser::program::{extract_program, ExtractProgramResult};
use clap::Parser;
use itertools::Itertools;
use serde_felt::to_felts;
use starknet::accounts::{Account, Call, ExecutionEncoding, SingleOwnerAccount};
use starknet::core::types::{
    BlockId, BlockTag, FieldElement, TransactionExecutionStatus, TransactionStatus,
};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::providers::Provider;
use starknet::signers::{LocalWallet, SigningKey};
use starknet::{accounts::ConnectedAccount, core::types::InvokeTransactionResult};
use starknet_crypto::poseidon_hash_many;
use std::io::{self, Read};
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

const FACT_REGISTRY: &str = "0x18c9ffecaf64edf3f10ee150272c870251436d1a9c0e2c8b14491da589a2b3f";
const RPC_URL: &str = "https://starknet-sepolia.g.alchemy.com/starknet/version/rpc/v0_7/PovJ0plog8O9RxyaPMYAZiKHqZ5LLII_";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The StarkNet address of the signer.
    #[clap(short, long, value_parser)]
    address: String,

    /// The private key of the signer in hexadecimal.
    #[clap(short, long, value_parser)]
    key: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse(); // Automatically parse command line arguments

    let address = FieldElement::from_hex_be(&args.address).expect("Invalid signer address hex");
    let key = SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(&args.key).expect("Invalid signer key hex"),
    );

    // Setup StarkNet provider and wallet
    let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(RPC_URL).unwrap()));
    let signer = LocalWallet::from(key);
    let chain_id = FieldElement::from_hex_be(
        "0x00000000000000000000000000000000000000000000534e5f5345504f4c4941",
    )
    .unwrap();
    let mut account =
        SingleOwnerAccount::new(provider, signer, address, chain_id, ExecutionEncoding::New);
    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    // Read input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    println!("Before extract_program");

    let ExtractProgramResult {
        program: _,
        program_hash,
    } = extract_program(&input).unwrap();

    println!("Before extract_output");

    let ExtractOutputResult {
        program_output: _,
        program_output_hash,
    } = extract_output(&input).unwrap();

    println!("Before verify_and_register_fact");

    let parsed = parse(&input)?;

    let serialized_proof = to_felts(&parsed)?;
    println!("serialized_proof: {}", serialized_proof.len());

    let mut nonce = account.get_nonce().await?;

    let mut hashes = vec![];

    for fragment in serialized_proof.into_iter().chunks(1500).into_iter() {
        let mut fragment: Vec<FieldElement> = fragment.collect();
        let hash = poseidon_hash_many(&fragment);
        hashes.push(hash);

        fragment.insert(0, fragment.len().into());

        // File::create(format!("proof_{nonce}_{hash}.txt"))?.write_all(
        //     fragment
        //         .iter()
        //         .map(|x| format!("{x}"))
        //         .collect::<Vec<_>>()
        //         .join(" ")
        //         .as_bytes(),
        // )?;

        let _ = publish_fragment(&account, nonce, fragment).await?;
        nonce += 1u64.into();
    }

    let calldata = vec![hashes.len().into()]
        .into_iter()
        .chain(hashes.into_iter())
        .chain(vec![1u64.into()].into_iter())
        .collect::<Vec<FieldElement>>();

    println!("Registering Fact");

    let tx = verify_and_register_fact(account, calldata, nonce).await?;
    println!("tx: {tx}");
    let expected_fact = poseidon_hash_many(&[program_hash, program_output_hash]);
    println!("expected_fact: {}", expected_fact);

    Ok(())
}

async fn publish_fragment(
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    nonce: FieldElement,
    serialized_proof: Vec<FieldElement>,
) -> anyhow::Result<FieldElement> {
    let tx = account
        .execute(vec![Call {
            to: FieldElement::from_hex_be(FACT_REGISTRY).expect("invalid world address"),
            selector: get_selector_from_name("publish_fragment").expect("invalid selector"),
            calldata: serialized_proof,
        }])
        .nonce(nonce)
        .max_fee(starknet::macros::felt!("20787888426336769")) // sometimes failing without this line
        .send()
        .await?;

    wait_for(account, tx).await?;

    Ok(0u64.into())
}

async fn verify_and_register_fact(
    account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    serialized_proof: Vec<FieldElement>,
    nonce: FieldElement,
) -> anyhow::Result<String> {
    println!("Sending transaction...");
    let tx = account
        .execute(vec![Call {
            to: FieldElement::from_hex_be(FACT_REGISTRY).expect("invalid world address"),
            selector: get_selector_from_name("verify_and_register_fact_from_fragments")
                .expect("invalid selector"),
            calldata: serialized_proof,
        }])
        .nonce(nonce)
        .max_fee(starknet::macros::felt!("20787888426336769")) // sometimes failing without this line
        .send()
        .await?;

    wait_for(&account, tx).await
}

async fn wait_for(
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    tx: InvokeTransactionResult,
) -> anyhow::Result<String> {
    println!("tx hash: {:#x}", tx.transaction_hash);

    let start_fetching = std::time::Instant::now();
    let wait_for = Duration::from_secs(360);
    let execution_status = loop {
        if start_fetching.elapsed() > wait_for {
            anyhow::bail!("Transaction not mined in {} seconds.", wait_for.as_secs());
        }

        let status = match account
            .provider()
            .get_transaction_status(tx.transaction_hash)
            .await
        {
            Ok(status) => status,
            Err(_e) => {
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        break match status {
            TransactionStatus::Received => {
                println!("Transaction received.");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            TransactionStatus::Rejected => {
                anyhow::bail!("Transaction {:#x} rejected.", tx.transaction_hash);
            }
            TransactionStatus::AcceptedOnL2(execution_status) => execution_status,
            TransactionStatus::AcceptedOnL1(execution_status) => execution_status,
        };
    };

    match execution_status {
        TransactionExecutionStatus::Succeeded => {
            println!("Transaction accepted on L2.");
        }
        TransactionExecutionStatus::Reverted => {
            anyhow::bail!("Transaction failed with.");
        }
    }

    Ok(format!("{:#x}", tx.transaction_hash))
}

#[test]
fn assert_the_same_poseidon() {
    let to_hash = vec![1u64.into()];
    let result = poseidon_hash_many(&to_hash);
    let expected = FieldElement::from_hex_be(
        "0x579e8877c7755365d5ec1ec7d3a94a457eff5d1f40482bbe9729c064cdead2",
    )
    .unwrap();

    assert_eq!(result, expected);
}
