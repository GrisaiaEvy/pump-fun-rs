mod pump_fun;

use chrono::prelude::*;
use std::error::Error;
use std::fmt::Debug;
use std::io;
use std::str::FromStr;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use {
    solana_client::{
        connection_cache::ConnectionCache,
        nonblocking::tpu_client::TpuClient,
        tpu_client::TpuClientConfig,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        signature::Signer,
    },
    std::{sync::Arc},
    spl_associated_token_account::{
        get_associated_token_address,
        instruction::create_associated_token_account
    },
};
use serde::{Deserialize, Serialize};
use solana_sdk::message::Message;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;

#[derive(Debug, Deserialize, Serialize)]
struct PumpFunCoin {
    virtual_sol_reserves: u64,
    virtual_token_reserves: u64,
    total_supply: u64,
    market_cap: f64,
    bonding_curve: String,
    associated_bonding_curve: String,
}

const SOLANA_HTTP_URL: &str = "https://api.mainnet-beta.solana.com";

const SOLANA_WEBSOCKET_URL: &str = "https://api.mainnet-beta.solana.com";

const PUMP_FUN_COIN_URL: &str = "https://client-api-2-74b1891ee9f9.herokuapp.com/coins/";

const PUMP_FUN_ACCOUNT_ADDR: &str = "4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf";
const PUMP_FUN_TX_ADDR: &str = "Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1";
const PUMP_FUN_PROGRAM_ADDR: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

const RENT_ADDR: &str = "SysvarRent111111111111111111111111111111111";
const SYSTEM_PROGRAM_ADDR: &str = "11111111111111111111111111111111";
const TOKEN_PROGRAM_ADDR: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

const PUMP_BUY_CODE: u64 = 16927863322537952870;

const PUMP_SELL_CODE: u64 = 12502976635542562355;

// how much sol to buy the token
const BUY_SOL_COUNT: f64 = 0.001;

const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

const SLIPPAGE: f64 = 0.5;

const PRIVATE_KEY: &str = "";



#[tokio::main]
async fn main() {
    trade().await;
}

async fn trade() {
    let pump_fun_account_pubkey: Pubkey = Pubkey::from_str(PUMP_FUN_ACCOUNT_ADDR).unwrap();
    let pump_fun_tx_pubkey: Pubkey = Pubkey::from_str(PUMP_FUN_TX_ADDR).unwrap();
    let pump_fun_program_pubkey: Pubkey = Pubkey::from_str(PUMP_FUN_PROGRAM_ADDR).unwrap();

    let rent_pubkey: Pubkey = Pubkey::from_str(RENT_ADDR).unwrap();
    let system_program_pubkey: Pubkey = Pubkey::from_str(SYSTEM_PROGRAM_ADDR).unwrap();
    let token_program_pubkey: Pubkey = Pubkey::from_str(TOKEN_PROGRAM_ADDR).unwrap();
    // let client = rpc_client();
    let keypair = Keypair::from_base58_string(PRIVATE_KEY);
    let client = Arc::new(RpcClient::new(SOLANA_HTTP_URL.to_string()));

    let owner = keypair.pubkey();


    let connection_cache = ConnectionCache::new_quic("connection_cache_cli_program_quic", 1);

    if let ConnectionCache::Quic(cache) = connection_cache {
        let tpu_client_result= TpuClient::new_with_connection_cache(
            client.clone(),
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
                    println!("收到指令: {}", Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"));

                    // buy
                    let token_address = input.trim();

                    let mint = Pubkey::from_str(token_address).unwrap();

                    let spl_token_address = get_associated_token_address(&owner, &mint);

                    // default create token account
                    let token_account_instruction =
                        create_associated_token_account(&owner, &owner, &spl_token_address, &mint);

                    let target_coin: PumpFunCoin;

                    // 计算交易数额
                    let coin_result = coin_info(token_address).await;
                    match coin_result {
                        Ok(coin) => {
                            // let price_per_token: f64 = coin.market_cap * (10 ** 6) / coin.total_supply;
                            target_coin = coin;
                        }
                        Err(e) => {
                            eprintln!("Fetch coin info failed: {}", e.to_string());
                            continue;
                        }
                    }

                    let sol_lamports: u64 = (BUY_SOL_COUNT * LAMPORTS_PER_SOL as f64) as u64;
                    let buy_token_count: u64 = sol_lamports *
                        (target_coin.virtual_token_reserves / target_coin.virtual_sol_reserves);

                    let sol_slippage = BUY_SOL_COUNT * (1.0 + SLIPPAGE);
                    let max_sol_cost: u64 = (sol_slippage * LAMPORTS_PER_SOL as f64) as u64;

                    let buy: u64 = 16927863322537952870;
                    let mut data = vec![];
                    data.extend_from_slice(&buy.to_le_bytes());
                    data.extend_from_slice(&buy_token_count.to_le_bytes());
                    data.extend_from_slice(&max_sol_cost.to_le_bytes());

                    println!("构建账户: {}", Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"));
                    let accounts = vec![
                        // 用户自己的账户
                        AccountMeta::new(owner, true),
                        // 用户的代币账户
                        AccountMeta::new(spl_token_address, false),
                        // sol上管理spl代币
                        AccountMeta::new(Pubkey::from_str("CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM").unwrap(), false),
                        AccountMeta::new(Pubkey::from_str(target_coin.bonding_curve.as_str()).unwrap(), false),
                        AccountMeta::new(Pubkey::from_str(target_coin.associated_bonding_curve.as_str()).unwrap(), false),
                        // read-only
                        // mint token address
                        AccountMeta::new_readonly(mint, false),
                        // pump.fun related
                        AccountMeta::new_readonly(pump_fun_account_pubkey, false),
                        AccountMeta::new_readonly(pump_fun_tx_pubkey, false),
                        AccountMeta::new_readonly(pump_fun_program_pubkey, false),
                        AccountMeta::new_readonly(rent_pubkey, false),
                        AccountMeta::new_readonly(system_program_pubkey, false),
                        AccountMeta::new_readonly(token_program_pubkey, false),

                    ];

                    // 交易程序必须要填入pump fun的地址吗
                    let swap_instruction = Instruction {
                        program_id: Pubkey::from_str("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P").unwrap(),
                        accounts,
                        data,
                    };

                    let instructions = vec![token_account_instruction, swap_instruction];

                    let hash = client.clone().get_latest_blockhash().await.unwrap();
                    // let tx = Transaction::new_signed_with_payer(
                    //     &instructions,
                    //     Some(&keypair.pubkey()),
                    //     &[&keypair],
                    //     hash
                    // );

                    let message = Message::new(&instructions, Some(&keypair.pubkey()));

                    println!("发送交易: {}", Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"));
                    println!("{:?}", message);
                    let tx_result =
                        tpu_client.send_and_confirm_messages_with_spinner(
                            &[message],
                            &[&keypair]
                        ).await;
                    println!("交易发送完毕: {}", Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"));
                    match tx_result {
                        Ok(r) => {
                            println!("{:?}", r);
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            //⠖   0.0% | Waiting for next block, 1 transactions pending... [block height 248705304; re-sign in 23
                        }
                    }
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