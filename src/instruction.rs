use crate::error::{VaultError, VaultError::InvalidInstruction};
use solana_program::program_error::ProgramError;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    program_option::COption,
    sysvar,
};

use std::convert::TryInto;
use std::mem::size_of;

pub enum VaultInstruction {
    /// Creates a Vault.
    ///
    /// Vaults are designed to be highly composable and don't directly hold any of their
    /// underlying asset (X) - instead, they just hold the underlying strategy's asset (lX) and then
    /// wraps it in its own mirror asset (llX) which is returned to the user. The user can redeem
    /// llX tokens for their underlying X token (plus profits) and will be charged a fixed fee
    /// against their returned assets.
    ///
    /// The interaction with a vault looks like the following:
    ///
    /// Deposit:
    ///   User sends X to Vault, Vault sends X to the strategy and gets back an lX token, which it
    ///   stores, and then mints a corresponding llX token which it gives to the user.
    /// Withdraw:
    ///   User sends llX to Vault, Vault burns the tokens and sends the corresponding lX to the
    ///   strategy and gets back X tokens, which it forwards to the user, minus a fee.
    ///
    /// Strategies should be contained within a single program and should implement the
    /// StrategyInstruction interface below. If a Strategy requires additional data, it can specify
    /// it in a data account which will be included in calls to the strategy instance.
    ///
    /// Accounts expected:
    /// `[signer]` initializer of the lx token account
    /// `[writeable]` Vault storage account (vault ID)
    /// `[]` lX token account
    /// `[]` The llX Token ID with this program is a mint authority.
    /// `[]` The strategy program's pubkey.
    /// `[]` The rent sysvar
    /// `[]` (Optional) Strategy instance data account
    /// `[]` (Optional) X token account if hodling.
    InitializeVault {
        // TODO: Governance address, strategist address, keeper address.
        // TODO: Withdrawal fee.
        // https://github.com/yearn/yearn-vaults/blob/master/contracts/BaseStrategy.sol#L781
        strategy_program_deposit_instruction_id: u8,
        strategy_program_withdraw_instruction_id: u8,
        // TODO: Maybe change from bool to float percentage for holding.
        hodl: bool,
    },

    /// Deposits a given token into the vault.
    ///
    /// Accounts expected:
    /// 1. `[signer]` The source wallet containing X tokens.
    /// 2. `[]` The destination wallet for llX tokens.
    /// 4. `[]` SPL Token program
    /// 3. `[]` The Vault storage account.
    /// `[]` (Optional) X SPL account owned by Vault if hodling.
    /// TODO: Signer pubkeys for multisignature wallets.
    Deposit { amount: u64 },

    /// Withdraws a token from the strategy.
    ///
    /// Accounts expected:
    /// 2. `[signer]` Source Wallet for derivative token (lX).
    /// 1. `[]` Target token (X) wallet destination.
    /// 4. `[]` SPL Token program
    /// 3. `[]` The Vault storage account.
    /// `[]` (Optional) X SPL account owned by Vault if hodling.
    Withdraw {
        amount: u64, // # of derivative tokens.
    },
    // / An implementation of a Hodl strategy.
    // /
    // / TODO: Move this to a separate program?

    // / Initializes a hodl strategy.
    // /
    // / Accounts expected:
    // / 1 `[signer]` initializer of tokens
    // / 1. `[writable]` Storage account
    // / 2. `[]` X token wallet
    // / 2. `[]` lx mint
    // / 3. `[]` The rent sysvar
    // InitializeHodlStrategy{},
    // HodlStrategyDeposit {
    //     amount: u64,
    // },
    // HodlStrategyWithdraw {
    //     amount: u64,
    // }
}

// Strategy programs should implement the following interface for strategies.
pub enum StrategyInstruction {
    /// Deposits a token into the strategy.
    ///
    /// Accounts expected:
    /// 1. `[signer]` Source token (X) wallet
    /// 2. `[]` Target wallet for derivative token (lX)
    ///  `[]` SPL Token program
    /// 3. `[]` (Optional) Strategy instance data account
    /// TODO: Additional signers.
    Deposit {
        amount: u64, // # of X tokens.
    },
    /// Withdraws a token from the strategy.
    ///
    /// Accounts expected:
    /// 1. `[signer]` Source Wallet for derivative token (lX).
    /// 2. `[]` Target token (X) wallet destination.
    ///  `[]` SPL Token program
    /// 3. `[]` (Optional) Strategy instance data account
    /// TODO: Additional signers.
    Withdraw {
        amount: u64, // # of lX tokens.
    },
}

impl StrategyInstruction {
    /// Unpacks a byte buffer into a [VaultInstruction](enum.VaultInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 | 1 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                match tag {
                    1 => Self::Deposit { amount },
                    2 => Self::Withdraw { amount },
                    _ => return Err(VaultError::InvalidInstruction.into()),
                }
            }
            _ => return Err(VaultError::InvalidInstruction.into()),
        })
    }

    fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::Deposit { amount } => {
                buf.push(2);
                buf.extend_from_slice(&amount.to_le_bytes());
            }

            &Self::Withdraw { amount } => {
                buf.push(3);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
        }
        buf
    }

    pub fn deposit(
        program_id: &Pubkey,
        token_program_id: &Pubkey,
        source_pubkey: &Pubkey,
        target_pubkey: &Pubkey,
        additional_account_metas: Vec<AccountMeta>,
        amount: u64,
    ) -> Result<Instruction, ProgramError> {
        return create_transfer(
            Self::Deposit { amount }.pack(),
            program_id,
            token_program_id,
            source_pubkey,
            target_pubkey,
            additional_account_metas,
        );
    }

    pub fn withdraw(
        program_id: &Pubkey,
        token_program_id: &Pubkey,
        source_pubkey: &Pubkey,
        target_pubkey: &Pubkey,
        additional_account_metas: Vec<AccountMeta>,
        amount: u64,
    ) -> Result<Instruction, ProgramError> {
        return create_transfer(
            Self::Withdraw { amount }.pack(),
            program_id,
            token_program_id,
            source_pubkey,
            target_pubkey,
            additional_account_metas,
        );
    }
}

impl VaultInstruction {
    /// Unpacks a byte buffer into a [VaultInstruction](enum.VaultInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => {
                let hodl = *rest.get(0).unwrap();
                let strategy_program_deposit_instruction_id = *rest.get(0).unwrap();
                let strategy_program_withdraw_instruction_id = *rest.get(1).unwrap();
                Self::InitializeVault {
                    hodl: if hodl == 1 { true } else { false },
                    strategy_program_deposit_instruction_id,
                    strategy_program_withdraw_instruction_id,
                }
            }
            // 3 => {
            //     Self::InitializeHodlStrategy{}
            // }
            1 | 2 | 4 | 5 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                match tag {
                    1 => Self::Deposit { amount },
                    2 => Self::Withdraw { amount },
                    // 4 => Self::HodlStrategyDeposit { amount },
                    // 5 => Self::HodlStrategyWithdraw { amount },
                    _ => return Err(VaultError::InvalidInstruction.into()),
                }
            }
            _ => return Err(VaultError::InvalidInstruction.into()),
        })
    }

    fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::InitializeVault {
                hodl,
                strategy_program_deposit_instruction_id,
                strategy_program_withdraw_instruction_id,
            } => {
                buf.push(0);
                buf.push(hodl as u8);
                buf.push(strategy_program_deposit_instruction_id);
                buf.push(strategy_program_withdraw_instruction_id);
            }
            &Self::Deposit { amount } => {
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }

            &Self::Withdraw { amount } => {
                buf.push(2);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
        }
        buf
    }

    pub fn initialize_vault(
        vault_program_id: &Pubkey,
        initializer: &Pubkey,
        vault_storage_account: &Pubkey,
        lx_token_account: &Pubkey,
        llx_token_mint_id: &Pubkey,
        token_program: &Pubkey,
        strategy_program: &Pubkey,
        hodl: bool,
        x_token_account: COption<Pubkey>,
        strategy_program_deposit_instruction_id: u8,
        strategy_program_withdraw_instruction_id: u8,
    ) -> Result<Instruction, ProgramError> {
        let mut accounts = vec![
            AccountMeta::new_readonly(*initializer, true),
            AccountMeta::new(*vault_storage_account, false),
            AccountMeta::new_readonly(*lx_token_account, false),
            AccountMeta::new_readonly(*llx_token_mint_id, false),
            AccountMeta::new_readonly(*token_program, false),
            AccountMeta::new_readonly(*strategy_program, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ];
        assert_eq!(hodl, x_token_account.is_some());
        if hodl {
            accounts.push(AccountMeta::new_readonly(x_token_account.unwrap(), false));
        }
        let data = VaultInstruction::InitializeVault {
            hodl,
            strategy_program_deposit_instruction_id,
            strategy_program_withdraw_instruction_id,
        }
        .pack();
        Ok(Instruction {
            program_id: *vault_program_id,
            accounts,
            data,
        })
    }

    pub fn deposit(
        vault_program_id: &Pubkey,
        token_program_id: &Pubkey,
        source_pubkey: &Pubkey,
        target_pubkey: &Pubkey,
        additional_account_metas: Vec<AccountMeta>,
        amount: u64,
    ) -> Result<Instruction, ProgramError> {
        return create_transfer(
            Self::Deposit { amount }.pack(),
            vault_program_id,
            token_program_id,
            source_pubkey,
            target_pubkey,
            additional_account_metas,
        );
    }

    pub fn withdraw(
        vault_program_id: &Pubkey,
        token_program_id: &Pubkey,
        source_pubkey: &Pubkey,
        target_pubkey: &Pubkey,
        additional_account_metas: Vec<AccountMeta>,
        amount: u64,
    ) -> Result<Instruction, ProgramError> {
        return create_transfer(
            Self::Withdraw { amount }.pack(),
            vault_program_id,
            token_program_id,
            source_pubkey,
            target_pubkey,
            additional_account_metas,
        );
    }
    // pub fn withdraw(
    //     vault_program_id: &Pubkey,
    //     token_program_id: &Pubkey,
    //     source_pubkey: &Pubkey,
    //     target_pubkey: &Pubkey,
    //     amount: u64,
    // ) -> Result<Instruction, ProgramError> {
    //     let data = VaultInstruction::Deposit { amount }.pack();

    //     let accounts = vec![
    //         AccountMeta::new(*source_pubkey, false),
    //         AccountMeta::new(*target_pubkey, false),
    //         AccountMeta::new_readonly(*token_program_id, false),
    //     ];

    //     Ok(Instruction {
    //         program_id: *vault_program_id,
    //         accounts,
    //         data,
    //     })
    // }
}

pub fn create_transfer(
    data: Vec<u8>,
    vault_program_id: &Pubkey,
    token_program_id: &Pubkey,
    source_pubkey: &Pubkey,
    target_pubkey: &Pubkey,
    additional_account_metas: Vec<AccountMeta>,
) -> Result<Instruction, ProgramError> {
    let mut accounts = vec![
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new(*source_pubkey, false),
        AccountMeta::new(*target_pubkey, false),
    ];
    accounts.extend(additional_account_metas);

    Ok(Instruction {
        program_id: *vault_program_id,
        accounts,
        data,
    })
}
