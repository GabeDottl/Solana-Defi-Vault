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
  let mut program_test_context = program_test.start_with_context().await;
  // A basic Vault has 3 relevant tokens: X (underlying asset), lX (strategy derivative), llX (vault
  // derivative). We roughly need a client-managed & vault-managed SPL token account per-token.
  // For succintnesss, we set all of these up together:
  let mint_client_vault_accounts =
    create_tokens_and_accounts(&mut program_test_context, 3, 3).await;

  // Create Vault account
  let hodl_vault_storage_account = Keypair::new();
  let mut transaction = Transaction::new_with_payer(
    &[
      // Create Vault storage acccount.
      system_instruction::create_account(
        &program_test_context.payer.pubkey(),
        &hodl_vault_storage_account.pubkey(),
        1.max(Rent::default().minimum_balance(::Vault::state::Vault::LEN)),
        ::Vault::state::Vault::LEN as u64,
        &::Vault::id(),
      ),
      // Initialize the vault & setup its storage account.
      VaultInstruction::initialize_vault(
        &::Vault::id(),
        &program_test_context.payer.pubkey(),
        &hodl_vault_storage_account.pubkey(),
        &mint_client_vault_accounts[1][2].pubkey(), // vault_lx_token account
        &mint_client_vault_accounts[2][0].pubkey(), // llx mint account
        &spl_token::id(),
        &::Vault::id(), // Strategy program ID
        true,           // hodl
        COption::Some(mint_client_vault_accounts[0][2].pubkey()), // vault_lx_token account
        99,             // unused deposit inst. ID
        99,             // unused withdraw inst. ID
      )
      .unwrap(),
    ],
    Some(&program_test_context.payer.pubkey()),
  );
  transaction.sign(
    &[&program_test_context.payer, &hodl_vault_storage_account],
    program_test_context.last_blockhash,
  );
  assert_matches!(
    program_test_context
      .banks_client
      .process_transaction(transaction)
      .await,
    Ok(())
  );

  // Transact with hodl vault.
  let (pda, bump_seed) = Pubkey::find_program_address(&[b"vault"], &::Vault::id());
  let mut transaction = Transaction::new_with_payer(
    &[
      // Generate a bunch of X tokens and send them to the appropriate client-managed token acct.
      spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint_client_vault_accounts[0][0].pubkey(),
        &mint_client_vault_accounts[0][1].pubkey(),
        &program_test_context.payer.pubkey(),
        &[&program_test_context.payer.pubkey()],
        1000,
      )
      .unwrap(),
      // Deposit X tokens from client account into Vault in exchange for llX tokens.
      VaultInstruction::deposit(
        &::Vault::id(),
        &spl_token::id(),
        &mint_client_vault_accounts[0][1].pubkey(), // client_x_token account
        &mint_client_vault_accounts[1][2].pubkey(), // vault_lx_token account
        vec![
          AccountMeta::new_readonly(program_test_context.payer.pubkey(), true), // source authority
          AccountMeta::new_readonly(hodl_vault_storage_account.pubkey(), false),
          AccountMeta::new(mint_client_vault_accounts[0][2].pubkey(), false), // hodl destination.
        ],
        100,
      )
      .unwrap(),
    ],
    Some(&program_test_context.payer.pubkey()),
  );
  transaction.sign(
    &[&program_test_context.payer],
    program_test_context.last_blockhash,
  );
  assert_matches!(
    program_test_context
      .banks_client
      .process_transaction(transaction)
      .await,
    Ok(())
  );
  // Ensure accounts have correct balances.
  // Due to Rust semantics limitations around borrowing, we don't pass an expected owner.
  check_token_account(
    &mut program_test_context,
    &mint_client_vault_accounts[0][1].pubkey(),
    &COption::None,
    900,
  )
  .await;
  check_token_account(
    &mut program_test_context,
    &mint_client_vault_accounts[0][2].pubkey(),
    &COption::Some(pda),
    100,
  )
  .await;
  check_token_account(
    &mut program_test_context,
    &mint_client_vault_accounts[1][2].pubkey(),
    &COption::Some(pda),
    0,
  )
  .await;

  let mut transaction = Transaction::new_with_payer(
    &[
      // Withdraw X tokens from vault into client account in exchange for llX tokens.
      VaultInstruction::withdraw(
        &::Vault::id(),
        &spl_token::id(),
        &mint_client_vault_accounts[0][2].pubkey(), // vault_x_token account
        &mint_client_vault_accounts[0][1].pubkey(), // client_x_token account
        vec![
          AccountMeta::new_readonly(pda, false),
          AccountMeta::new_readonly(hodl_vault_storage_account.pubkey(), false),
          AccountMeta::new(mint_client_vault_accounts[0][2].pubkey(), false), // hodl destination.
        ],
        100,
      )
      .unwrap(),
    ],
    Some(&program_test_context.payer.pubkey()),
  );
  transaction.sign(
    &[&program_test_context.payer],
    program_test_context.last_blockhash,
  );
  assert_matches!(
    program_test_context
      .banks_client
      .process_transaction(transaction)
      .await,
    Ok(())
  );
  check_token_account(
    &mut program_test_context,
    &mint_client_vault_accounts[0][1].pubkey(),
    &COption::None,
    1000,
  )
  .await;
  check_token_account(
    &mut program_test_context,
    &mint_client_vault_accounts[0][2].pubkey(),
    &COption::Some(pda),
    0,
  )
  .await;

  // Create wrapper vault which uses the hodl vault as a Strategy.
  // TODO(004): Uncomment below.
  // let wrapper_vault_storage_account = Keypair::new();
  // let mut transaction = Transaction::new_with_payer(
  //   &[
  //     // Create Vault storage acccount.
  //     system_instruction::create_account(
  //       &program_test_context.payer.pubkey(),
  //       &wrapper_vault_storage_account.pubkey(),
  //       1.max(Rent::default().minimum_balance(::Vault::state::Vault::LEN)),
  //       ::Vault::state::Vault::LEN as u64,
  //       &::Vault::id(),
  //     ),
  //     // Initialize the vault & setup its storage account.
  //     VaultInstruction::initialize_vault(
  //       &::Vault::id(),
  //       &program_test_context.payer.pubkey(),
  //       &wrapper_vault_storage_account.pubkey(),
  //       &mint_client_vault_accounts[1][2].pubkey(), // vault_lx_token account
  //       &mint_client_vault_accounts[2][0].pubkey(), // llx mint account
  //       &spl_token::id(),
  //       &::Vault::id(), // Strategy program ID
  //       false,          // hodl
  //       COption::None,  // Unused vault_lx_token account
  //       2,              // deposit inst. ID
  //       3,              // withdraw inst. ID
  //     )
  //     .unwrap(),
  //   ],
  //   Some(&program_test_context.payer.pubkey()),
  // );
  // transaction.sign(
  //   &[&program_test_context.payer, &wrapper_vault_storage_account],
  //   program_test_context.last_blockhash,
  // );
  // assert_matches!(
  //   program_test_context
  //     .banks_client
  //     .process_transaction(transaction)
  //     .await,
  //   Ok(())
  // );
}

/// Checks for expected values on a token account.
async fn check_token_account(
  program_test_context: &mut ProgramTestContext,
  token_account_key: &Pubkey,
  expected_owner: &COption<Pubkey>,
  expected_amount: u64,
) {
  let token_account = program_test_context
    .banks_client
    .get_account(*token_account_key)
    .await
    .unwrap()
    .expect("Account unretrievable");
  assert_eq!(token_account.owner, spl_token::id());
  let internal_account = spl_token::state::Account::unpack(&token_account.data).unwrap();
  if expected_owner.is_some() {
    assert_eq!(internal_account.owner, expected_owner.unwrap());
  }
  assert_eq!(internal_account.amount, expected_amount);
}

/// Generates tokens & token-accounts to hold them in the specified numbers.
///
/// Returns a Vec matrix in which each row corresponds to a single token, the first value in the
/// row is the mint account, and the remaining values are token accounts.
async fn create_tokens_and_accounts(
  program_test_context: &mut ProgramTestContext,
  num_tokens: u64,
  num_accounts: u64,
) -> Vec<Vec<Keypair>> {
  let mint_client_vault_accounts = (1..(num_tokens + 1))
    .map(|_| {
      (1..(num_accounts + 2))
        .map(|_| Keypair::new())
        .collect::<Vec<Keypair>>()
    })
    .collect::<Vec<Vec<Keypair>>>();

  // Mint our various tokens & setup accounts.
  for accounts in mint_client_vault_accounts.iter() {
    let mut instructions = Vec::with_capacity(2);
    let mint = &accounts[0]; // First account is always mint
    instructions.push(system_instruction::create_account(
      &program_test_context.payer.pubkey(),
      &mint.pubkey(),
      1.max(Rent::default().minimum_balance(spl_token::state::Mint::LEN)),
      spl_token::state::Mint::LEN as u64,
      &spl_token::id(),
    ));
    instructions.push(
      spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint.pubkey(),
        &program_test_context.payer.pubkey(),
        None, // Freeze authority
        6,    // decimals
      )
      .unwrap(),
    );
    let mut transaction =
      Transaction::new_with_payer(&instructions, Some(&program_test_context.payer.pubkey()));
    transaction.sign(
      &[&program_test_context.payer, &mint],
      program_test_context.last_blockhash,
    );
    assert_matches!(
      program_test_context
        .banks_client
        .process_transaction(transaction)
        .await,
      Ok(())
    );

    for token_account in accounts[1..].iter() {
      let mut instructions = Vec::with_capacity(2);
      instructions.push(system_instruction::create_account(
        &program_test_context.payer.pubkey(),
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
          &program_test_context.payer.pubkey(),
        )
        .unwrap(),
      );
      // Note: We can only sign with so many signatures at once, so we need to split transactions
      // up quite a
      let mut transaction =
        Transaction::new_with_payer(&instructions, Some(&program_test_context.payer.pubkey()));
      transaction.sign(
        &[&program_test_context.payer, &token_account],
        program_test_context.last_blockhash,
      );
      assert_matches!(
        program_test_context
          .banks_client
          .process_transaction(transaction)
          .await,
        Ok(())
      );
    }
  }
  return mint_client_vault_accounts;
}
