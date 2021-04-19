#![cfg(feature = "test-bpf")]

use {
  assert_matches::*,
  hearttoken::entrypoint::process_instruction,
  // hearttoken::processor,
  solana_program::{
    borsh::get_packed_len,
    instruction::{AccountMeta, Instruction},
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::{self},
  },
  solana_program_test::{processor, ProgramTest},
  solana_sdk::signature::Keypair,
  solana_sdk::{account::Account, signature::Signer, transaction::Transaction},
  spl_token::{processor::Processor, state::AccountState},
  // c::state::Mint,
  std::str::FromStr,
};
pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

// #[tokio::test]
// async fn test_sysvar() {
//   let program_id = Pubkey::from_str(&"Sysvar1111111111111111111111111111111111111").unwrap();
//   let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
//     "spl_example_sysvar",
//     program_id,
//     processor!(process_instruction),
//   )
//   .start()
//   .await;

//   let mut transaction = Transaction::new_with_payer(
//     &[Instruction::new_with_bincode(
//       program_id,
//       &(),
//       vec![
//         AccountMeta::new(sysvar::clock::id(), false),
//         AccountMeta::new(sysvar::rent::id(), false),
//       ],
//     )],
//     Some(&payer.pubkey()),
//   );
//   transaction.sign(&[&payer], recent_blockhash);
//   banks_client.process_transaction(transaction).await.unwrap();
// }

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

// Based on Record functional test: https://github.com/solana-labs/solana-program-library/blob/2b3f71ead5b81f4ea4a2fd3e4fe9583a6e39b6a4/record/program/tests/functional.rs
// Unisqap example test https://github.com/dzmitry-lahoda/solana-uniswap-example/blob/a8f108adefe8fa61a947d408a5ce0064b1d8c2df/tests/tests.rs
#[tokio::test]
async fn test_token() {
  // Create a SPL token
  // Create a main token account for Alice
  // Create temporary token account for Alice
  let token_program_id = Pubkey::new_unique();
  // TODO: Make authority derived from program?
  let authority = Keypair::new();
  let seed = "token";
  let mint_a = Keypair::new();
  let account_alice = Keypair::new();
  let account_alice_temp = Keypair::new();

  // let account = Pubkey::create_with_seed(&authority.pubkey(), seed, &token_program_id).unwrap();
  let mut program_test = ProgramTest::new(
    "token_test",
    token_program_id,
    processor!(Processor::process),
  );

  // Start the test client
  let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

  // CREATE Mint A.
  let account_space = spl_token::state::Mint::LEN;
  let mut transaction = Transaction::new_with_payer(
    &[
      system_instruction::create_account(
        &payer.pubkey(),
        &mint_a.pubkey(),
        1.max(Rent::default().minimum_balance(account_space)),
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(),
      ),
      spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_a.pubkey(),
        &payer.pubkey(),
        None, // Freeze authority
        6,
      )
      .unwrap(),
    ],
    Some(&payer.pubkey()),
  );
  transaction.sign(&[&payer, &mint_a], recent_blockhash);
  // Create mint:
  assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
  let mut transaction = Transaction::new_with_payer(
    &[
      // Create Alice's account & transfer 1000 $A.
      system_instruction::create_account(
        &payer.pubkey(),
        &account_alice.pubkey(),
        1.max(Rent::default().minimum_balance(spl_token::state::Account::LEN)),
        spl_token::state::Account::LEN as u64,
        &spl_token::id(),
      ),
      spl_token::instruction::initialize_account(
        &spl_token::id(),
        &account_alice.pubkey(),
        &mint_a.pubkey(),
        &authority.pubkey(),
      )
      .unwrap(),
      spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint_a.pubkey(),
        &account_alice.pubkey(),
        &payer.pubkey(),
        &[&payer.pubkey()],
        1000,
      )
      .unwrap(),
      // Create Alice's temp account.
      system_instruction::create_account(
        &payer.pubkey(),
        &account_alice_temp.pubkey(),
        1.max(Rent::default().minimum_balance(spl_token::state::Account::LEN)),
        spl_token::state::Account::LEN as u64,
        &spl_token::id(),
      ),
      spl_token::instruction::initialize_account(
        &spl_token::id(),
        &account_alice_temp.pubkey(),
        &mint_a.pubkey(),
        &authority.pubkey(),
      )
      .unwrap(),
    ],
    Some(&payer.pubkey()),
  );

  transaction.sign(&[&payer, &account_alice, &account_alice_temp], recent_blockhash);
  assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
}

// #[tokio::test]
// https://github.com/solana-labs/solana-program-library/blob/2b3f71ead5b81f4ea4a2fd3e4fe9583a6e39b6a4/record/program/tests/functional.rs
// async fn test_escrow() {
//   let token_program_id = Pubkey::new_unique();
//   let mut program_test =
//     ProgramTest::new("token_test", escrow_program_id, processor!(process_instruction));

//   let escrow_program_id = Pubkey::new_unique();

//   // Create a SPL token
//   // Create a main token account for Alice
//   // Create temporary token account for Alice
//   // Create a receiving account for Alice
//   // Create Escrow program

//   let alice_pubkey = Pubkey::new_unique();
//   let destination_pubkey = Pubkey::new_unique();
//   // TODO: Create SPL token program & transactions?
//   // let mut program_test =
//   // ProgramTest::new("escrow_test", escrow_program_id, processor!(process_instruction));
//   // add_usdc_mint(&mut program_test);

//   // Start the test client
//   let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

//   let mut transaction = Transaction::new_with_payer(
//     &[Instruction {
//       escrow_program_id,
//       accounts: vec![AccountMeta::new(payer.pubkey(), false)],
//       data: vec![1, 2, 3],
//     }],
//     Some(&payer.pubkey()),
//   );
//   transaction.sign(&[&payer], recent_blockhash);
//   // assert_eq!(true, false);
//   assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
// }

// pub struct TestQuoteMint {
//   pub pubkey: Pubkey,
//   pub authority: Keypair,
//   pub decimals: u8,
// }

// pub fn add_usdc_mint(test: &mut ProgramTest) -> TestQuoteMint {
//   let authority = Keypair::new();
//   let pubkey = Pubkey::from_str(USDC_MINT).unwrap();
//   let decimals = 6;
//   test.add_packable_account(
//     pubkey,
//     u32::MAX as u64,
//     &Mint {
//       is_initialized: true,
//       mint_authority: COption::Some(authority.pubkey()),
//       decimals,
//       ..Mint::default()
//     },
//     &spl_token::id(),
//   );
//   TestQuoteMint {
//     pubkey,
//     authority,
//     decimals,
//   }
// }
