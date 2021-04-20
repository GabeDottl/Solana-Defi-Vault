use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  msg,
  program::invoke,
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack},
  pubkey::Pubkey,
  sysvar::{rent::Rent, Sysvar},
};

use crate::{error::EscrowError, instruction::EscrowInstruction, state::Escrow};

pub struct Processor;
impl Processor {
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = EscrowInstruction::unpack(instruction_data)?;

    match instruction {
      EscrowInstruction::InitEscrow { amount } => {
        msg!("Instruction: InitEscrow");
        Self::process_init_escrow(accounts, amount, program_id)
      }
    }
  }

  fn process_init_escrow(
    accounts: &[AccountInfo],
    amount: u64,
    program_id: &Pubkey,
  ) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    
    let initializer = next_account_info(account_info_iter)?;
    if !initializer.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    // No need to check owner bc function will fail.
    let temp_token_account = next_account_info(account_info_iter)?;

    // Need to check owner to insure not entering invalid state.
    let token_to_receive_account = next_account_info(account_info_iter)?;
    if *token_to_receive_account.owner != spl_token::id() {
      return Err(ProgramError::IncorrectProgramId);
    }

    let escrow_account = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

    if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
      return Err(EscrowError::NotRentExempt.into());
    }

    let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;
    if escrow_info.is_initialized() {
      return Err(ProgramError::AccountAlreadyInitialized);
    }

    escrow_info.is_initialized = true;
    escrow_info.initializer_pubkey = *initializer.key;
    escrow_info.temp_token_account_pubkey = *temp_token_account.key;
    escrow_info.initializer_token_to_receive_account_pubkey = *token_to_receive_account.key;
    escrow_info.expected_amount = amount;

    // Write the escrow info to the actual account.
    Escrow::pack(escrow_info, &mut escrow_account.data.borrow_mut())?;

    // Transfer ownership of the temp account to this program via a derived address.
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

    let token_program = next_account_info(account_info_iter)?;
    let owner_change_ix = spl_token::instruction::set_authority(
      token_program.key,
      temp_token_account.key,
      Some(&pda),
      spl_token::instruction::AuthorityType::AccountOwner,
      initializer.key,
      &[&initializer.key],
    )?;

    msg!("Calling the token program to transfer token account ownership...");
    msg!("Token program: {}. Transferring ownership {} -> {}", token_program.key, initializer.key, pda);
    invoke(
      &owner_change_ix,
      &[
        temp_token_account.clone(),
        initializer.clone(),
        token_program.clone(),
      ],
    )?;
    msg!("Called");
    Ok(())
  }
}
