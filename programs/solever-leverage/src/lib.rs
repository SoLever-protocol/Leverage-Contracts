use anchor_lang::prelude::*;
use test_lrt::{cpi::accounts::{Deposit, TokensDeposited}, program::TestLrt};
use anchor_spl::{
    associated_token::AssociatedToken, metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata}, token::{
        self, mint_to, transfer as transfer_tokens, Mint, MintTo, Token, TokenAccount, Transfer as TransferTokens
    }
};
use mpl_token_metadata::accounts::Metadata as MetadataAccount;


declare_id!("43MDCtWchEp5bEjte91xGDqGU2HCwsjxxoiPcaPfNBp4");

#[program]
pub mod solever_leverage {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn add_lrt(_ctx: Context<LRTInfo>) -> Result<()> {
        Ok(())
    }

    pub fn add_lst(_ctx: Context<LSTInfo>) -> Result<()> {
        // I think that the mint accounts for both the interest and principal have already been created
        Ok(())
    }

    pub fn lend(ctx: Context<LendInfo>, amount: u64) -> Result<()> {
        // TODO: check that the LST has already been added
        // specifically, that the principal and interest token mint accounts exist
        transfer_tokens(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferTokens {
                    from: ctx.accounts.deposit_from.to_account_info(),
                    to: ctx.accounts.lst_holding_account.to_account_info(),// TODO add pool account
                    authority: ctx.accounts.depositor_signer.to_account_info()
                }
            ),
            // TODO: add amount here
            amount
        )?;

        let bumps = [ctx.bumps.p_token_mint, ctx.bumps.i_token_mint];
        let p_i_seeds: [&[u8]; 2] = [b"principal", b"interest"];
        let lst_mint_key = ctx.accounts.lst_mint.key();
        let mint_signers: [&[&[&[u8]]];2] = [
            &[&[lst_mint_key.as_ref(), p_i_seeds[0], &[bumps[0]]]],
            &[&[lst_mint_key.as_ref(), p_i_seeds[1], &[bumps[1]]]]
        ];
        // use CPI to mint the principal token
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.p_token_mint.to_account_info(),
                    to: ctx.accounts.mint_p_to.to_account_info(),
                    // for now, make the mint account also the authority
                    authority: ctx.accounts.p_token_mint.to_account_info(),
                },
                mint_signers[0]
            ),
            // TODO: replace this amount with something that gets the evSOL price
            amount
        )?;
        // use CPI to mint the interest token. TODO: deal with potentially different rates.
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.i_token_mint.to_account_info(),
                    to: ctx.accounts.mint_i_to.to_account_info(),
                    // for now, make the mint account also the authority
                    authority: ctx.accounts.i_token_mint.to_account_info(),
                },
                mint_signers[1]
            ),
            // TODO: replace this amount with something that gets the evSOL price
            amount
        )?;

        Ok(())
    }

    //pub fn leverage_restake()
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct LRTInfo<'info> {
    // LRT program
    pub lrt_program: Program<'info, TestLrt>,

    // require an LRT holdings account
    #[account(
        init, 
        payer = user, 
        seeds = [lrt_mint.key().as_ref()],
        bump, 
        token::mint = lrt_mint,
        token::authority = lrt_holding_account
    )]
    pub lrt_holding_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    // system program
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub lrt_mint: Account<'info, Mint>,
}

#[derive(Accounts)]
pub struct LSTInfo<'info> {
    // require an LST holdings account
    #[account(
        init, 
        payer = user, 
        seeds = [lst_mint.key().as_ref()],
        bump, 
        token::mint = lst_mint,
        token::authority = lst_holding_account
    )]
    pub lst_holding_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    // system program
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub lst_mint: Account<'info, Mint>,
    
    // add some more stuff, since we need to initialize 
    // two mints respectively representing the principal and interest for lending
    #[account(
        init,
        seeds = [lst_mint.key().as_ref(), b"principal"],
        bump,
        payer = user,
        mint::decimals = 9,
        mint::authority = p_token_mint,
    )]
    pub p_token_mint: Account<'info, Mint>,

    // ///CHECK: Using "address" constraint to validate metadata account address
    // #[account(
    //     mut,
    //     address = mpl_token_metadata::accounts::Metadata::find_pda(&p_token_mint.key()).0,
    // )]
    // pub p_metadata_account: UncheckedAccount<'info>,

    #[account(
        init,
        seeds = [lst_mint.key().as_ref(), b"interest"],
        bump,
        payer = user,
        mint::decimals = 9,
        mint::authority = i_token_mint,
    )]
    pub i_token_mint: Account<'info, Mint>,

    // ///CHECK: Using "address" constraint to validate metadata account address
    // #[account(
    //     mut,
    //     address = mpl_token_metadata::accounts::Metadata::find_pda(&i_token_mint.key()).0,
    // )]
    // pub i_metadata_account: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct LendInfo<'info> {
    #[account(
        mut,
        associated_token::mint = lst_mint,
        associated_token::authority = depositor_signer
    )]
    pub deposit_from: Account<'info, TokenAccount>,

    pub lst_mint: Account<'info, Mint>,

    #[account(
        init_if_needed, 
        payer = depositor_signer,
        associated_token::mint = i_token_mint,
        associated_token::authority = depositor_signer
    )]
    pub mint_i_to: Account<'info, TokenAccount>,

    #[account(
        init_if_needed, 
        payer = depositor_signer,
        associated_token::mint = p_token_mint,
        associated_token::authority = depositor_signer
    )]
    pub mint_p_to: Account<'info, TokenAccount>,


    // must be mutable because pays fees
    #[account(mut)]
    pub depositor_signer: Signer<'info>,

    #[account(
        mut,
        seeds = [lst_mint.key().as_ref()],
        bump
    )]
    lst_holding_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [lst_mint.key().as_ref(), b"principal"],
        bump,
    )]
    pub p_token_mint: Account<'info, Mint>,

    #[account(
        seeds = [lst_mint.key().as_ref(), b"interest"],
        bump,
    )]
    pub i_token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct BorrowInfo<'info> {
    #[account(
        mut,
        associated_token::mint = lst_mint,
        associated_token::authority = depositor_signer
    )]
    pub deposit_from: Account<'info, TokenAccount>,

    pub lst_mint: Account<'info, Mint>,

    #[account(
        init_if_needed, 
        payer = depositor_signer,
        associated_token::mint = i_token_mint,
        associated_token::authority = depositor_signer
    )]
    pub mint_i_to: Account<'info, TokenAccount>,

    #[account(
        init_if_needed, 
        payer = depositor_signer,
        associated_token::mint = p_token_mint,
        associated_token::authority = depositor_signer
    )]
    pub mint_p_to: Account<'info, TokenAccount>,


    // must be mutable because pays fees
    #[account(mut)]
    pub depositor_signer: Signer<'info>,

    #[account(
        mut,
        seeds = [lst_mint.key().as_ref()],
        bump
    )]
    lst_holding_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [lst_mint.key().as_ref(), b"principal"],
        bump,
    )]
    pub p_token_mint: Account<'info, Mint>,

    #[account(
        seeds = [lst_mint.key().as_ref(), b"interest"],
        bump,
    )]
    pub i_token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}


