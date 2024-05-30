mod pump_fun;

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
    std::{sync::Arc, time::Instant}
};

async fn send_transactions() {
    let client = RpcClient::new("f".to_string());
    let rpc_client = client;
    let from = Pubkey::default();
    let to = Pubkey::default();
    let signer = Pubkey::default();

    let blockhash = rpc_client.get_latest_blockhash().await?;

    // make your transaction messages here
    let messages = (0..100)
        .map(|i| {
            Message::new_with_blockhash(
                &[system_instruction::transfer(&from, &to, i)],
                Some(&signer.pubkey()),
                &blockhash,
            )
        })
        .collect::<Vec<_>>();

    let now = Instant::now();
    let connection_cache = ConnectionCache::new_quic("connection_cache_cli_program_quic", 1);
    let rpc_client = Arc::new(rpc_client);
    let transaction_errors = if let ConnectionCache::Quic(cache) = connection_cache {
        let tpu_client = TpuClient::new_with_connection_cache(
            rpc_client.clone(),
            "fadsf",
            TpuClientConfig::default(),
            cache,
        )
            .await?;
        // send_and_confirm_transactions_in_parallel(
        //     rpc_client,
        //     Some(tpu_client),
        //     &messages,
        //     &[signer],
        //     SendAndConfirmConfig {
        //         resign_txs_count: Some(5),
        //         with_spinner: true,
        //     },
        // )
        //     .await
        //     .map_err(|err| format!("Data writes to account failed: {err}"))?
        //     .into_iter()
        //     .flatten()
        //     .collect::<Vec<_>>();
    };
}

fn main() {
}
