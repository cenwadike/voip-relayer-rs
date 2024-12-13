use anchor_client::anchor_lang::prelude::AccountMeta;
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::hash::Hash;
use anchor_client::{
    solana_sdk::{
        instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
        transaction::Transaction,
    },
    Client, Cluster,
};
use dotenv::dotenv;
use hex_literal::hex;
use std::env;
use std::rc::Rc;
use std::str::FromStr;
use web3::contract::{Contract, Options};
use web3::ethabi::{decode, ParamType};
use web3::futures::StreamExt;
use web3::signing;
use web3::types::{Address, FilterBuilder, H160, U256};

#[tokio::main]
async fn main() {
    // load .env file
    dotenv().ok();

    // verify environment variables
    let sol_admin_private_key =
        env::var("SOLANA_ADMIN_PRIVATE_KEY").expect("Failed to get SOLANA_ADMIN_PRIVATE_KEY");
    let sol_admin_keypair = Keypair::from_base58_string(&sol_admin_private_key);

    let eth_admin_address =
        env::var("ETHEREUM_ADMIN_ADDRESS").expect("Failed to get ETHEREUM_ADMIN_ADDRESS");
    let eth_admin_private_key =
        env::var("ETHEREUM_ADMIN_PRIVATE_KEY").expect("Failed to get ETHEREUM_ADMIN_PRIVATE_KEY");

    let sol_rpc_endpoint =
        env::var("SOLANA_RPC_ENDPOINT").expect("Failed to get SOLANA_RPC_ENDPOINT");
    let eth_wss_rpc_endpoint =
        env::var("ETHEREUM_WSS_RPC_ENDPOINT").expect("Failed to get ETHEREUM_WSS_RPC_ENDPOINT");
    let eth_http_rpc_endpoint =
        env::var("ETHEREUM_HTTP_RPC_ENDPOINT").expect("Failed to get ETHEREUM_HTTP_RPC_ENDPOINT");
    let eth_voip_bridge_address =
        env::var("ETH_BRIDGE_CONTRACT_ADDRESS").expect("Failed to get ETH_BRIDGE_CONTRACT_ADDRESS");
    let sol_voip_mint_address =
        env::var("SOL_VOIP_TOKEN_MINT").expect("Failed to get SOL_VOIP_TOKEN_MINT");
    let sol_voip_migration_address =
        env::var("SOL_MIGRATION_PROGRAM_ID").expect("Failed to get SOL_MIGRATION_PROGRAM_ID");

    // print logs
    println!(
        "
                  VVVVVVVV           VVVVVVVV
                  V::::::V           V::::::V
                  V::::::V           V::::::V
                  V::::::V           V::::::V
                   V:::::V           V:::::V 
                    V:::::V         V:::::V  
                     V:::::V       V:::::V   
                      V:::::V     V:::::V    
                       V:::::V   V:::::V     
                        V:::::V V:::::V      
                         V:::::V:::::V       
                          V:::::::::V        
                           V:::::::V         
                            V:::::V          
                             V:::V           
                              VVV                                          
                  
                VOIP FINANCE RELAYER ACTIVATED 🚀
                      Made with ❤️ by Kombi.         
    "
    );

    println!(
        "
                ------- CONFIGURATION START -------
    "
    );
    println!(
        "
        SOL Admin Wallet Address:  {}
        ",
        sol_admin_keypair.pubkey().to_string()
    );
    println!(
        "
        ETH Admin Wallet Address: {}
        ",
        eth_admin_address.to_owned()
    );

    println!(
        "
        SOL Migration Contract Address: {}
        ",
        sol_voip_migration_address
    );
    println!(
        "
        ETH Bridge Contract Address: {}
        ",
        eth_voip_bridge_address
    );
    println!(
        "
        SOL Token Mint Address: {}
        ",
        sol_voip_mint_address
    );

    println!(
        "
            ------ CONFIGURATION COMPLETE -------
    "
    );
    println!(
        "
            Relayer is starting...
    "
    );

    let sol_voip_token_mint = Pubkey::from_str(&sol_voip_mint_address).expect(
        "
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
              Failed to parse solana VOIP mint 
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                Status:      Failed❌
        ",
    );

    let sol_admin_pubkey = sol_admin_keypair.pubkey();

    let eth_admin_private_key = signing::SecretKey::from_str(&eth_admin_private_key).expect(
        "
        
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
              Failed to parse admin private key
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    Status:      Failed❌
        
    ",
    );

    loop {
        let _ = run_relayer(
            &eth_wss_rpc_endpoint,
            &eth_http_rpc_endpoint,
            &eth_voip_bridge_address,
            &eth_admin_private_key,
            &sol_rpc_endpoint,
            &sol_voip_token_mint,
            &sol_admin_pubkey,
            &sol_admin_private_key,
            &sol_admin_keypair,
            &sol_voip_migration_address,
        )
        .await;
    }
}

async fn run_relayer(
    eth_wss_rpc_endpoint: &str,
    eth_http_rpc_endpoint: &str,
    eth_voip_bridge_address: &str,
    eth_admin_private_key: &signing::SecretKey,
    sol_rpc_endpoint: &str,
    sol_voip_token_mint: &Pubkey,
    sol_admin_pubkey: &Pubkey,
    sol_admin_private_key: &str,
    sol_admin_keypair: &Keypair,
    sol_voip_migration_address: &str,
) -> web3::contract::Result<()> {
    // --------------------- Set up eth connections --------------------- //
    // set up websocket transport layer
    let wss_transport = web3::transports::WebSocket::new(eth_wss_rpc_endpoint).await;

    // set up websocket connection
    let web3 = match wss_transport {
        Ok(wss) => web3::Web3::new(wss),
        Err(err) => panic!(
            "
                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                  Failed to setup transport layer, Use a dedicated Ethereum Websocket URL
                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                        Status:      Failed❌
                                        Error:       {err}
            "
        ),
    };

    // set up http transport layer
    let http_transport = web3::transports::Http::new(eth_http_rpc_endpoint);

    // set up http connection
    let http_web3: web3::Web3<web3::transports::Http> = match http_transport {
        Ok(http) => web3::Web3::new(http),
        Err(err) => panic!(
            "
                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    Failed to setup transport layer, Use a dedicated Ethereum HTTP URL
                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                        Status:      Failed❌
                                        Error:       {err}
            "
        ),
    };

    // create eth bridge contract instance
    let contract_address = eth_voip_bridge_address.parse();
    let contract = match contract_address {
        Ok(address) => Contract::from_json(
            http_web3.eth(),
            address,
            include_bytes!("../artifacts/eth/bridge/bridge.json"),
        )?,
        Err(err) => panic!(
            "
                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                            Failed to Parse Eth Bridge Contract Address
                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                        Status:      Failed❌
                                        Error:       {err}
            "
        ),
    };

    // --------------------- Set up sol connections --------------------- //
    // Create sol rpc connection
    let connection = RpcClient::new(sol_rpc_endpoint);

    // Create client
    let payer = Keypair::from_base58_string(sol_admin_private_key);
    let cluster = Cluster::from_str(sol_rpc_endpoint).expect(
        "
    
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                Failed to get solana cluster
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    Status:      Failed❌
    ",
    );
    let client = Client::new(cluster, Rc::new(payer));

    // Create program
    let voip_migration_program_id = Pubkey::from_str(sol_voip_migration_address).expect(
        "
    
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
              Failed to get migration program ID
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    Status:      Failed❌
        ",
    );
    let program = client.program(voip_migration_program_id).expect(
        "
    
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
            Failed to get solana migration program
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    Status:      Failed❌
        ",
    );

    // --------------------- Set up sol constants --------------------- //
    // solana token program id
    let token_program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").expect(
        "
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
               Failed to parse token program id
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    Status:      Failed❌
        ",
    );

    // solana system program id
    let system_program_id = Pubkey::from_str("11111111111111111111111111111111").expect(
        "
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
               Failed to parse system program id
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    Status:      Failed❌
    ",
    );

    // solana associated token program id
    let associated_token_program_id =
        Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").expect(
            "
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
            Failed to parse associated program id
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    Status:      Failed❌
    ",
        );

    // get state PDA
    let (state_pda, _) = Pubkey::find_program_address(&[&b"state"[..]], &voip_migration_program_id);

    // admin ata
    let (admin_ata, _) = Pubkey::find_program_address(
        &[
            &sol_admin_pubkey.as_ref(),
            &token_program_id.as_ref(),
            &sol_voip_token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    // decimal 10**9
    let decimals = U256::from(1_000_000_000);

    // --------------------- Set up TokensLocked event filter --------------------- //
    // filter TokensLocked event
    let filter = FilterBuilder::default()
        .address(vec![Address::from_str(eth_voip_bridge_address).unwrap()])
        .topics(
            // this is 'TokensLocked(uint256 amount, address indexed user, string solanaAddress, uint256 timestamp)' event
            // use https://emn178.github.io/online-tools/keccak_256.html,
            // and type in 'TokensLocked(uint256,address,string,uint256)' (without the quote)
            // it will return result hash as used in next line
            Some(vec![hex!(
                "a3c29410d4173cda5ec6e52fca2d334b67df70664e718bee6d216e089b408442"
            )
            .into()]),
            None,
            None,
            None,
        )
        .build();

    // --------------------- Subscribe to TokensLocked event --------------------- //
    // subscribe to event
    let subs = web3.eth_subscribe().subscribe_logs(filter).await?;

    // --------------------- Orchestrate bridging for each event --------------------- //
    subs.for_each_concurrent(20, |log| {
        println!(
            "
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                  Processing New Migration
            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    Status:      Processing ♻️
        "
        );

        let contract = contract.clone();
        let program = &program;
        let connection = &connection;
        async move {
            match log.clone() {
                Ok(log) => {
                    let decoded_amount = decode(&[ParamType::Uint(256)], &log.topics[1].as_bytes());

                    let mut amount = 0u128;
                    match decoded_amount {
                        Ok(amount_vec) => {
                            let amount_uint = amount_vec[0].clone().into_uint();
                            match amount_uint {
                                Some(amount_u) => {
                                    let amount_u256 = amount_u * decimals;
                                    amount = amount_u256.as_u128();
                                }
                                None => {
                                    println!(
                                        "
                                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                        Decoding Failed: Invalid Amount Provided
                                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                Status:      Failed❌
                                    "
                                    );
                                }
                            };
                        }
                        Err(err) => {
                            println!(
                                "
                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                    Failed to parse amount from event
                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                        Status:      Failed❌
                                        Error:       {err}
                            "
                            )
                        }
                    };

                    let decoded_eth_address =
                        decode(&[ParamType::Address], &log.topics[2].as_bytes());

                    let mut eth_address = H160::zero();
                    match decoded_eth_address {
                        Ok(eth_address_vec) => {
                            let eth_address_addr = eth_address_vec[0].clone().into_address();
                            match eth_address_addr {
                                Some(address) => eth_address = address,
                                None => {
                                    println!(
                                        "
                                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                     Decoding Failed: Invalid Eth Address Provided
                                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                Status:      Failed❌
                                    "
                                    );
                                }
                            }
                        }
                        Err(err) => {
                            println!(
                                "
                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                    Failed to parse eth address from event
                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                        Status:      Failed❌
                                        Error:       {err}
                            "
                            )
                        }
                    }

                    let decoded_solana_address = decode(&[ParamType::String], &log.data.0);
                    let mut solana_address = Pubkey::default();

                    match decoded_solana_address {
                        Ok(solana_address_vec) => {
                            let solana_address_str_op = solana_address_vec[0].clone().into_string();
                            match solana_address_str_op {
                                Some(address_str) => {
                                    let solana_address_pk = Pubkey::from_str(&address_str);
                                    match solana_address_pk {
                                        Ok(solana_address_pubkey) => {
                                            solana_address = solana_address_pubkey;
                                        }
                                        Err(err) => {
                                            println!(
                                                "
                                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                Decoding Failed: Failed to parse Sol Address
                                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                        Status:      Failed❌
                                                        Error:       {err}
                                            "
                                            )
                                        }
                                    }
                                }
                                None => {
                                    println!(
                                        "
                                        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                        Decoding Failed: Invalid Sol Address Provided
                                        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                Status:      Failed❌
                                    "
                                    )
                                }
                            }
                        }
                        Err(err) => {
                            println!(
                                "
                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                    Failed to parse sol address from event
                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                        Status:      Failed❌
                                        Error:       {err}
                            "
                            )
                        }
                    }

                    // assert decoding pass before proceeding
                    if amount != 0u128
                        && eth_address != H160::zero()
                        && solana_address != Pubkey::default()
                    {
                        // migrate token
                        let sol_migration_hash = migrate(
                            connection,
                            &program,
                            &state_pda,
                            sol_voip_token_mint,
                            &voip_migration_program_id,
                            sol_admin_pubkey,
                            sol_admin_keypair,
                            &admin_ata,
                            &solana_address,
                            &token_program_id,
                            &associated_token_program_id,
                            &system_program_id,
                            &amount,
                        )
                        .await;

                        match sol_migration_hash {
                            Ok(hash) => match hash {
                                Ok(signature) => {
                                    // burn VOIP tokens on ethereum
                                    let eth_burn_receipt = burn(
                                        &eth_admin_private_key,
                                        &contract,
                                        &eth_address,
                                        &solana_address,
                                    )
                                    .await;

                                    println!(
                                        "
                                            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                            Successfully migrated SOL VOIP Token
                                            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                Eth Address:  {eth_address}
                                                SOL Address:  {solana_address}
                                                Tx Hash:      {signature}
                                                TX Status:    Success✅
                                        "
                                    );
                                    match eth_burn_receipt {
                                        Ok(receipt_rs) => match receipt_rs {
                                            Ok(receipt) => {
                                                let receipt_hash =
                                                    receipt.transaction_hash.to_string();
                                                println!(
                                                    "
                                                            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                            Successfully burnt ETH VOIP Token
                                                            ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                                ETH Address:  {eth_address}
                                                                Sol Address:  {solana_address}
                                                                Tx Hash:      {receipt_hash}
                                                                TX Status:    Success✅

                                                        "
                                                );
                                                println!(
                                                    "
                                                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                          Processed New Migration 💥
                                                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                            ETH Address:  {eth_address}
                                                            Sol Address:  {solana_address}
                                                            Amount:       {amount}
                                                            Sol Tx Hash:  {signature}
                                                            Eth Tx Hash:  {receipt_hash}
                                                            Status:       Success✅
                                                "
                                                );
                                            }
                                            Err(err) => {
                                                println!(
                                                    "
                                                        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                            Failed to Burn ETH VOIP tokens
                                                        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                            ETH Address: {eth_address}
                                                            Sol Address: {solana_address}
                                                            Status:      Failed❌
                                                            Error:       {err}
                                                    "
                                                )
                                            }
                                        },
                                        Err(err) => {
                                            println!(
                                                "
                                                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                        Failed to Burn ETH VOIP tokens
                                                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                        ETH Address: {eth_address}
                                                        Sol Address: {solana_address}
                                                        Status:      Failed❌
                                                        Error:       {err}
                                                "
                                            )
                                        }
                                    }
                                }
                                Err(err) => {
                                    println!(
                                        "
                                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                  Failed to Migrate SOL VOIP Tokens
                                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                                    Eth Address: {eth_address}
                                                    Sol Address: {solana_address}
                                                    Status:      Failed❌
                                                    Error Type:  Client Error
                                                    Error:       {err}
                                                "
                                    )
                                }
                            },
                            Err(err) => {
                                println!(
                                    "
                                        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                            Failed to Migrate SOL VOIP Tokens
                                        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                            Eth Address: {eth_address}
                                            Sol Address: {solana_address}
                                            Status:      Failed❌
                                            Error:       {err}
                                        "
                                )
                            }
                        }
                    } else {
                        println!(
                            "
                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                        Failed to Decode Event
                                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                        Status:      Failed❌
                                "
                        )
                    }
                }
                Err(err) => {
                    println!(
                        "
                        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                Failed to Read Log
                        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                                Status:      Failed❌
                                Error:       {err}
                    "
                    )
                }
            }
        }
    })
    .await;

    Ok(())
}

async fn migrate(
    connection: &RpcClient,
    program: &anchor_client::Program<Rc<Keypair>>,
    state_pda: &Pubkey,
    sol_voip_token_mint: &Pubkey,
    sol_voip_migration_program_id: &Pubkey,
    sol_admin_pubkey: &Pubkey,
    sol_admin_keypair: &Keypair,
    sol_admin_ata: &Pubkey,
    solana_address: &Pubkey,
    token_program_id: &Pubkey,
    associated_token_program_id: &Pubkey,
    system_program_id: &Pubkey,
    amount: &u128,
) -> Result<
    Result<anchor_client::solana_sdk::signature::Signature, anchor_client::ClientError>,
    Box<dyn std::error::Error>,
> {
    // --------------------- set up ATAs --------------------- //
    // admin ata
    let admin_ata = sol_admin_ata;

    // derive destination ata
    let (destination_ata, _) = Pubkey::find_program_address(
        &[
            &solana_address.as_ref(),
            &token_program_id.as_ref(),
            &sol_voip_token_mint.as_ref(),
        ],
        &associated_token_program_id,
    );

    // check if derived account has been created
    let destination_account = connection.get_account(&destination_ata);

    // create destination ATA if it does not exist
    if destination_account.is_err() {
        // get recent block hash
        let recent_blockhash = connection.get_latest_blockhash();
        let mut latest_blockhash = Hash::default();
        match recent_blockhash {
            Ok(hash) => {
                latest_blockhash = hash;
            }
            Err(err) => {
                println!(
                    "
                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                            Failed to Get Recent Hash
                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                            Status:      Failed❌
                            Error:       {err}
                "
                )
            }
        }

        // construct create destination ATA instruction
        let create_destination_ata_ix = Instruction {
            program_id: associated_token_program_id.clone(),
            accounts: vec![
                AccountMeta::new(sol_admin_pubkey.clone(), true),
                AccountMeta::new(destination_ata, false),
                AccountMeta::new_readonly(solana_address.clone(), false),
                AccountMeta::new_readonly(sol_voip_token_mint.clone(), false),
                AccountMeta::new_readonly(system_program_id.clone(), false),
                AccountMeta::new_readonly(token_program_id.clone(), false),
            ],
            data: vec![0],
        };

        // create create destination ATA transaction
        let mut transaction =
            Transaction::new_with_payer(&[create_destination_ata_ix], Some(&sol_admin_pubkey));

        // sign create destination ATA transaction
        transaction.sign(&[&sol_admin_keypair], latest_blockhash);

        // send and confirm transaction
        match connection.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                println!(
                    "
                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                        Created associated token account
                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                            Status:      Success✅
                            Tx Hash:     {signature}
                    ",
                );
            }
            Err(err) => {
                println!(
                    "
                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    Failed to create associated token account
                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                            Status:      Failed❌
                            Error:       {err}
                "
                );
            }
        }
    }

    // get migration PDA
    let (migration_pda, _) = Pubkey::find_program_address(
        &[&b"migration"[..], &solana_address.as_ref()],
        &sol_voip_migration_program_id,
    );

    // call migrate function
    let migrate_transaction_hash = program
        .request()
        .accounts(voip_migration::accounts::Migrate {
            migration: migration_pda,
            state: state_pda.clone(),
            destination_ata,
            admin_ata: admin_ata.clone(),
            admin: sol_admin_pubkey.clone(),
            destination: solana_address.clone(),
            mint: sol_voip_token_mint.clone(),
            token_program: token_program_id.clone(),
            system_program: system_program_id.clone(),
            associated_token_program: associated_token_program_id.clone(),
        })
        .args(voip_migration::instruction::Migrate {
            amount: *amount as u64,
        })
        .signer(&sol_admin_keypair)
        .send()
        .await;

    Ok(migrate_transaction_hash)
}

async fn burn(
    eth_admin_private_key: &signing::SecretKey,
    contract: &Contract<web3::transports::Http>,
    ethereum_address: &H160,
    solana_address: &Pubkey,
) -> Result<Result<web3::types::TransactionReceipt, web3::Error>, Box<dyn std::error::Error>> {
    let burn_transaction_receipt = contract
        .signed_call_with_confirmations(
            "burnTokens",
            (ethereum_address.clone(), solana_address.to_string()),
            Options::default(),
            1,
            eth_admin_private_key,
        )
        .await;

    Ok(burn_transaction_receipt)
}
