#![cfg(feature = "test-bpf")]

use {
  ::Vault::{entrypoint::process_instruction, id, instruction::VaultInstruction, state::Vault},
  assert_matches::*,
  solana_program::{
    borsh::get_packed_len,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    msg,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::{self},
  },
  solana_program_test::{processor, BanksClient, ProgramTest, ProgramTestContext},
  solana_sdk::signature::Keypair,
  solana_sdk::{account::Account, signature::Signer, transaction::Transaction},
  spl_token::{processor::Processor, state::AccountState},
  std::str::FromStr,
};

/// Tests a simple hodl vault
/// Based on Record functional test: https://github.com/solana-labs/solana-program-library/blob/2b3f71ead5b81f4ea4a2fd3e4fe9583a6e39b6a4/record/program/tests/functional.rs
#[tokio::test]
async fn test_hodl_vault() {
  // Start the test client
  let mut program_test = ProgramTest::new(
    "token_test",
    spl_token::id(),
    processor!(Processor::process),
  );
  program_test.add_program(
    "vault_test",
    ::Vault::id(),
    processor!(::Vault::processor::Processor::process),
  );
  let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

  // A basic Vault has 3 relevant tokens: X (underlying asset), lX (strategy derivative), llX (vault
  // derivative). We roughly need a client-managed & vault-managed SPL token account per-token.
  // For succintnesss, we set all of these up together:
  let mint_client_vault_accounts = (1..4)
    .map(|_| (Keypair::new(), Keypair::new(), Keypair::new()))
    .collect::<Vec<_>>();
  // Mint our various tokens & setup accounts.
  for (mint, client_account, vault_account) in mint_client_vault_accounts.iter() {
    let mut instructions = Vec::with_capacity(6);
    instructions.push(system_instruction::create_account(
      &payer.pubkey(),
      &mint.pubkey(),
      1.max(Rent::default().minimum_balance(spl_token::state::Mint::LEN)),
      spl_token::state::Mint::LEN as u64,
      &spl_token::id(),
    ));
    instructions.push(
      spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint.pubkey(),
        &payer.pubkey(),
        None, // Freeze authority
        6,
      )
      .unwrap(),
    );
    for token_account in [&client_account, &vault_account].iter() {
      instructions.push(system_instruction::create_account(
        &payer.pubkey(),
        &token_account.pubkey(),
        1.max(Rent::default().minimum_balance(spl_token::state::Account::LEN)),
        spl_token::state::Account::LEN as u64,
        &spl_token::id(),
      ));
      instructions.push(
        spl_token::instruction::initialize_account(
          &spl_token::id(),
          &token_account.pubkey(),
          &mint.pubkey(),
          &payer.pubkey(),
        )
        .unwrap(),
      );
    }
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
    // Note: We can't sign with too many signatures, hence doing multiple transactions.
    transaction.sign(
      &[&payer, &mint, &client_account, &vault_account],
      recent_blockhash,
    );
    // Create mint & initialize accounts.
    assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
  }

  // Create Vault account
  let vault_storage_account = Keypair::new();
  let mut transaction = Transaction::new_with_payer(
    &[
      // Create Vault storage acccount.
      system_instruction::create_account(
        &payer.pubkey(),
        &vault_storage_account.pubkey(),
        1.max(Rent::default().minimum_balance(::Vault::state::Vault::LEN)),
        ::Vault::state::Vault::LEN as u64,
        &::Vault::id(),
      ),
      // Initialize the vault & setup its storage account.
      VaultInstruction::initialize_vault(
        &::Vault::id(),
        &payer.pubkey(),
        &vault_storage_account.pubkey(),
        &mint_client_vault_accounts[1].2.pubkey(), // vault_lx_token account
        &mint_client_vault_accounts[2].0.pubkey(), // llx mint account
        &spl_token::id(),
        &::Vault::id(),                                          // Strategy program ID
        true,                                                    // hodl
        COption::Some(mint_client_vault_accounts[0].2.pubkey()), // vault_lx_token account
        99,                                                      // unused deposit inst. ID
        99,                                                      // unused withdraw inst. ID
      )
      .unwrap(),
    ],
    Some(&payer.pubkey()),
  );
  transaction.sign(&[&payer, &vault_storage_account], recent_blockhash);
  assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));

  // Transact with hodl vault.
  let (pda, bump_seed) = Pubkey::find_program_address(&[b"vault"], &::Vault::id());
  let mut transaction = Transaction::new_with_payer(
    &[
      // Generate a bunch of X tokens and send them to the appropriate client-managed token acct.
      spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint_client_vault_accounts[0].0.pubkey(),
        &mint_client_vault_accounts[0].1.pubkey(),
        &payer.pubkey(),
        &[&payer.pubkey()],
        1000,
      )
      .unwrap(),
      // Deposit X tokens from client account into Vault in exchange for llX tokens.
      VaultInstruction::deposit(
        &::Vault::id(),
        &spl_token::id(),
        &mint_client_vault_accounts[0].1.pubkey(), // client_x_token account
        &mint_client_vault_accounts[1].2.pubkey(), // vault_lx_token account
        vec![
          AccountMeta::new_readonly(payer.pubkey(), true), // source authority
          AccountMeta::new_readonly(vault_storage_account.pubkey(), false),
          AccountMeta::new(mint_client_vault_accounts[0].2.pubkey(), false), // hodl destination.
        ],
        100,
      )
      .unwrap(),
      // Withdraw X tokens from vault into client account in exchange for llX tokens.
      VaultInstruction::withdraw(
        &::Vault::id(),
        &spl_token::id(),
        &mint_client_vault_accounts[0].2.pubkey(), // vault_x_token account
        &mint_client_vault_accounts[0].1.pubkey(), // client_x_token account
        vec![
          AccountMeta::new_readonly(pda, false),
          AccountMeta::new_readonly(vault_storage_account.pubkey(), false),
          AccountMeta::new(mint_client_vault_accounts[0].2.pubkey(), false), // hodl destination.
        ],
        100,
      )
      .unwrap(),
    ],
    Some(&payer.pubkey()),
  );
  transaction.sign(&[&payer], recent_blockhash);
  assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));

  let (pda, _bump_seed) = Pubkey::find_program_address(&[b"vault"], &::Vault::id());
  check_token_account(&mut banks_client, &mint_client_vault_accounts[0].2.pubkey(), &pda, 0);
}

// Utils
async fn check_token_account(
  banks_client: &mut BanksClient,
  token_account_key: &Pubkey,
  expected_owner: &Pubkey,
  expected_amount: u64,
) {
  let token_account = banks_client
    .get_account(*token_account_key)
    .await
    .unwrap()
    .expect("Account unretrievable");
  assert_eq!(token_account.owner, spl_token::id());
  let internal_account = spl_token::state::Account::unpack(&token_account.data).unwrap();
  
  assert_eq!(internal_account.owner, *expected_owner);
  assert_eq!(internal_account.amount, expected_amount);
}
