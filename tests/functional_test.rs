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
#[tokio::test]
async fn test_token() {
  // Create a SPL token
  // Create a main token account for Alice
  // Create temporary token account for Alice
  let token_program_id = Pubkey::new_unique();
  let authority = Keypair::new();
  let seed = "token";
  let account = Pubkey::create_with_seed(&authority.pubkey(), seed, &token_program_id).unwrap();
  let mut program_test = ProgramTest::new(
    "token_test",
    token_program_id,
    processor!(Processor::process),
  );

  // Start the test client
  let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

  let account_space = 1;
  let mut transaction = Transaction::new_signed_with_payer(
    &[
      system_instruction::create_account_with_seed(
        &payer.pubkey(),
        &account,
        &authority.pubkey(),
        seed,
        1.max(Rent::default().minimum_balance(account_space)),
        account_space as u64,
        &token_program_id,
      ),
      // instruction::initialize(&account.pubkey(), &authority.pubkey()),
      // instruction::write(
      //     &account.pubkey(),
      //     &authority.pubkey(),
      //     0,
      //     data.try_to_vec().unwrap(),
      // ),
    ],
    Some(&payer.pubkey()),
    &[&payer, &authority],
    recent_blockhash,
  );
  // banks_client.process_transaction(transaction).await;

  // transaction.sign(&[&payer], recent_blockhash);
  // assert_eq!(true, false);
  assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
  //   invoke(
  //     &spl_token::instruction::initialize_account(
  //         token_program_info.key,
  //         deposit_account_info.key,
  //         deposit_token_mint_info.key,
  //         authority_info.key,
  //     )
  //     .unwrap(),
  //     &[
  //         token_program_info.clone(),
  //         deposit_account_info.clone(),
  //         deposit_token_mint_info.clone(),
  //         authority_info.clone(),
  //         rent_info.clone(),
  //     ],
  // )?;

  // invoke(
  //     &spl_token::instruction::initialize_mint(
  //         &spl_token::id(),
  //         token_pass_mint_info.key,
  //         authority_info.key,
  //         None,
  //         deposit_token_mint.decimals,
  //     )
  //     .unwrap(),
  //     &[
  //         token_program_info.clone(),
  //         token_pass_mint_info.clone(),
  //         rent_info.clone(),
  //     ],
  // )?;

  // invoke(
  //     &spl_token::instruction::initialize_mint(
  //         &spl_token::id(),
  //         token_fail_mint_info.key,
  //         authority_info.key,
  //         None,
  //         deposit_token_mint.decimals,
  //     )
  //     .unwrap(),
  //     &[
  //         token_program_info.clone(),
  //         token_fail_mint_info.clone(),
  //         rent_info.clone(),
  //     ],
  // )?;

  //   let mut transaction = Transaction::new_with_payer(
  //     &[Instruction {
  //       token_program_id,
  //       accounts: vec![AccountMeta::new(payer.pubkey(), false)],
  //       data: vec![0, 2, 3],
  //     }],
  //     Some(&payer.pubkey()),
  //   );
  //   transaction.sign(&[&payer], recent_blockhash);
  //   // assert_eq!(true, false);
  //   assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
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
