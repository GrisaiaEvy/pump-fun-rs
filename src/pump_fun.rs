use std::str::FromStr;
use {
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_client::{
        connection_cache::ConnectionCache,
        nonblocking::tpu_client::{TpuClient, TpuClientConfig},
        tpu_client::send_and_confirm_transactions_in_parallel::{send_and_confirm_transactions_in_parallel, SendAndConfirmConfig},
    },
    solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_instruction,
        transaction::Transaction,
    },
    spl_associated_token_account::{create_associated_token_account, get_associated_token_address},
    spl_token::state::Account as TokenAccount,
    std::{sync::Arc, time::Instant},
    tokio,
};

async fn buy(
    client: Arc<RpcClient>,
    tpu_client: Arc<TpuClient>,
    payer_keypair: &Keypair,
    coin_data: &serde_json::Value,
    sol_in: f64,
    slippage_decimal: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Begin buying...");

    let mint_str = coin_data["mint"].as_str().unwrap();
    let owner = payer_keypair.pubkey();
    let mint = Pubkey::from_str(mint_str)?;

    let token_account = get_associated_token_address(&owner, &mint);
    let token_account_instruction = create_associated_token_account(&payer_keypair, &owner, &mint);

    let virtual_sol_reserves = coin_data["virtual_sol_reserves"].as_u64().unwrap();
    let virtual_token_reserves = coin_data["virtual_token_reserves"].as_u64().unwrap();
    let sol_in_lamports = (sol_in * 1_000_000_000.0) as u64;
    let token_out = (sol_in_lamports * virtual_token_reserves) / virtual_sol_reserves;

    let sol_in_with_slippage = sol_in * (1.0 + slippage_decimal);
    let max_sol_cost = (sol_in_with_slippage * 1_000_000_000.0) as u64;

    let bonding_curve = Pubkey::from_str(coin_data["bonding_curve"].as_str().unwrap())?;
    let associated_bonding_curve = Pubkey::from_str(coin_data["associated_bonding_curve"].as_str().unwrap())?;

    let keys = vec![
        AccountMeta::new_readonly(Pubkey::from_str("GlobalAccountPubkey")?, false),
        AccountMeta::new(Pubkey::from_str("FeeRecipientPubkey")?, false),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new(bonding_curve, false),
        AccountMeta::new(associated_bonding_curve, false),
        AccountMeta::new(token_account, false),
        AccountMeta::new(owner, true),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
        AccountMeta::new_readonly(Pubkey::from_str("PumpFunAccountPubkey")?, false),
        AccountMeta::new_readonly(Pubkey::from_str("PumpFunProgramPubkey")?, false),
    ];

    let buy: u64 = 16927863322537952870;
    let mut data = vec![];
    data.extend_from_slice(&buy.to_le_bytes());
    data.extend_from_slice(&token_out.to_le_bytes());
    data.extend_from_slice(&max_sol_cost.to_le_bytes());

    let swap_instruction = Instruction {
        program_id: Pubkey::from_str("PumpFunProgramPubkey")?,
        accounts: keys,
        data,
    };

    let instructions = vec![token_account_instruction, swap_instruction];

    let latest_blockhash = client.get_latest_blockhash().await?;
    let message = Message::new(&instructions, Some(&payer_keypair.pubkey()));
    let transaction = Transaction::new_signed_with_payer(
        &message,
        Some(&payer_keypair.pubkey()),
        &[payer_keypair],
        latest_blockhash,
    );

    let (recent_blockhash, _) = client.get_latest_blockhash().await?;
    let tx_signature = tpu_client.send_transaction(&transaction).await?;

    println!("Transaction sent with signature: {:?}", tx_signature);

    // Confirm the transaction
    let confirm = client.confirm_transaction(&tx_signature).await?;
    if confirm {
        println!("Transaction confirmed!");
    } else {
        eprintln!("Transaction confirmation failed.");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com".to_string()));
    let websocket_url = "wss://api.mainnet-beta.solana.com";
    let tpu_client = Arc::new(
        TpuClient::new_with_connection_cache(
            client.clone(),
            websocket_url,
            TpuClientConfig::default(),
            ConnectionCache::default(),
        ).await?
    );
    let payer_keypair = Keypair::new();
    let coin_data = serde_json::json!({
        "mint": "TokenMintPubkey",
        "virtual_sol_reserves": 1000000,
        "virtual_token_reserves": 1000000,
        "bonding_curve": "BondingCurvePubkey",
        "associated_bonding_curve": "AssociatedBondingCurvePubkey",
    });

    buy(client, tpu_client, &payer_keypair, &coin_data, 0.003, 0.5).await?;
    Ok(())
}
