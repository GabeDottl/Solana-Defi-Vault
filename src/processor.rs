use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  msg,
  program::{invoke, invoke_signed},
  program_error::ProgramError,
  program_option::COption,
  program_pack::{IsInitialized, Pack},
  pubkey::Pubkey,
  sysvar::{rent::Rent, Sysvar},
};

use crate::{error::VaultError, instruction::VaultInstruction, state::Vault};

pub struct Processor;
impl Processor {
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    msg!("Unpacking instruction");
    let instruction = VaultInstruction::unpack(instruction_data)?;

    match instruction {
      VaultInstruction::InitializeVault {
        hodl,
        strategy_program_deposit_instruction_id,
        strategy_program_withdraw_instruction_id,
      } => {
        msg!("Instruction: InitializeVault");
        Self::process_initialize_vault(
          program_id,
          accounts,
          hodl,
          strategy_program_deposit_instruction_id,
          strategy_program_withdraw_instruction_id,
        )
      }
      VaultInstruction::Deposit { amount } => {
        msg!("Instruction: Deposit");
        Self::process_transfer(program_id, accounts, amount, true)
      }
      VaultInstruction::Withdraw { amount } => {
        msg!("Instruction: Withdraw");
        Self::process_transfer(program_id, accounts, amount, false)
      }
    }
  }

  fn process_initialize_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    hodl: bool,
    strategy_program_deposit_instruction_id: u8,
    strategy_program_withdraw_instruction_id: u8,
  ) -> ProgramResult {
    msg!("Initializing vault");
    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }
    let storage_account = next_account_info(account_info_iter)?;
    let lx_token_account = next_account_info(account_info_iter)?;
    let llx_token_mint_id = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let strategy_program = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

    if *lx_token_account.owner != spl_token::id() || *llx_token_mint_id.owner != spl_token::id() {
      return Err(ProgramError::IncorrectProgramId);
    }

    if !rent.is_exempt(storage_account.lamports(), storage_account.data_len()) {
      return Err(VaultError::NotRentExempt.into());
    }

    let mut storage_info = Vault::unpack_unchecked(&storage_account.data.borrow())?;
    if storage_info.is_initialized() {
      return Err(ProgramError::AccountAlreadyInitialized);
    }

    storage_info.is_initialized = true;
    storage_info.hodl = hodl;
    storage_info.llx_token_mint_id = *llx_token_mint_id.key;
    msg!("Setting auth");
    if hodl {
      msg!("Transferring program X token ownership");
      let x_token_account = next_account_info(account_info_iter)?;
      storage_info.x_token_account = COption::Some(*x_token_account.key);
      // Transfer ownership of the temp account to this program via a derived address.
      let (pda, _bump_seed) = Pubkey::find_program_address(&[b"vault"], program_id);
      let account_owner_change_ix = spl_token::instruction::set_authority(
        token_program.key,
        x_token_account.key,
        Some(&pda),
        spl_token::instruction::AuthorityType::AccountOwner,
        initializer.key,
        &[&initializer.key],
      )?;
      invoke(
        &account_owner_change_ix,
        &[
          x_token_account.clone(),
          initializer.clone(),
          token_program.clone(),
        ],
      )?;
    }
    storage_info.strategy_program_id = *strategy_program.key;
    storage_info.strategy_program_deposit_instruction_id = strategy_program_deposit_instruction_id;
    storage_info.strategy_program_withdraw_instruction_id =
      strategy_program_withdraw_instruction_id;

    // Write the info to the actual account.
    Vault::pack(storage_info, &mut storage_account.data.borrow_mut())?;

    // Transfer ownership of the temp account to this program via a derived address.
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"vault"], program_id);
    let account_owner_change_ix = spl_token::instruction::set_authority(
      token_program.key,
      lx_token_account.key,
      Some(&pda),
      spl_token::instruction::AuthorityType::AccountOwner,
      initializer.key,
      &[&initializer.key],
    )?;

    msg!("Calling the token program to transfer X vault token account ownership");
    msg!(
      "Token program: {}. Transferring ownership {} -> {}",
      token_program.key,
      initializer.key,
      pda
    );
    invoke(
      &account_owner_change_ix,
      &[
        lx_token_account.clone(),
        initializer.clone(),
        token_program.clone(),
      ],
    )?;
    let mint_owner_change_ix = spl_token::instruction::set_authority(
      token_program.key,
      llx_token_mint_id.key,
      Some(&pda),
      spl_token::instruction::AuthorityType::MintTokens,
      initializer.key,
      &[&initializer.key],
    )?;

    msg!("Calling the token program to transfer llX token mint authority");
    msg!(
      "Token program: {}. Transferring minting control {} -> {}",
      token_program.key,
      initializer.key,
      pda
    );
    invoke(
      &mint_owner_change_ix,
      &[
        llx_token_mint_id.clone(),
        initializer.clone(),
        token_program.clone(),
      ],
    )?;
    Ok(())
  }

  fn process_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    is_deposit: bool,
  ) -> ProgramResult {
    msg!("Transferring");
    let account_info_iter = &mut accounts.iter();

    let token_program = next_account_info(account_info_iter)?;
    let source_token_account = next_account_info(account_info_iter)?;
    let target_token_account = next_account_info(account_info_iter)?;

    // Additional account metas:
    let source_authority = next_account_info(account_info_iter)?;
    let storage_account = next_account_info(account_info_iter)?;

    let storage_info = Vault::unpack_unchecked(&storage_account.data.borrow())?;
    if !storage_info.is_initialized() {
      msg!("Storage not configured!");
      return Err(VaultError::InvalidInstruction.into());
    }

    // Charge fees
    if is_deposit {
      // TODO(001): implement.
      msg!("Mint lX tokens to client account");
    } else {
      // TODO(002): implement.
      msg!("Transfer & burn lX tokens from client");
    }

    // Check if this is a HODL Vault; if so, we deposit & withdraw from 
    if storage_info.hodl {
      let x_token_account = next_account_info(account_info_iter)?;
      msg!("Calling the token program to transfer tokens");
      if is_deposit {
        let transfer_to_vault_ix = spl_token::instruction::transfer(
          token_program.key,
          source_token_account.key,
          x_token_account.key,
          &source_authority.key,
          &[&source_authority.key],
          amount,
        )?;
        msg!("Depositing to hodl account");
        invoke(
          &transfer_to_vault_ix,
          &[
            source_token_account.clone(),
            x_token_account.clone(),
            source_authority.clone(),
            token_program.clone(),
          ],
        )?;
      } else {
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"vault"], program_id);
        let transfer_to_client_ix = spl_token::instruction::transfer(
          token_program.key,
          x_token_account.key,
          target_token_account.key,
          &pda,
          &[&pda],
          amount,
        )?;
        msg!("Withdrawing from hodl account");
        invoke_signed(
          &transfer_to_client_ix,
          &[
            x_token_account.clone(),
            target_token_account.clone(),
            source_authority.clone(),
            token_program.clone(),
          ],
          &[&[&b"vault"[..], &[bump_seed]]],
        )?;
      }
    }
    else {
      if is_deposit {
        // TODO(003): implement.
        msg!("Depositing into strategy");
      } else {
        // TODO(003): implement.
        msg!("Withdrawing from strategy");
      }
    }
    Ok(())
  }
}
