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


use crate::{
  error::{VaultError},
  instruction::{VaultInstruction},
  state::{
    Vault
  },
};

pub struct Processor;
impl Processor {
  // Vault Process
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = VaultInstruction::unpack(instruction_data)?;

    match instruction {
      VaultInstruction::ConfigureVault {
        strategy_program_deposit_instruction_id,
        strategy_program_withdraw_instruction_id
      } => {
        msg!("Instruction: ConfigureVault");
        Self::process_configure_vault(
          program_id,
          accounts,
          strategy_program_deposit_instruction_id,
          strategy_program_withdraw_instruction_id,
        )
      }
      VaultInstruction::Deposit { amount } => {
        msg!("Instruction: Deposit");
        Self::process_transfer(program_id, accounts, amount)
      }
      VaultInstruction::Withdraw { amount } => {
        msg!("Instruction: Withdraw");
        Self::process_transfer(program_id, accounts, amount)
      }
      _ => return Err(VaultError::InvalidInstruction.into()),
    }
  }

  fn process_configure_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    strategy_program_deposit_instruction_id: u8,
    strategy_program_withdraw_instruction_id: u8,
  ) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let storage_account = next_account_info(account_info_iter)?;
    let lx_token_account = next_account_info(account_info_iter)?;
    let llx_token_mint_id = next_account_info(account_info_iter)?;
    let strategy_program = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    
    if !initializer.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    
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
    storage_info.llx_token_mint_id = *llx_token_mint_id.key;
    storage_info.strategy_program_id = *strategy_program.key;
    storage_info.strategy_program_deposit_instruction_id = strategy_program_deposit_instruction_id;
    storage_info.strategy_program_withdraw_instruction_id = strategy_program_withdraw_instruction_id;

    // Write the info to the actual account.
    Vault::pack(storage_info, &mut storage_account.data.borrow_mut())?;

    // Transfer ownership of the temp account to this program via a derived address.
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"vault"], program_id);

    let token_program = next_account_info(account_info_iter)?;
    let owner_change_ix = spl_token::instruction::set_authority(
      token_program.key,
      lx_token_account.key,
      Some(&pda),
      spl_token::instruction::AuthorityType::AccountOwner,
      initializer.key,
      &[&initializer.key],
    )?;

    msg!("Calling the token program to transfer token account ownership...");
    msg!(
      "Token program: {}. Transferring ownership {} -> {}",
      token_program.key,
      initializer.key,
      pda
    );
    invoke(
      &owner_change_ix,
      &[
        lx_token_account.clone(),
        initializer.clone(),
        token_program.clone(),
      ],
    )?;
    msg!("Called");
    Ok(())
  }

  fn process_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
  ) -> ProgramResult {
    Ok(())
  }


  // fn process_configure_vault(
  //   accounts: &[AccountInfo],
  //   strategy_program_id: &Pubkey,
  //   strategy_program_instruction_id: u8,
  //   program_id: &Pubkey,
  // ) -> ProgramResult {
  //   let account_info_iter = &mut accounts.iter();
  //   let storage_account = next_account_info(account_info_iter)?;

  //   let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
  //   if !rent.is_exempt(storage_account.lamports(), storage_account.data_len()) {
  //     return Err(VaultError::NotRentExempt.into());
  //   }

  //   let mut claim_type_info = Vault::unpack_unchecked(&storage_account.data.borrow())?;
  //   if claim_type_info.is_initialized() {
  //     return Err(ProgramError::AccountAlreadyInitialized);
  //   }
  //   claim_type_info.is_initialized = true;
  //   claim_type_info.strategy_program_id = *strategy_program_id;
  //   claim_type_info.strategy_program_instruction_id = strategy_program_instruction_id;

  //   // Write the claim_type_info to the actual account.
  //   Vault::pack(claim_type_info, &mut storage_account.data.borrow_mut())?;
  //   Ok(())
  // }
  // fn check_minter(account: &AccountInfo, program_id: &Pubkey) -> ProgramResult {
  //   msg!("Account: {}", account.key.to_string());
  //   if account.key.to_string() == GOD_PUBKEY_STR {
  //     return Ok(());
  //   }
  //   // Ensure minter account is owned by Vault program.
  //   if account.owner != program_id {
  //     msg!("Invalid owner!");
  //     return Err(VaultError::InvalidMinter.into());
  //   }
  //   // Check that minter has Minter credential.
  //   let account_info = Vault::unpack_unchecked(&account.data.borrow())?;
  //   if !account_info
  //     .verified_credentials
  //     .iter()
  //     .any(|&vc| vc.type_ == CredentialType::VaultMinter)
  //   {
  //     msg!("No Minter VC!");
  //     return Err(VaultError::InvalidMinter.into());
  //   }
  //   Ok(())
  // }

  // fn process_create_heart_token(
  //   accounts: &[AccountInfo],
  //   // heart_token_owner: Pubkey,
  //   program_id: &Pubkey,
  // ) -> ProgramResult {
  //   let account_info_iter = &mut accounts.iter();

  //   let owner = next_account_info(account_info_iter)?;
  //   if !owner.is_signer {
  //     return Err(ProgramError::MissingRequiredSignature);
  //   }

  //   let heart_token_account = next_account_info(account_info_iter)?;
  //   let heart_token_minter = next_account_info(account_info_iter)?;
  //   msg!("pub key {}", *heart_token_minter.key);

  //   Processor::check_minter(heart_token_minter, program_id)?;

  //   let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
  //   if !rent.is_exempt(
  //     heart_token_account.lamports(),
  //     heart_token_account.data_len(),
  //   ) {
  //     return Err(VaultError::NotRentExempt.into());
  //   }

  //   let mut heart_token_info = Vault::unpack_unchecked(&heart_token_account.data.borrow())?;
  //   if heart_token_info.is_initialized() {
  //     return Err(ProgramError::AccountAlreadyInitialized);
  //   }
  //   heart_token_info.is_initialized = true;
  //   heart_token_info.owner_pubkey = *owner.key;

  //   // Write the heart_token info to the actual account.
  //   Vault::pack(heart_token_info, &mut heart_token_account.data.borrow_mut())?;

  //   Ok(())
  // }


  // fn process_issue_claim(
  //   accounts: &[AccountInfo],
  //   claim_type_id: &Pubkey,
  //   subject_heart_token_id: &Pubkey,
  //   program_id: &Pubkey,
  // ) -> ProgramResult {
  //   let account_info_iter = &mut accounts.iter();
  //   let storage_account = next_account_info(account_info_iter)?;
  //   // Ensure rent is paid for the claim-storage.
  //   let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
  //   if !rent.is_exempt(storage_account.lamports(), storage_account.data_len()) {
  //     return Err(VaultError::NotRentExempt.into());
  //   }
  //   let claim_type_account = next_account_info(account_info_iter)?;
  //   let claim_strategy_program = next_account_info(account_info_iter)?;

  //   // Ensure not a fake claim owned by another program.
  //   if claim_type_account.owner != program_id {
  //     return Err(ProgramError::InvalidAccountData);
  //   }
  //   let claim_type_info = Vault::unpack_unchecked(&claim_type_account.data.borrow())?;
  //   // Ensure specified check program corresponds to the actual required check program.
  //   if claim_type_info.strategy_program_id != *claim_strategy_program.key {
  //     return Err(ProgramError::InvalidArgument);
  //   }
  //   let mut account_metas = Vec::new();
  //   account_metas.push(AccountMeta::new_readonly(*claim_strategy_program.key, claim_strategy_program.is_signer));
  //   for account in account_info_iter {
  //     account_metas.push(if account.is_writable {
  //       AccountMeta::new(*account.key, account.is_signer)
  //     } else {
  //       AccountMeta::new_readonly(*account.key, account.is_signer)
  //     })
  //   }
  //   let mut data = Vec::new();
  //   data.push(claim_type_info.strategy_program_instruction_id);
  //   let instruction = Instruction {
  //     program_id: claim_type_info.strategy_program_id,
  //     accounts: account_metas,
  //     data,
  //   };
  //   // Run the claim-check program - if it fails, then the user cannot be granted the credential.
  //   // First account in 
  //   invoke(&instruction, &accounts[3..])?;
  //   msg!("Called");

  //   let mut claim_info = Claim::unpack_unchecked(&storage_account.data.borrow())?;
  //   if claim_info.is_initialized() {
  //     return Err(ProgramError::AccountAlreadyInitialized);
  //   }
  //   claim_info.is_initialized = true;
  //   claim_info.claim_type_id = *claim_type_id;
  //   claim_info.subject_heart_token_id = *subject_heart_token_id;

  //   // Write the claim_info to the actual account.
  //   Claim::pack(claim_info, &mut storage_account.data.borrow_mut())?;
  //   Ok(())
  // }

  // fn process_create_simple_claim_check(
  //   accounts: &[AccountInfo],
  //   subject_required_credentials: &[Pubkey],
  //   issuer_required_credentials: &[Pubkey],
  //   program_id: &Pubkey,
  // ) -> ProgramResult {
  //   msg!("Creating claim1");
  //   let account_info_iter = &mut accounts.iter();
  //   msg!("Creating claim1");
  //   let storage_account = next_account_info(account_info_iter)?;
  //   msg!("Creating claim1");
  //   let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
  //   if !rent.is_exempt(storage_account.lamports(), SimpleClaimCheck::LEN) {
  //     return Err(ProgramError::AccountNotRentExempt);
  //   }
  //   msg!("Creating claim1");

  //   let mut claim_check_info = SimpleClaimCheck::unpack_unchecked(&storage_account.data.borrow())?;
  //   if claim_check_info.is_initialized() {
  //     return Err(ProgramError::AccountAlreadyInitialized);
  //   }
  //   msg!("Creating claim2");
  //   claim_check_info.is_initialized = true;
  //   if subject_required_credentials.len() > MAX_REQUIRED_CREDENTIALS
  //     || issuer_required_credentials.len() > MAX_REQUIRED_CREDENTIALS
  //   {
  //     return Err(ProgramError::InvalidArgument);
  //   }

  //   msg!("Creating claim3");
  //   claim_check_info
  //     .issuer_required_credentials
  //     .copy_from_slice(issuer_required_credentials);
  //   claim_check_info
  //     .subject_required_credentials
  //     .copy_from_slice(subject_required_credentials);

  //   // Write the heart_token info to the actual account.
  //   SimpleClaimCheck::pack(claim_check_info, &mut storage_account.data.borrow_mut())?;

  //   Ok(())
  // }

  // fn process_execute_simple_claim_check(
  //   accounts: &[AccountInfo],
  //   program_id: &Pubkey,
  // ) -> ProgramResult {
  //   let account_info_iter = &mut accounts.iter();
  //   let storage_account = next_account_info(account_info_iter)?;

  //   // let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
  //   // if !rent.is_exempt(storage_account.lamports(), storage_account.data_len()) {
  //   //   return Err(VaultError::NotRentExempt.into());
  //   // }

  //   let simple_claim_check = SimpleClaimCheck::unpack_unchecked(&storage_account.data.borrow())?;
  //   if !simple_claim_check.is_initialized() {
  //     return Err(VaultError::InvalidInstruction.into());
  //   }
  //   let subject_heart_token = next_account_info(account_info_iter)?;
  //   let issuer_heart_token = next_account_info(account_info_iter)?;
  //   let mut passed: bool = true;
  //   for claim_type_id in simple_claim_check.subject_required_credentials.iter() {
  //     if *claim_type_id == NULL_PUBKEY {
  //       break;
  //     }
  //     // If the claim type is the target HT, then ignore other credential requirements.
  //     if claim_type_id == subject_heart_token.key {
  //       passed = true;
  //       break;
  //     }
  //     let account_claim = next_account_info(account_info_iter).unwrap();
  //     // Ensure not a fake claim owned by another program.
  //     if account_claim.owner != program_id {
  //       passed = false;
  //     }
  //     let account_claim_info = Claim::unpack_unchecked(&account_claim.data.borrow())?;
  //     if account_claim_info.claim_type_id != *claim_type_id
  //       || account_claim_info.subject_heart_token_id != *subject_heart_token.key
  //     {
  //       passed = false;
  //     }
  //   }
  //   if !passed {
  //     return Err(ProgramError::InvalidArgument);
  //   }
  //   passed = true;
  //   for claim_type_id in simple_claim_check.issuer_required_credentials.iter() {
  //     if *claim_type_id == NULL_PUBKEY {
  //       break;
  //     }
  //     // If the claim type is the target HT, then ignore other credential requirements.
  //     if *claim_type_id == *issuer_heart_token.key {
  //       passed = true;
  //       break;
  //     }
  //     let account_claim = next_account_info(account_info_iter).unwrap();
  //     // Ensure not a fake claim owned by another program.
  //     if account_claim.owner != program_id {
  //       passed = false;
  //     }
  //     let account_claim_info = Claim::unpack_unchecked(&account_claim.data.borrow())?;
  //     if account_claim_info.claim_type_id != *claim_type_id
  //       || account_claim_info.subject_heart_token_id != *issuer_heart_token.key
  //     {
  //       passed = false;
  //     }
  //   }
  //   if !passed {
  //     return Err(ProgramError::InvalidArgument);
  //   }

  //   Ok(())
  // }

  // // Escrow Process
  // pub fn process(
  //   program_id: &Pubkey,
  //   accounts: &[AccountInfo],
  //   instruction_data: &[u8],
  // ) -> ProgramResult {
  //   let instruction = EscrowInstruction::unpack(instruction_data)?;

  //   match instruction {
  //     EscrowInstruction::InitEscrow { amount } => {
  //       msg!("Instruction: InitEscrow");
  //       Self::process_init_escrow(accounts, amount, program_id)
  //     }
  //   }
  // }

}
