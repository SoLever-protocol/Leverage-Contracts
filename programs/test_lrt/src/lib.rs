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
use mpl_token_metadata::types::DataV2;

const ADMIN_PUBKEY: Pubkey = pubkey!("8Vog23RLStZ3H8vEZMW7tCMow687Xba6EAarhd5f4UU");
const EVSOL_SEED: &[u8] = b"evSOL";

declare_id!("GWukmhTitefhHyGpz3G8a6e5RbGGWaiEJgCdRWpMfXYj");

#[program]
pub mod test_lrt {
    use super::*;

    pub fn create_mint(
        ctx: Context<CreateMint>,
        uri: String,
        name: String,
        symbol: String,
    ) -> Result<()> {
        // NOTE: unsure about this; does the * get the pointer to the context in memory?
        // actually seems like it's dereferencing ctx 
        //let bump = *ctx.bumps.get("reward_token_mint").unwrap();
        let bump = ctx.bumps.evsol_mint;

        let signer: &[&[&[u8]]] = &[&[EVSOL_SEED, &[bump]]];
        
        // token metadata that we'll put on-chain
        // guessing this will be put into the mint account
        let data_v2 = DataV2 {
            name: name,
            symbol: symbol,
            uri: uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(), 
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.evsol_mint.to_account_info(),
                mint_authority: ctx.accounts.evsol_mint.to_account_info(),
                update_authority: ctx.accounts.evsol_mint.to_account_info(),
                payer: ctx.accounts.admin.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info()
            }, 
            signer,
        );

        create_metadata_accounts_v3(
            cpi_ctx,
            data_v2,
            true,
            true,
            None,
        )?;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // tokens, either wSOL or LSTs, go in
        // evSOL comes out

        // use CPI to transfer the tokens of interest to the contract pool
        // we are not signing anything from this program, it should already have been signed by the caller
        transfer_tokens(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferTokens {
                    from: ctx.accounts.deposit_from.to_account_info(),
                    to: ctx.accounts.deposit_to.to_account_info(),// TODO add pool account
                    authority: ctx.accounts.depositor_signer.to_account_info()
                }
            ),
            // TODO: add amount here
            amount
        )?;

        
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

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct Deposit<'info> {
    // where to mint to
    // perhaps will need to change the constraint
    // perhaps make this a PDA and add an init constraint
    #[account(
        init_if_needed, 
        payer = depositor_signer,
        associated_token::mint = evsol_mint,
        associated_token::authority = depositor_signer
    )]
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
    // TODO: URGENT!!! Otherwise, can deposit wherever you feel like and get evSOL
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
    // necessary for PDA inited associated token account
    // --that is, for the evSOL we mint
    pub associated_token_program: Program<'info, AssociatedToken>,

    // system program
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(
        mut,
        address = ADMIN_PUBKEY
    )]
    pub admin: Signer<'info>,

    // this same PDA is used as both the address of the mint account and the authority
    // but this implies they could be different(??)
    // also interesting that the mint account seems like it is/must be owned by this program
    #[account(
        init,
        seeds = [EVSOL_SEED],
        bump,
        payer = admin,
        mint::decimals = 9,
        mint::authority = evsol_mint,
    )]
    pub evsol_mint: Account<'info, Mint>,

    ///CHECK: Using "address" constraint to validate metadata account address
    #[account(
        mut,
        address = mpl_token_metadata::accounts::Metadata::find_pda(&evsol_mint.key()).0,
    )]
    pub metadata_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}
