use cairo_proof_parser::{
    output::{extract_output, ExtractOutputResult},
    parse,
    program::{extract_program, ExtractProgramResult},
};
use clap::Parser;
use starknet::accounts::ConnectedAccount;
use starknet::accounts::{Account, Call, ExecutionEncoding, SingleOwnerAccount};
use starknet::core::types::{
    BlockId, BlockTag, FieldElement, TransactionExecutionStatus, TransactionStatus,
};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::providers::Provider;
use starknet::signers::{LocalWallet, SigningKey};
use starknet_crypto::poseidon_hash_many;
use std::io::{self, Read};
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

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
    let provider = JsonRpcClient::new(HttpTransport::new(
        Url::parse("https://free-rpc.nethermind.io/sepolia-juno/v0_7").unwrap(),
    ));
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

    // let ExtractProgramResult {
    //     program: _,
    //     program_hash,
    // } = extract_program(&input).unwrap();

    // let ExtractOutputResult {
    //     program_output: _,
    //     program_output_hash,
    // } = extract_output(&input).unwrap();

    // let expected_fact = poseidon_hash_many(&[program_hash, program_output_hash]);

    // let serialized_proof: Vec<FieldElement> = parse(&input)?.into();
    // let tx = verify_and_register_fact(account, serialized_proof).await?;
    // println!("tx: {tx}");
    // println!("expected_fact: {}", expected_fact.to_string());

    Ok(())
}

async fn verify_and_register_fact(
    account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    serialized_proof: Vec<FieldElement>,
) -> anyhow::Result<String> {
    let tx = account
        .execute(vec![Call {
            to: FieldElement::from_hex_be(
                "0x282969f3212819740d03929f52c709bc62f4c2ce8c17b5f24bd835a03cca22",
            )
            .expect("invalid world address"),
            selector: get_selector_from_name("verify_and_register_fact").expect("invalid selector"),
            calldata: serialized_proof,
        }])
        .max_fee(starknet::macros::felt!("1000000000000000")) // sometimes failing without this line
        .send()
        .await?;

    println!("tx hash: {:#x}", tx.transaction_hash);

    let start_fetching = std::time::Instant::now();
    let wait_for = Duration::from_secs(60);
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
