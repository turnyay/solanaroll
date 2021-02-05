
use byteorder::{ByteOrder, BigEndian, LittleEndian};
use solana_sdk::{
    account_info::{next_account_info, AccountInfo},
    entrypoint_deprecated,
    entrypoint_deprecated::ProgramResult,
    info,
    hash::{Hash, HASH_BYTES},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{
        clock::Clock, slot_hashes::SlotHashes, Sysvar,
    },
};

use crate::{
    error::SolanarollError,
};

use std::convert::TryInto;

use solana_sdk::program::invoke_signed;
use spl_token::{instruction};
use solana_sdk::program_pack::Pack as TokenPack;
use spl_token::state::{Account as TokenAccount, Mint};

use std::hash::{Hash as StdHash, Hasher};
use std::collections::hash_map::DefaultHasher;

use num_derive::FromPrimitive;
use solana_sdk::{decode_error::DecodeError};
use thiserror::Error;

pub fn hash_value<T>(obj: T) -> u64
    where
        T: StdHash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

const MAX_NUM_SLOT_HASHES: u64 = 512;
pub fn get_slot_hash(data: &[u8], slot_height: u64) -> Hash {
    let current_slot = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let diff = current_slot - slot_height;
    if (diff > MAX_NUM_SLOT_HASHES) {
        let mut buf = [0u8; HASH_BYTES];
        return Hash::new(&buf);
    } else {
        let target_index = (16 + (diff * 40)) as usize;
        let hash = &data[target_index..target_index + 32];
        return Hash::new(hash);
    }
}

pub fn unpack_mint(data: &[u8]) -> Result<Mint, SolanarollError> {
    TokenPack::unpack(data).map_err(|_| SolanarollError::ExpectedMint)
}
