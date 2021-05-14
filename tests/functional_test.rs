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
  solana_program_test::{processor, ProgramTest, ProgramTestContext},
  solana_sdk::signature::Keypair,
  solana_sdk::{account::Account, signature::Signer, transaction::Transaction},
  spl_token::{processor::Processor, state::AccountState},
  std::str::FromStr,
};

trait AddPacked {
  fn add_packable_account<T: Pack>(
    &mut self,
    pubkey: Pubkey,
    amount: u64,
    data: &T,
    owner: &Pubkey,
  );
}

impl AddPacked for ProgramTest {
  fn add_packable_account<T: Pack>(
    &mut self,
    pubkey: Pubkey,
    amount: u64,
    data: &T,
    owner: &Pubkey,
  ) {
    let mut account = Account::new(amount, T::get_packed_len(), owner);
    data.pack_into_slice(&mut account.data);
    self.add_account(pubkey, account);
  }
}

// fn program_test() -> ProgramTest {
//   ProgramTest::new("spl_record", id(), processor!(process_instruction))
// }

// #[tokio::test]
// async fn test_create_heart_token() {
//   let client_x_token_account = Keypair::new();
//   let account_heart_token = Keypair::new();
//   let mut program_test = ProgramTest::new(
//     "heart_token_test",
//     Vault::id(),
//     processor!(Vault::processor::Processor::process_heart_token),
//   );

//   // Start the test client
//   let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

//   // Create Vault.
//   let account_space = spl_token::state::Mint::LEN;
//   let mut transaction = Transaction::new_with_payer(
//     &[
//       system_instruction::create_account(
//         &payer.pubkey(),
//         &account_heart_token.pubkey(),
//         1.max(Rent::default().minimum_balance(Vault::state::Vault::LEN)),
//         Vault::state::Vault::LEN as u64,
//         &Vault::id(),
//       ),
//       VaultInstruction::create_heart_token(
//         &Vault::id(),
//         &client_x_token_account.pubkey(),
//         &account_heart_token.pubkey()
//       )
//       .unwrap(),
//     ],
//     Some(&payer.pubkey()),
//   );
//   transaction.sign(&[&payer, &client_x_token_account, &account_heart_token], recent_blockhash);
//   // Create mint:
//   assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
// }

// #[tokio::test]
// async fn test_execute_create_simple_claim_check() {
//   let mut program_test = ProgramTest::new(
//     "heart_token_test",
//     Vault::id(),
//     processor!(Vault::processor::Processor::process_heart_token),
//   );
//   let alice_account = Keypair::new();
//   // Start the test client
//   let mut program_context = program_test.start_with_context().await;
//   // Create Vault.
//   let account_space = spl_token::state::Mint::LEN;
//   let claim_check_account =
//     create_simple_claim_check_transaction(&mut program_context, alice_account.pubkey());
//   let mut transaction = Transaction::new_with_payer(
//     &[
//       system_instruction::create_account(
//         &program_context.payer.pubkey(),
//         &storage_account.pubkey(),
//         1.max(Rent::default().minimum_balance(Vault::state::SimpleClaimCheck::LEN)),
//         Vault::state::SimpleClaimCheck::LEN as u64,
//         &Vault::id(),
//       ),
//       VaultInstruction::create_execute_simple_claim_check(
//         &Vault::id(),
//         [
//           AccountMeta::new_readonly(storage_account.pubkey(), false),
//           AccountMeta::new_readonly(alice_account.pubkey(), false), // Subject
//           AccountMeta::new_readonly(alice_account.pubkey(), true),  // Issuer
//           AccountMeta::new_readonly(alice_account.pubkey(), false), // Issuer credential
//         ],
//       )
//       .unwrap(),
//     ],
//     Some(&program_context.payer.pubkey()),
//   );
//   transaction.sign(
//     &[&program_context.payer, &storage_account, &alice_account],
//     program_context.last_blockhash,
//   );
//   // Create mint:
//   assert_matches!(
//     program_context
//       .banks_client
//       .process_transaction(transaction)
//       .await,
//     Ok(())
//   );
// }

/// Returns the SimpleClaimCheck-storing account.
// async fn create_simple_claim_check_transaction(
//   program_context: &mut ProgramTestContext,
//   issuer: Pubkey,
// ) -> Keypair {
//   let storage_account = Keypair::new();
//   let subject_required_credentials = [NULL_PUBKEY; MAX_REQUIRED_CREDENTIALS];
//   let mut issuer_required_credentials = [NULL_PUBKEY; MAX_REQUIRED_CREDENTIALS];
//   issuer_required_credentials[0] = issuer;

//   let mut transaction = Transaction::new_with_payer(
//     &[
//       system_instruction::create_account(
//         &program_context.payer.pubkey(),
//         &storage_account.pubkey(),
//         1.max(Rent::default().minimum_balance(Vault::state::SimpleClaimCheck::LEN)),
//         Vault::state::SimpleClaimCheck::LEN as u64,
//         &Vault::id(),
//       ),
//       VaultInstruction::create_simple_claim_check(
//         &Vault::id(),
//         &storage_account.pubkey(),
//         &subject_required_credentials,
//         &issuer_required_credentials,
//       )
//       .unwrap(),
//     ],
//     Some(&program_context.payer.pubkey()),
//   );
//   transaction.sign(
//     &[&program_context.payer, &storage_account],
//     program_context.last_blockhash,
//   );
//   // Create mint:
//   assert_matches!(
//     program_context
//       .banks_client
//       .process_transaction(transaction)
//       .await,
//     Ok(())
//   );
//   return storage_account;
// }

// #[tokio::test]
// async fn test_create_simple_claim_check() {
//   let mut program_test = ProgramTest::new(
//     "heart_token_test",
//     Vault::id(),
//     processor!(Vault::processor::Processor::process_heart_token),
//   );

//   // Start the test client
//   let mut program_context = program_test.start_with_context().await;
//   // Create Vault.
//   let account_space = spl_token::state::Mint::LEN;
//   create_simple_claim_check_transaction(&mut program_context, NULL_PUBKEY);
// }

// #[tokio::test]
// async fn test_create_heart_minter() {
//   let client_x_token_account = Keypair::new();
//   let account_heart_token = Keypair::new();
//   // let heart_token_minter = Keypair::new();
//   let keypair: [u8; 64] = [
//     107, 254, 121, 199, 233, 104, 91, 98, 219, 230, 11, 238, 73, 88, 242, 134, 198, 227, 13, 235,
//     0, 64, 96, 208, 124, 152, 133, 96, 65, 88, 149, 96, 68, 150, 109, 75, 78, 72, 134, 74, 26, 54,
//     152, 10, 233, 15, 48, 202, 174, 83, 206, 230, 45, 171, 29, 138, 3, 221, 137, 56, 228, 100, 153,
//     203,
//   ];
//   let heart_token_minter = Keypair::from_bytes(&keypair).unwrap();
//   let mut program_test = ProgramTest::new(
//     "heart_token_test",
//     Vault::id(),
//     processor!(Vault::processor::Processor::process_heart_token),
//   );

//   // Start the test client
//   let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
//   // Create Vault.
//   let account_space = spl_token::state::Mint::LEN;
//   let mut transaction = Transaction::new_with_payer(
//     &[
//       system_instruction::create_account(
//         &payer.pubkey(),
//         &account_heart_token.pubkey(),
//         1.max(Rent::default().minimum_balance(Vault::state::Vault::LEN)),
//         Vault::state::Vault::LEN as u64,
//         &Vault::id(),
//       ),
//       VaultInstruction::create_heart_token(
//         &Vault::id(),
//         &client_x_token_account.pubkey(),
//         &account_heart_token.pubkey(),
//         &heart_token_minter.pubkey(),
//       )
//       .unwrap(),
//     ],
//     Some(&payer.pubkey()),
//   );
//   transaction.sign(
//     &[
//       &payer,
//       &client_x_token_account,
//       &account_heart_token,
//       &heart_token_minter,
//     ],
//     recent_blockhash,
//   );
//   // Create mint:
//   assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
// }

// Based on Record functional test: https://github.com/solana-labs/solana-program-library/blob/2b3f71ead5b81f4ea4a2fd3e4fe9583a6e39b6a4/record/program/tests/functional.rs
// Unisqap example test https://github.com/dzmitry-lahoda/solana-uniswap-example/blob/a8f108adefe8fa61a947d408a5ce0064b1d8c2df/tests/tests.rs
#[tokio::test]
async fn test_hodl_vault() {
  // Create a SPL token
  // Create a main token account for Alice
  // Create temporary token account for Alice
  // let Vault::id() = Pubkey::new_unique();
  // TODO: Make authority derived from program?
  // let authority = Keypair::new();
  let seed = "token";
  // X, lX, llX mints, client accounts, and vault accounts. We don't need all of these, but succint.
  let mint_client_vault_accounts = (1..4)
    .map(|_| (Keypair::new(), Keypair::new(), Keypair::new()))
    .collect::<Vec<_>>();
  let vault_storage_account = Keypair::new();
  // let mint_x = Keypair::new();
  // let mint_lx = Keypair::new();
  // let mint_llx = Keypair::new();
  // let client_x_token_account = Keypair::new();
  // let client_llx_token_account = Keypair::new();
  // let vault_x_token_account = Keypair::new();
  // let vault_lx_token_account = Keypair::new();

  // let account = Pubkey::create_with_seed(&authority.pubkey(), seed, &spl_token::id()).unwrap();
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

  // Start the test client
  let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

  // Mint our tokens & setup accounts
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
        1,                                                       // deposit instruction ID
        2,                                                       // withdraw instruction ID
      )
      .unwrap(),
    ],
    Some(&payer.pubkey()),
  );
  transaction.sign(&[&payer, &vault_storage_account], recent_blockhash);
  // Create Alice's account with 1000 $A & temp-account for vault.
  assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));

  // Transact with hodl vault.
  let (pda, bump_seed) = Pubkey::find_program_address(&[b"vault"], &::Vault::id());
  let mut transaction = Transaction::new_with_payer(
    &[
      spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint_client_vault_accounts[0].0.pubkey(),
        &mint_client_vault_accounts[0].1.pubkey(),
        &payer.pubkey(),
        &[&payer.pubkey()],
        1000,
      )
      .unwrap(),
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
  //  &vault_storage_account,
  // &mint_client_vault_accounts[0].0, &mint_client_vault_accounts[0].1
  transaction.sign(&[&payer], recent_blockhash);
  // Create Alice's account with 1000 $A & temp-account for vault.
  assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));

  // Verify some data on Alice's temp account for sanity checking & fun.
  // let alice_account_temp_account = banks_client
  //   .get_account(vault_lx_token_account.pubkey())
  //   .await
  //   .unwrap()
  //   .expect("Account unretrievable");
  // assert_eq!(alice_account_temp_account.owner, spl_token::id());
  // let internal_account =
  //   spl_token::state::Account::unpack(&alice_account_temp_account.data).unwrap();
  // let (pda, _bump_seed) = Pubkey::find_program_address(&[b"vault"], &Vault::id());

  // // Ensure that the vault account's ownership
  // assert_eq!(internal_account.owner, pda);
}

// // Create accounts for holding coins.
// let mut transaction = Transaction::new_with_payer(
//   &[
//     // Create Alice's account & transfer 1000 $A.
//     system_instruction::create_account(
//       &payer.pubkey(),
//       &client_x_token_account.pubkey(),
//       1.max(Rent::default().minimum_balance(spl_token::state::Account::LEN)),
//       spl_token::state::Account::LEN as u64,
//       &spl_token::id(),
//     ),
//     spl_token::instruction::initialize_account(
//       &spl_token::id(),
//       &client_x_token_account.pubkey(),
//       &mint_x.pubkey(),
//       &authority.pubkey(),
//     )
//     .unwrap(),
//     spl_token::instruction::mint_to(
//       &spl_token::id(),
//       &mint_x.pubkey(),
//       &client_x_token_account.pubkey(),
//       &payer.pubkey(),
//       &[&payer.pubkey()],
//       1000,
//     )
//     .unwrap(),
//     // Create Alice's temp account.
//     system_instruction::create_account(
//       &payer.pubkey(),
//       &vault_lx_token_account.pubkey(),
//       1.max(Rent::default().minimum_balance(spl_token::state::Account::LEN)),
//       spl_token::state::Account::LEN as u64,
//       &spl_token::id(),
//     ),
//     spl_token::instruction::initialize_account(
//       &spl_token::id(),
//       &vault_lx_token_account.pubkey(),
//       &mint_x.pubkey(),
//       &client_x_token_account.pubkey(),
//     )
//     .unwrap(),
//   ],
//   Some(&payer.pubkey()),
// );

// transaction.sign(
//   &[&payer, &client_x_token_account, &vault_lx_token_account],
//   recent_blockhash,
// );
// // Create Alice's account with 1000 $A & temp-account for escrow.
// assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));

// // Transfer 100 from Alice's account to her temp.
// let mut transaction = Transaction::new_with_payer(
//   &[spl_token::instruction::transfer(
//     &spl_token::id(),
//     &client_x_token_account.pubkey(),
//     &vault_lx_token_account.pubkey(),
//     &authority.pubkey(),
//     &[&&authority.pubkey()],
//     100,
//   )
//   .unwrap()],
//   Some(&payer.pubkey()),
// );
// transaction.sign(&[&payer, &authority], recent_blockhash);
// assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));

// // Verify some data on Alice's temp account for sanity checking & fun.
// let alice_account_temp_account = banks_client
//   .get_account(vault_lx_token_account.pubkey())
//   .await
//   .unwrap()
//   .expect("Account unretrievable");
// assert_eq!(alice_account_temp_account.owner, spl_token::id());
// let internal_account =
//   spl_token::state::Account::unpack(&alice_account_temp_account.data).unwrap();
// assert_eq!(internal_account.owner, client_x_token_account.pubkey());
// assert_matches!(
//   internal_account.state,
//   spl_token::state::AccountState::Initialized
// );

// // // Create Escrow account
// let mut transaction = Transaction::new_with_payer(
//   &[
//     // Create Alice's account & transfer 1000 $A.
//     system_instruction::create_account(
//       &payer.pubkey(),
//       &vault_storage_account.pubkey(),
//       1.max(Rent::default().minimum_balance(Vault::state::Escrow::LEN)),
//       Vault::state::Escrow::LEN as u64,
//       &Vault::id(),
//     ),
//     EscrowInstruction::initialize_escrow(
//       &Vault::id(),
//       &client_x_token_account.pubkey(),
//       &vault_lx_token_account.pubkey(),
//       &client_x_token_account.pubkey(), // Using Alice in lieu of Bob.
//       &vault_storage_account.pubkey(),
//       &spl_token::id(),
//       100, // amount
//     )
//     .unwrap(),
//   ],
//   Some(&payer.pubkey()),
// );
// transaction.sign(
//   &[
//     &payer,
//     &vault_storage_account,
//     &client_x_token_account,
//     // &vault_lx_token_account,
//   ],
//   recent_blockhash,
// );
// // Create Alice's account with 1000 $A & temp-account for escrow.
// assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
// // Verify some data on Alice's temp account for sanity checking & fun.
// let alice_account_temp_account = banks_client
//   .get_account(vault_lx_token_account.pubkey())
//   .await
//   .unwrap()
//   .expect("Account unretrievable");
// assert_eq!(alice_account_temp_account.owner, spl_token::id());
// let internal_account =
//   spl_token::state::Account::unpack(&alice_account_temp_account.data).unwrap();
// let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], &Vault::id());

// // Ensure that the escrow account's ownership
// assert_eq!(internal_account.owner, pda);
