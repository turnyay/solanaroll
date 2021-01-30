
use std::hash::{Hash as StdHash, Hasher};
use std::collections::hash_map::DefaultHasher;

fn hash_value<T>(obj: T) -> u64
    where
        T: StdHash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

const MAX_NUM_SLOT_HASHES: u64 = 512;
fn get_slot_hash(data: &[u8], slot_height: u64) -> Hash {
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

fn unpack_mint(data: &[u8]) -> Result<Mint, SwapError> {
    TokenPack::unpack(data).map_err(|_| SwapError::ExpectedMint)
}

