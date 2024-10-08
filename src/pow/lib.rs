

use std::{
    array::TryFromSliceError,
    fmt::{Debug, Display, Formatter},
    hash::{Hash as StdHash, Hasher as StdHasher},
    str::{self, FromStr},
};


pub const HASH_SIZE: usize = 32;


#[derive(Clone, Copy, Debug)]
pub struct LHash(pub(crate) [u8; HASH_SIZE]);



impl From<[u8; HASH_SIZE]> for LHash {
    fn from(value: [u8; HASH_SIZE]) -> Self {
        LHash(value)
    }
}

impl TryFrom<&[u8]> for LHash {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        LHash::try_from_slice(value)
    }
}

impl LHash {
    #[inline(always)]
    pub const fn from_bytes(bytes: [u8; HASH_SIZE]) -> Self {
        LHash(bytes)
    }

    #[inline(always)]
    pub const fn as_bytes(self) -> [u8; 32] {
        self.0
    }

    #[inline(always)]
    /// # Panics
    /// Panics if `bytes` length is not exactly `HASH_SIZE`.
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self(<[u8; HASH_SIZE]>::try_from(bytes).expect("Slice must have the length of Hash"))
    }

    #[inline(always)]
    pub fn try_from_slice(bytes: &[u8]) -> Result<Self, TryFromSliceError> {
        Ok(Self(<[u8; HASH_SIZE]>::try_from(bytes)?))
    }

    #[inline(always)]
    pub fn to_le_u64(self) -> [u64; 4] {
        let mut out = [0u64; 4];
        out.iter_mut().zip(self.iter_le_u64()).for_each(|(out, word)| *out = word);
        out
    }

    #[inline(always)]
    pub fn iter_le_u64(&self) -> impl ExactSizeIterator<Item = u64> + '_ {
        self.0.chunks_exact(8).map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
    }

    #[inline(always)]
    pub fn from_le_u64(arr: [u64; 4]) -> Self {
        let mut ret = [0; HASH_SIZE];
        ret.chunks_exact_mut(8).zip(arr.iter()).for_each(|(bytes, word)| bytes.copy_from_slice(&word.to_le_bytes()));
        Self(ret)
    }

    #[inline(always)]
    pub fn from_u64_word(word: u64) -> Self {
        Self::from_le_u64([0, 0, 0, word])
    }
}

// Override the default Hash implementation, to: A. improve perf a bit (siphash works over u64s), B. allow a hasher to just take the first u64.
// Don't change this without looking at `consensus/core/src/blockhash/BlockHashMap`.
impl StdHash for LHash {
    #[inline(always)]
    fn hash<H: StdHasher>(&self, state: &mut H) {
        self.iter_le_u64().for_each(|x| x.hash(state));
    }
}

/// We only override PartialEq because clippy wants us to.
/// This should always hold: PartialEq(x,y) => Hash(x) == Hash(y)
impl PartialEq for LHash {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}


impl Display for LHash {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut hex = [0u8; HASH_SIZE * 2];
        faster_hex::hex_encode(&self.0, &mut hex).expect("The output is exactly twice the size of the input");
        f.write_str(unsafe { str::from_utf8_unchecked(&hex) })
    }
}
pub trait ToHex {
    fn to_hex(&self) -> String;
}
impl ToHex for LHash {
    fn to_hex(&self) -> String {
        self.to_string()
    }
}

impl From<u64> for LHash {
    #[inline(always)]
    fn from(word: u64) -> Self {
        Self::from_u64_word(word)
    }
}

impl AsRef<[u8; HASH_SIZE]> for LHash {
    #[inline(always)]
    fn as_ref(&self) -> &[u8; HASH_SIZE] {
        &self.0
    }
}

impl AsRef<[u8]> for LHash {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
pub trait FromHex: Sized {
    type Error: std::fmt::Display;
    fn from_hex(hex_str: &str) -> Result<Self, Self::Error>;
}
impl FromStr for LHash {
    type Err = faster_hex::Error;

    #[inline]
    fn from_str(hash_str: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0u8; HASH_SIZE];
        faster_hex::hex_decode(hash_str.as_bytes(), &mut bytes)?;
        Ok(LHash(bytes))
    }
}

impl FromHex for LHash {
    type Error = faster_hex::Error;
    fn from_hex(hex_str: &str) -> Result<Self, Self::Error> {
        Self::from_str(hex_str)
    }
}

pub const ZERO_HASH: LHash = LHash([0; HASH_SIZE]);

pub const EMPTY_MUHASH: LHash = LHash::from_bytes([
    0x54, 0x4e, 0xb3, 0x14, 0x2c, 0x0, 0xf, 0xa, 0xd2, 0xc7, 0x6a, 0xc4, 0x1f, 0x42, 0x22, 0xab, 0xba, 0xba, 0xbe, 0xd8, 0x30, 0xee,
    0xaf, 0xee, 0x4b, 0x6d, 0xc5, 0x6b, 0x52, 0xd5, 0xca, 0xc0,
]);

#[cfg(test)]
mod tests {
    use super::LHash;
    use std::str::FromStr;

    #[test]
    fn test_hash_basics() {
        let hash_str = "8e40af02265360d59f4ecf9ae9ebf8f00a3118408f5a9cdcbcc9c0f93642f3af";
        let hash = LHash::from_str(hash_str).unwrap();
        assert_eq!(hash_str, hash.to_string());
        let hash2 = LHash::from_str(hash_str).unwrap();
        assert_eq!(hash, hash2);

        let hash3 = LHash::from_str("8e40af02265360d59f4ecf9ae9ebf8f00a3118408f5a9cdcbcc9c0f93642f3ab").unwrap();
        assert_ne!(hash2, hash3);

        let odd_str = "8e40af02265360d59f4ecf9ae9ebf8f00a3118408f5a9cdcbcc9c0f93642f3a";
        let short_str = "8e40af02265360d59f4ecf9ae9ebf8f00a3118408f5a9cdcbcc9c0f93642f3";

        assert!(matches!(dbg!(LHash::from_str(odd_str)), Err(faster_hex::Error::InvalidLength(len)) if len == 64));
        assert!(matches!(dbg!(LHash::from_str(short_str)), Err(faster_hex::Error::InvalidLength(len)) if len == 64));
    }
}
