
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, MintTo, Transfer as TransferTokens};
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use test_lrt::program::test_lrt::test_lrt;
use mpl_token_metadata::{
    instruction::create_metadata_accounts_v3,
    state::{DataV2, Metadata},
};
use tokio;

// Set up constants for tests
const TOKEN_DECIMALS: u8 = 9;

// Mock Admin Pubkey
const ADMIN_PUBKEY: Pubkey = Pubkey::new_unique();

// Helper function to create a Token Mint account
async fn create_mint_account(
    program_test: &mut ProgramTest,
    token_program: &Pubkey,
    mint_authority: &Pubkey,
) -> Pubkey {
    let mint_account = Keypair::new();
    program_test.add_account(
        mint_account.pubkey(),
        Account {
            lamports: 1_000_000_000,
            data: vec![0; Mint::LEN],
            owner: *token_program,
            ..Account::default()
        },
    );
    mint_account.pubkey()
}

// Unit test for `create_mint` instruction
#[tokio::test]
async fn test_create_mint() {
    // Set up a test environment using `ProgramTest`
    let mut program_test = ProgramTest::new(
        "test_lrt",
        test_lrt::ID,
        processor!(test_lrt::test_lrt::handler),
    );

    // Create and initialize the mint account
    let mint_pubkey = create_mint_account(&mut program_test, &token::ID, &ADMIN_PUBKEY).await;

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create metadata for the mint using the CreateMint context
    let metadata_pubkey = Pubkey::new_unique();
    let uri = "https://example.com/metadata".to_string();
    let name = "evSOL Token".to_string();
    let symbol = "evSOL".to_string();

    let accounts = test_lrt::CreateMint {
        admin: payer.clone(),
        evsol_mint: mint_pubkey,
        metadata_account: metadata_pubkey,
        token_program: token::ID,
        token_metadata_program: mpl_token_metadata::ID,
        system_program: solana_sdk::system_program::ID,
        rent: solana_sdk::sysvar::rent::ID,
    };

    let ix = Instruction::new_with_bincode(
        test_lrt::ID,
        &test_lrt::instruction::CreateMint {
            uri,
            name,
            symbol,
        },
        accounts.to_account_infos(),
    );

    let mut transaction = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    // Submit transaction and assert it executes correctly
    assert!(banks_client.process_transaction(transaction).await.is_ok());

    // Retrieve the created metadata account and assert metadata correctness
    let metadata_account = banks_client.get_account(metadata_pubkey).await.unwrap().unwrap();
    let metadata = Metadata::unpack(&metadata_account.data).unwrap();

    assert_eq!(metadata.data.name, "evSOL Token");
    assert_eq!(metadata.data.symbol, "evSOL");
    assert_eq!(metadata.data.uri, "https://example.com/metadata");
}

// Unit test for `deposit` instruction
#[tokio::test]
async fn test_deposit() {
    // Set up a test environment using `ProgramTest`
    let mut program_test = ProgramTest::new(
        "test_lrt",
        test_lrt::ID,
        processor!(test_lrt::test_lrt::handler),
    );

    // Create evSOL mint account
    let evsol_mint = create_mint_account(&mut program_test, &token::ID, &ADMIN_PUBKEY).await;

    // Create token accounts for deposit and mint_to
    let depositor = Keypair::new();
    let deposit_account = Keypair::new();
    let mint_to_account = Keypair::new();

    // Create an account for the evSOL tokens
    let deposit_to_account = Keypair::new();

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let deposit_amount: u64 = 100;

    // Initialize the deposit context
    let accounts = test_lrt::Deposit {
        mint_to: deposit_to_account.pubkey(),
        deposit_from: deposit_account.pubkey(),
        depositor_signer: depositor.pubkey(),
        deposit_to: deposit_to_account.pubkey(),
        evsol_mint: evsol_mint,
        token_program: token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        system_program: solana_sdk::system_program::ID,
    };

    let ix = Instruction::new_with_bincode(
        test_lrt::ID,
        &test_lrt::instruction::Deposit {
            amount: deposit_amount,
        },
        accounts.to_account_infos(),
    );

    let mut transaction = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &depositor], recent_blockhash);

    // Submit transaction and assert it executes correctly
    assert!(banks_client.process_transaction(transaction).await.is_ok());

    // Verify token transfer and minting to the `mint_to` account
    let depositor_account_data = banks_client.get_account(mint_to_account.pubkey()).await.unwrap();
    let mint_to_data = TokenAccount::unpack(&depositor_account_data.unwrap().data).unwrap();

    assert_eq!(mint_to_data.amount, deposit_amount);
}