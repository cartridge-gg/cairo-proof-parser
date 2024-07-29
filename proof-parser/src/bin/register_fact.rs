use anyhow::Context;

use cairo_proof_parser::StarkProof;
use clap::Parser;
use itertools::Itertools;
use serde_felt::to_felts;
use starknet::accounts::{Account, Call, ExecutionEncoding, SingleOwnerAccount};
use starknet::core::types::{
    BlockId, BlockTag, Felt, TransactionExecutionStatus, TransactionStatus,
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
const FEE_MULTIPLIER: f64 = 2.0;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The StarkNet address of the signer.
    #[clap(short, long, value_parser)]
    address: String,

    /// The private key of the signer in hexadecimal.
    #[clap(short, long, value_parser)]
    key: String,

    #[clap(short, long, value_parser)]
    store_proof: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse(); // Automatically parse command line arguments

    let address = Felt::from_hex(&args.address).expect("Invalid signer address hex");
    let key =
        SigningKey::from_secret_scalar(Felt::from_hex(&args.key).expect("Invalid signer key hex"));

    // Setup StarkNet provider and wallet
    let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(RPC_URL).unwrap()));
    let signer = LocalWallet::from(key);
    let chain_id =
        Felt::from_hex("0x00000000000000000000000000000000000000000000534e5f5345504f4c4941")
            .unwrap();
    let mut account =
        SingleOwnerAccount::new(provider, signer, address, chain_id, ExecutionEncoding::New);
    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    // Read input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let proof = StarkProof::try_from(&input[..]).context("Failed to parse proof")?;
    let program_hash = proof.extract_program().unwrap().program_hash;
    let output_hash = proof.extract_output().unwrap().program_output_hash;

    let serialized_proof = to_felts(&proof)?;

    let mut nonce = account.get_nonce().await?;
    let mut hashes = vec![];

    for fragment in serialized_proof.into_iter().chunks(2000).into_iter() {
        let mut fragment: Vec<FieldElement> = fragment.collect();
        let hash = poseidon_hash_many(&fragment);
        hashes.push(hash);

        fragment.insert(0, fragment.len().into());

        if args.store_proof {
            io::Write::write_all(
                &mut std::fs::File::create(format!("proof_{nonce}_{hash}.txt"))?,
                fragment
                    .iter()
                    .map(|x| format!("{x}"))
                    .collect::<Vec<_>>()
                    .join(" ")
                    .as_bytes(),
            )?;
        }

        let tx = publish_fragment(&account, nonce, fragment).await?;
        println!("Publish transaction: {tx:#x} .");

        nonce += 1u64.into();
    }

    let calldata = vec![hashes.len().into()]
        .into_iter()
        .chain(hashes.into_iter())
        .chain(vec![1u64.into()].into_iter())
        .collect::<Vec<FieldElement>>();

    let tx = verify_and_register_fact(account, calldata, nonce).await?;
    println!("Verify transaction: {:#x} .", tx);

    let expected_fact = poseidon_hash_many(&[program_hash, output_hash]);
    println!("Expected fact: {:#x}", expected_fact);

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
        .fee_estimate_multiplier(FEE_MULTIPLIER)
        .send()
        .await?;

    wait_for(account, tx.clone()).await?;

    Ok(tx.transaction_hash)
}

async fn verify_and_register_fact(
    account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    serialized_proof: Vec<Felt>,
    nonce: Felt,
) -> anyhow::Result<FieldElement> {
    println!("Sending transaction...");
    let tx = account
        .execute(vec![Call {
            to: Felt::from_hex_be(FACT_REGISTRY).expect("invalid world address"),
            selector: get_selector_from_name("verify_and_register_fact_from_fragments")
                .expect("invalid selector"),
            calldata: serialized_proof,
        }])
        .nonce(nonce)
        .fee_estimate_multiplier(FEE_MULTIPLIER)
        .send()
        .await?;

    wait_for(&account, tx).await
}

async fn wait_for(
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    tx: InvokeTransactionResult,
) -> anyhow::Result<FieldElement> {
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

    Ok(tx.transaction_hash)
}
