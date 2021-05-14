use solana_program;

pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

// Random based on Token ID's ID. Defines Vault::id().
solana_program::declare_id!("VscrowegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
