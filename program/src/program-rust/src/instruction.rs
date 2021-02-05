use solana_program::program_error::ProgramError;
use std::convert::TryInto;

use crate::error::SolanarollError::InvalidInstruction;

pub enum SolanarollInstruction {
    /// Deposit funds - receive tokens
    ///
    ///
    /// Accounts expected:
    ///
    Deposit {
        amount: u64,
    },
    /// Withdraw funds - burn tokens
    ///
    ///
    /// Accounts expected:
    ///
    Withdraw {
        amount: u64,
    },
    /// Commits a reveal number to play dice game
    ///
    ///
    /// Accounts expected:
    ///
    CommitReveal {
        reveal_number: u64,
        amount: u64,
        roll_under: u32,
    },
    /// Resolves a dice game roll
    ///
    ///
    /// Accounts expected:
    ///
    ResolveRoll {
        reveal_number: u64,
        amount: u64,
        roll_under: u32,
    },
}

impl SolanarollInstruction {
    /// Unpacks a byte buffer into a SolanarollInstruction
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::CommitReveal {
                reveal_number: Self::unpack_amount(rest)?,
                amount: Self::unpack_amount(rest)?,
                roll_under: Self::unpack_amount_32(rest)?,
            },
            1 => Self::ResolveRoll {
                reveal_number: Self::unpack_amount(rest)?,
                amount: Self::unpack_amount(rest)?,
                roll_under: Self::unpack_amount_32(rest)?,
            },
            2 => Self::Deposit {
                amount: Self::unpack_amount(rest)?,
            },
            3 => Self::Withdraw {
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

    fn unpack_amount_32(input: &[u8]) -> Result<u32, ProgramError> {
        let amount = input
            .get(..4)
            .and_then(|slice| slice.try_into().ok())
            .map(u32::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
}
