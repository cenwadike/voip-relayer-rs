# VoIP finance one way ETH -> SOL relayer.

VoIP finance one way ETH -> SOL relayer is a software tool designed to automate the bridging of VOIP ERC20 token on EThereum to VOIP SPL token on Solana.

Has admin privilege on Migration Solana program.
Also has admin privilege on Bridge contract on Ethereum.

It listens to `TokensLocked` events on Ethereum from bridge contract.
Uses logs from event to migrate VOIP SPL token from admin account to user's Solana address.

## Setup

To run the script, you need to:

- Create a new empty Ethereum wallet.
- Transfer some Ether to it
- Create a new empty Solana wallet.
- Transfer some Sol to it
- Deploy relevant contracts with wallet.
- Configure the script by updating `.env` file.
  - Check [Configuration](#configuration) section below.
- Install dependencies by running: `npm i`.
- Compile by running `cargo build`.
- Run the script by running `cargo run` in terminal.
- If you prefer to run this in docker, you can either use:
  1. Use docker engine `docker build -t voip-eth-to-sol-relayer-rs .  && docker run voip-eth-to-sol-relayer-rs`.
  2. Use docker compose `docker-compose up` (add -d for auto-restart mode).
- If you need scalability (eg. on kubernetes with different .env files), you can run `docker-compose up --scale app=3`.

### Configuration

#### Wallets

- SOLANA_ADMIN_PRIVATE_KEY
- ETHEREUM_ADMIN_ADDRESS
- ETHEREUM_ADMIN_PRIVATE_KEY

#### Connections

- SOLANA_RPC_ENDPOINT
- ETHEREUM_WSS_RPC_ENDPOINT
- ETHEREUM_HTTP_RPC_ENDPOINT

#### Contracts

- ETH_VOIP_TOKEN_ADDRESS
- ETH_BRIDGE_CONTRACT_ADDRESS
- SOL_VOIP_TOKEN_PROGRAM_ID
- SOL_VOIP_TOKEN_MINT
- SOL_MIGRATION_PROGRAM_ID
