use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3, CreateMetadataAccountsV3, Metadata},
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::{
    accounts::{MasterEdition, Metadata as MetadataAccount},
    types::DataV2,
};

declare_id!("CXGL4hPhcChVXwZX3cz6F8E6wfjqbEWWTkKzAdoc9ZFn");

#[program]
pub mod nft_mint_vault {
    use anchor_spl::token::{self, Transfer};

    use super::*;

    pub fn init_nft(
        ctx: Context<InitNFT>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        // create mint account
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.ata.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        );

        mint_to(cpi_context, 1)?;

        // create metadata account
        let cpi_context = CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.signer.to_account_info(),
                update_authority: ctx.accounts.signer.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );

        let data_v2 = DataV2 {
            name: name,
            symbol: symbol,
            uri: uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        create_metadata_accounts_v3(cpi_context, data_v2, false, true, None)?;

        //create master edition account
        let cpi_context = CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.master_edition_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.signer.to_account_info(),
                mint_authority: ctx.accounts.signer.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                metadata: ctx.accounts.metadata_account.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );

        create_master_edition_v3(cpi_context, None)?;

        Ok(())
    }

    pub fn create_vault(ctx: Context<CreateVault>, nft_mint: Pubkey) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = *ctx.accounts.owner.to_account_info().key;
        vault.nft_mint = nft_mint;
        vault.is_locked = false;
        Ok(())
    }

    pub fn lock_nft(ctx: Context<LockNft>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        require!(
            vault.owner == *ctx.accounts.owner.key,
            CustomError::Unauthorized
        );
        require!(!vault.is_locked, CustomError::AlreadyLocked);

        // Transfer NFT to vault
        let cpi_accounts: Transfer = Transfer {
            from: ctx.accounts.nft_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, 1)?;

        vault.is_locked = true;

        Ok(())
    }

    pub fn create_swap(ctx: Context<CreateSwap>, nft_mint: Pubkey, price: u64) -> Result<()> {
        let swap = &mut ctx.accounts.swap;
        swap.nft_mint = nft_mint;
        swap.seller = *ctx.accounts.seller.to_account_info().key;
        swap.price = price;
        Ok(())
    }

    pub fn execute_swap(ctx: Context<ExecuteSwap>) -> Result<()> {
        let swap = &ctx.accounts.swap;

        require!(
            ctx.accounts.buyer.to_account_info().lamports() >= swap.price,
            CustomError::InsufficientFunds
        );

        // Transfer SOL from buyer to seller
        **ctx
            .accounts
            .buyer
            .to_account_info()
            .try_borrow_mut_lamports()? -= swap.price;
        **ctx.accounts.seller.try_borrow_mut_lamports()? += swap.price;

        // Transfer NFT from seller to buyer
        let cpi_accounts = Transfer {
            from: ctx.accounts.nft_token_account.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.seller.clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, 1)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitNFT<'info> {
    /// CHECK: ok, we are passing in this account ourselves
    #[account(mut, signer)]
    pub signer: AccountInfo<'info>,
    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = signer.key(),
        mint::freeze_authority = signer.key(),
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub ata: Account<'info, TokenAccount>,
    /// CHECK: we are about to create this with metaplex
    #[account(mut, address = MetadataAccount::find_pda(&mint.key()).0)]
    pub metadata_account: AccountInfo<'info>,
    /// CHECK: we are about to create this with metaplex
    #[account(mut, address = MasterEdition::find_pda(&mint.key()).0)]
    pub master_edition_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(init, payer = owner, space = 8 + 32 + 32 + 1)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LockNft<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub nft_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateSwap<'info> {
    #[account(init, payer = seller, space = 8 + 32 + 32 + 8)]
    pub swap: Account<'info, Swap>,
    #[account(mut)]
    pub seller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteSwap<'info> {
    #[account(mut)]
    pub swap: Account<'info, Swap>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: this is created by the seller
    #[account(mut)]
    pub seller: AccountInfo<'info>,
    #[account(mut)]
    pub nft_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Swap {
    pub nft_mint: Pubkey,
    pub seller: Pubkey,
    pub price: u64,
}

#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub nft_mint: Pubkey,
    pub is_locked: bool,
}

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized.")]
    Unauthorized,
    #[msg("NFT is already locked.")]
    AlreadyLocked,
    #[msg("Insufficient funds to execute swap.")]
    InsufficientFunds,
}
