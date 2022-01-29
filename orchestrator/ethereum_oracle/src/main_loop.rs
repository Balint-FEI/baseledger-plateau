//! This file contains the main loop for ethereum oracle

use crate::ethereum_event_watcher::get_block_delay;
use crate::{ethereum_event_watcher::check_for_events, oracle_resync::get_last_checked_block};
use clarity::{address::Address as EthAddress, Uint256};
use deep_space::Contact;
use deep_space::{client::ChainStatus};
use deep_space::{coin::Coin, private_key::PrivateKey as CosmosPrivateKey};
use baseledger_proto::baseledger::query_client::QueryClient as BaseledgerQueryClient;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep as delay_for;
use tonic::transport::Channel;
use web30::client::Web3;

// TODO: this import should probably be somewhere globally, recheck and remove
use log::trace;
use log::info;
use log::error;
use log::warn;
/// The execution speed governing all loops in this file
/// which is to say all loops started by Orchestrator main
/// loop except the relayer loop
pub const ETH_ORACLE_LOOP_SPEED: Duration = Duration::from_secs(13);

/// This loop combines the three major roles required to make
/// up the 'Orchestrator', all three of these are async loops
/// meaning they will occupy the same thread, but since they do
/// very little actual cpu bound work and spend the vast majority
/// of all execution time sleeping this shouldn't be an issue at all.
#[allow(clippy::too_many_arguments)]
pub async fn orchestrator_main_loop(
    cosmos_key: CosmosPrivateKey,
    web3: Web3,
    contact: Contact,
    grpc_client: BaseledgerQueryClient<Channel>,
    baseledger_contract_address: EthAddress,
    user_fee_amount: Coin,
) {
    let fee = user_fee_amount;

    eth_oracle_main_loop(
        cosmos_key,
        web3.clone(),
        contact.clone(),
        grpc_client.clone(),
        baseledger_contract_address,
        fee.clone(),
    ).await;
}

const DELAY: Duration = Duration::from_secs(5);

/// This function is responsible for making sure that Ethereum events are retrieved from the Ethereum blockchain
/// and ferried over to Cosmos where they will be used to issue tokens or process batches.
pub async fn eth_oracle_main_loop(
    cosmos_key: CosmosPrivateKey,
    web3: Web3,
    contact: Contact,
    grpc_client: BaseledgerQueryClient<Channel>,
    baseledger_contract_address: EthAddress,
    fee: Coin,
) {
    let our_cosmos_address = cosmos_key.to_address(&contact.get_prefix()).unwrap();
    let long_timeout_web30 = Web3::new(&web3.get_url(), Duration::from_secs(120));
    let block_delay = get_block_delay(&web3).await;
    let mut last_checked_block: Uint256 = get_last_checked_block(
        grpc_client.clone(),
        our_cosmos_address,
        contact.get_prefix(),
        baseledger_contract_address,
        &long_timeout_web30,
    )
    .await;

    // In case of governance vote to unhalt bridge, need to replay old events. Keep track of the
    // last checked event nonce to detect when this happens
    let mut last_checked_event: Uint256 = 0u8.into();
    info!("Oracle resync complete, Oracle now operational");
    let mut grpc_client = grpc_client;

    loop {
        let loop_start = Instant::now();

        let latest_eth_block = web3.eth_block_number().await;
        let latest_cosmos_block = contact.get_chain_status().await;

        match (latest_eth_block, latest_cosmos_block) {
            (Ok(latest_eth_block), Ok(ChainStatus::Moving { block_height })) => {
                trace!(
                    "Latest Eth block {} Latest Cosmos block {}",
                    latest_eth_block,
                    block_height,
                );
            }
            (Ok(_latest_eth_block), Ok(ChainStatus::Syncing)) => {
                warn!("Cosmos node syncing, Eth oracle paused");
                delay_for(DELAY).await;
                continue;
            }
            (Ok(_latest_eth_block), Ok(ChainStatus::WaitingToStart)) => {
                warn!("Cosmos node syncing waiting for chain start, Eth oracle paused");
                delay_for(DELAY).await;
                continue;
            }
            (Ok(_), Err(_)) => {
                warn!("Could not contact Cosmos grpc, trying again");
                delay_for(DELAY).await;
                continue;
            }
            (Err(_), Ok(_)) => {
                warn!("Could not contact Eth node, trying again");
                delay_for(DELAY).await;
                continue;
            }
            (Err(_), Err(_)) => {
                error!("Could not reach Ethereum or Cosmos rpc!");
                delay_for(DELAY).await;
                continue;
            }
        }

        // Relays events from Ethereum -> Cosmos
        match check_for_events(
            &web3,
            &contact,
            &mut grpc_client,
            baseledger_contract_address,
            cosmos_key,
            fee.clone(),
            last_checked_block.clone(),
            block_delay.clone(),
        )
        .await
        {
            Ok(nonces) => {
                // this output CheckedNonces is accurate unless a governance vote happens
                last_checked_block = nonces.block_number;
                if last_checked_event > nonces.event_nonce {
                    // validator went back in history
                    info!(
                        "Governance unhalt vote must have happened, resetting the block to check!"
                    );
                    last_checked_block = get_last_checked_block(
                        grpc_client.clone(),
                        our_cosmos_address,
                        contact.get_prefix(),
                        baseledger_contract_address,
                        &web3,
                    )
                    .await;
                }
                last_checked_event = nonces.event_nonce;
            }
            Err(e) => error!(
                "Failed to get events for block range, Check your Eth node and Cosmos gRPC {:?}",
                e
            ),
        }

        // a bit of logic that tires to keep things running every LOOP_SPEED seconds exactly
        // this is not required for any specific reason. In fact we expect and plan for
        // the timing being off significantly
        let elapsed = Instant::now() - loop_start;
        if elapsed < ETH_ORACLE_LOOP_SPEED {
            delay_for(ETH_ORACLE_LOOP_SPEED - elapsed).await;
        }
    }
}


// Checks for fee errors on our confirm submission transactions, a failure here
// can be fatal and cause slashing so we want to warn the user and exit. There is
// no point in running if we can't perform our most important function
// fn check_for_fee_error(res: Result<TxResponse, CosmosGrpcError>, fee: &Coin) {
//     if let Err(CosmosGrpcError::InsufficientFees { fee_info }) = res {
//         match fee_info {
//             FeeInfo::InsufficientFees { min_fees } => {
//                 error!(
//                     "Your specified fee value {} is too small please use at least {}",
//                     fee,
//                     Coin::display_list(&min_fees)
//                 );
//                 error!("Correct fee argument immediately! You will be slashed within a few hours if you fail to do so");
//                 exit(1);
//             }
//             FeeInfo::InsufficientGas { .. } => {
//                 panic!("Hardcoded gas amounts insufficient!");
//             }
//         }
//     }
// }