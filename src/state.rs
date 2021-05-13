use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vault {
  pub is_initialized: bool,
  pub llx_token_mint_id: Pubkey,
  pub strategy_program_id: Pubkey,
  pub strategy_program_deposit_instruction_id: u8,
  pub strategy_program_withdraw_instruction_id: u8,
}

impl Sealed for Vault {}

impl Pack for Vault {
  const LEN: usize = 1 + 32 + 32 + 1 + 1;
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, Vault::LEN];
    let (is_initialized, llx_token_mint_id, strategy_program_id, strategy_program_deposit_instruction_id, strategy_program_withdraw_instruction_id) =
      array_refs![src, 1, 32, 32, 1, 1];

    let is_initialized = match is_initialized {
      [0] => false,
      [1] => true,
      _ => return Err(ProgramError::InvalidAccountData),
    };
    let mut result = Vault {
      is_initialized,
      llx_token_mint_id: Pubkey::new_from_array(*llx_token_mint_id),
      strategy_program_id: Pubkey::new_from_array(*strategy_program_id),
      strategy_program_deposit_instruction_id: strategy_program_deposit_instruction_id[0],
      strategy_program_withdraw_instruction_id: strategy_program_withdraw_instruction_id[0],
    };
    Ok(result)
  }

  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, Vault::LEN];
    let (is_initialized_dst, llx_token_mint_id_dst, strategy_program_id_dst, strategy_program_deposit_instruction_id_dst, strategy_program_withdraw_instruction_id_dst) =
      mut_array_refs![dst, 1, 32, 32, 1, 1];

    let Vault {
      is_initialized,
      llx_token_mint_id,
      strategy_program_id,
      strategy_program_deposit_instruction_id,
      strategy_program_withdraw_instruction_id
    } = self;

    is_initialized_dst[0] = *is_initialized as u8;
    llx_token_mint_id_dst.copy_from_slice(llx_token_mint_id.as_ref());
    strategy_program_id_dst.copy_from_slice(strategy_program_id.as_ref());
    strategy_program_deposit_instruction_id_dst[0] = *strategy_program_deposit_instruction_id;
    strategy_program_withdraw_instruction_id_dst[0] = *strategy_program_withdraw_instruction_id;
  }
}

impl IsInitialized for Vault {
  fn is_initialized(&self) -> bool {
    self.is_initialized
  }
}
