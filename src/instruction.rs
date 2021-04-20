use crate::error::EscrowError::InvalidInstruction;
use solana_program::program_error::ProgramError;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};
use std::convert::TryInto;
use std::mem::size_of;

pub enum EscrowInstruction {
    /// Starts the trade by creating and populating an escrow account and transferring ownership of the given temp token account to the PDA
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the escrow
    /// 1. `[writable]` Temporary token account that should be created prior to this instruction and owned by the initializer
    /// 2. `[]` The initializer's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The escrow account, it will hold all necessary info about the trade.
    /// 4. `[]` The rent sysvar
    /// 5. `[]` The SPL token program for transfer authority.
    InitEscrow {
        /// The amount party A expects to receive of token Y
        amount: u64,
    },
}

impl EscrowInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitEscrow {
                amount: Self::unpack_amount(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }

    fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::InitEscrow { amount } => {
                buf.push(0);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
        }
        buf
    }

    pub fn initialize_escrow(
        escrow_program_id: &Pubkey,
        account: &Pubkey,
        temp_escrow_account: &Pubkey,
        receiver_account: &Pubkey,
        escrow_account: &Pubkey,
        token_program_id: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, ProgramError> {
        let accounts = vec![
            AccountMeta::new_readonly(*account, true),
            AccountMeta::new(*temp_escrow_account, false),
            AccountMeta::new_readonly(*receiver_account, false),
            AccountMeta::new(*escrow_account, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(*token_program_id, false),
        ];
        let data = EscrowInstruction::InitEscrow { amount: amount }.pack();
        Ok(Instruction {
            program_id: *escrow_program_id,
            accounts,
            data,
        })
    }
}
