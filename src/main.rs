mod pump_fun;

use std::cmp::min;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::io;
use std::str::FromStr;
use std::sync::OnceLock;
use serde_json::json;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use {
    solana_client::{
        connection_cache::ConnectionCache,
        nonblocking::tpu_client::TpuClient,
        send_and_confirm_transactions_in_parallel::{
            send_and_confirm_transactions_in_parallel, SendAndConfirmConfig,
        },
        tpu_client::TpuClientConfig,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig, message::Message, signature::Signer,
        system_instruction,
    },
    std::{sync::Arc, time::Instant},
    spl_associated_token_account::{
        get_associated_token_address,
        instruction::create_associated_token_account
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct PumpFunCoin {
    virtual_sol_reserves: usize,
    virtual_token_reserves: usize
}

const SOLANA_HTTP_URL: &str = "https://api.mainnet-beta.solana.com";

const SOLANA_WEBSOCKET_URL: &str = "https://api.mainnet-beta.solana.com";

const PUMP_FUN_COIN_URL: &str = "https://client-api-2-74b1891ee9f9.herokuapp.com/coins/";

// fn rpc_client() -> &'static RpcClient {
//     static RPC_CLIENT: OnceLock<RpcClient> = OnceLock::new();
//     RPC_CLIENT.get_or_init(|| RpcClient::new(SOLANA_HTTP_URL.to_string()))
// }

#[tokio::main]
async fn main() {
    trade().await;
}

async fn trade() {
    // let client = rpc_client();

    let client = RpcClient::new(SOLANA_HTTP_URL.to_string());

    let owner = Pubkey::from_str("").unwrap();
    let token_program_id = Pubkey::from_str("").unwrap();

    let connection_cache = ConnectionCache::new_quic("connection_cache_cli_program_quic", 1);

    if let ConnectionCache::Quic(cache) = connection_cache {
        let tpu_client_result= TpuClient::new_with_connection_cache(
            Arc::new(client),
            "wss://api.mainnet-beta.solana.com",
            TpuClientConfig::default(),
            cache,
        ).await;
        match tpu_client_result {
            Ok(tpu_client) => {
                println!("Tpu init success");
                loop {
                    println!("Input token address.");
                    let mut input = String::new();
                    if let Err(e) = io::stdin().read_line(&mut input) {
                        eprintln!("Failed to get input, please try again. Error: {}", e);
                    }

                    // buy
                    let token_address = input.trim();

                    let mint = Pubkey::from_str(token_address).unwrap();

                    let spl_token_address = get_associated_token_address(&owner, &mint);

                    // default create token account
                    let token_account_instruction =
                        create_associated_token_account(&owner, &owner, &spl_token_address, &token_program_id);

                    let token_out;
                    let max_sol_cost;

                    // 计算交易数额
                    let coin_result = coin_info(token_address).await;
                    match coin_result {
                        Ok(coin) => {

                        }
                        Err(e) => {
                            eprintln!("Fetch coin info failed: {}", e.to_string());
                            continue;
                        }
                    }

                    let buy: u64 = 16927863322537952870;
                    let mut data = vec![];
                    data.extend_from_slice(&buy.to_le_bytes());
                    data.extend_from_slice(&token_out.to_le_bytes());
                    data.extend_from_slice(&max_sol_cost.to_le_bytes());

                    let accounts = vec![

                    ];

                    // 交易程序必须要填入pump fun的地址吗
                    let swap_instruction = Instruction {
                        program_id: Pubkey::from_str("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P").unwrap(),
                        accounts,
                        data,
                    };






                    // let tx_signature = tpu_client.send_wire_transaction()

                }
            }
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    };
}

async fn coin_info(mint_addr: &str) -> Result<PumpFunCoin, Box<dyn Error>> {
    let resp = reqwest::get(PUMP_FUN_COIN_URL.to_owned() + mint_addr)
        .await?
        .json::<PumpFunCoin>()
        .await?;
    Ok(resp)
}

fn cal_trade_amount(virtual_sol_reserves: usize, virtual_token_reserves: usize) {

}

fn trade_tx_accounts(owner: Pubkey, token_address: Pubkey,
                     bonding_curve: &str, associated_bonding_curve: &str) -> Vec<AccountMeta> {
    vec![
        // 用户自己的账户
        AccountMeta::new(owner, true),
        // 用户的代币账户
        AccountMeta::new(token_address, false),
        // sol上管理spl代币
        AccountMeta::new(Pubkey::from_str("CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM").unwrap(), false),
        AccountMeta::new(Pubkey::from_str(bonding_curve).unwrap(), false),
        AccountMeta::new(Pubkey::from_str(associated_bonding_curve).unwrap(), false),
        // read-only
        // mint token address
        AccountMeta::new_readonly(token_address, false),

    ]
}