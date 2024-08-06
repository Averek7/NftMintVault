# NFT Mint Vault on Solana

This project is a Solana program written in Rust using the Anchor framework. It allows you to mint NFTs, create a vault for storing NFTs, and facilitate NFT swaps. 

## Features

1. **Minting NFTs**: Create new NFTs with associated metadata and master editions.
2. **Creating Vaults**: Store and lock NFTs in a secure vault.
3. **Swapping NFTs**: Swap NFTs between users with a set price.

## Prerequisites

- Rust and Cargo: [Install Rust](https://www.rust-lang.org/tools/install)
- Solana CLI: [Install Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- Anchor CLI: [Install Anchor](https://project-serum.github.io/anchor/getting-started/installation.html)

## Installation

1. Clone the repository:
   ```sh
     git clone https://github.com/yourusername/nft-mint-vault.git
     cd nft-mint-vault
   ```
2. Build the project:
   ```sh
     anchor build
   ```
3. Deploy the program to the local Solana cluster:
   ```sh
     anchor deploy
    ```
## Usage

# Minting an NFT
To mint an NFT, use the init_nft function.

```bash
pub fn init_nft(
    ctx: Context<InitNFT>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    // Implementation here...
}
```

# Creating a Vault
To create a vault for storing NFTs, use the create_vault function.

```bash
pub fn create_vault(ctx: Context<CreateVault>, nft_mint: Pubkey) -> Result<()> {
    // Implementation here...
}
```

# Locking an NFT in the Vault
To lock an NFT in the vault, use the lock_nft function.
```bash
pub fn lock_nft(ctx: Context<LockNft>) -> Result<()> {
    // Implementation here...
}
```

# Creating a Swap
To create a swap offer for an NFT, use the create_swap function.
```bash
pub fn create_swap(ctx: Context<CreateSwap>, nft_mint: Pubkey, price: u64) -> Result<()> {
    // Implementation here...
}
```

# Executing a Swap
To execute a swap and transfer the NFT to the buyer, use the execute_swap function.

```bash
pub fn execute_swap(ctx: Context<ExecuteSwap>) -> Result<()> {
    // Implementation here...
}
```

## Test
To run the tests, use the following command:
```bash
anchor test
```
