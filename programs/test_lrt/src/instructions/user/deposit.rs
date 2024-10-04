use anchor_lang::prelude::*;
// use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
    token::{
        mint_to, transfer as transfer_tokens, Mint, MintTo, Token, TokenAccount,
        Transfer as TransferTokens,
    },
    associated_token::AssociatedToken,
    metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata}
};

pub struct Deposit<'info> {
    // where to mint to
    // perhaps will need to change the constraint
    // perhaps make this a PDA and add an init constraint
    #[account(mut, token::mint = evsol_mint)]
    pub mint_to: Account<'info, TokenAccount>,

    // NOTE: don't need this, if we always use an SPL token
    // where to transfer from
    // perhaps need to check that the owner is either the token program or the system program
    //#[account(mut)]
    //pub transfer_from: Signer<'info>,

    // deposited token account
    // if depositing SOL, this would be the user's wrapped SOL account
    #[account(mut)]
    pub deposit_from: Account<'info, TokenAccount>,

    #[account(mut)]
    pub depositor_signer: Signer<'info>,

    // add constraints to compute this PDA based on the mint of deposit_from
    #[account(mut)]
    pub deposit_to: Account<'info, TokenAccount>,

    // evsol mint account
    //#[account]
    // perhaps make this a PDA
    #[account(
        mut,
        seeds = [EVSOL_SEED],
        bump
    )]
    pub evsol_mint: Account<'info, Mint>,

    // token program
    pub token_program: Program<'info, Token>,

    // system program
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        // tokens, either wSOL or LSTs, go in
        // evSOL comes out

        // use CPI to transfer the tokens of interest to the contract pool
        // we are not signing anything from this program, it should already have been signed by the caller
        transfer_tokens(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferTokens {
                    from: self.deposit_from.to_account_info(),
                    to: self.deposit_to.to_account_info(),// TODO add pool account
                    authority: self.depositor_signer.to_account_info()
                }
            ),
            // TODO: add amount here
            amount
        )?;

        
        // TODO: fix this.
        // marinade stores the bump in a state account and references here
        let bump = ctx.bumps.evsol_mint;
        let mint_signer: &[&[&[u8]]] = &[&[EVSOL_SEED, &[bump]]];
        // use CPI to mint an evSOL
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.evsol_mint.to_account_info(),
                    to: ctx.accounts.mint_to.to_account_info(),
                    // for now, make the mint account also the authority
                    authority: ctx.accounts.evsol_mint.to_account_info(),
                },
                mint_signer
            ),
            // TODO: replace this amount with something that gets the evSOL price
            amount
        )?;

        Ok(())
    }
}
