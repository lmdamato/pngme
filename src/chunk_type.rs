use std::fmt::Display;
use std::str::FromStr;
use std::convert::TryFrom;
use anyhow::anyhow;


#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct ChunkType {
    content: [u8; 4],
}

impl ChunkType {
    fn new(content: &[u8; 4]) -> Self {
        Self { content: *content }
    }

    fn is_valid_byte(x: &u8) -> bool {
        x.is_ascii_alphabetic()
    }

    fn is_valid_chunk_type(x: &[u8]) -> bool {
        x.iter().all(ChunkType::is_valid_byte)
    }

    fn try_from_bytes(x: &[u8; 4]) -> Result<Self, anyhow::Error> {
        if Self::is_valid_chunk_type(x) {
            Ok(Self::new(x))
        } else {
            Err(anyhow!("Chunk type must be a 4-byte string of ASCII letters"))
        }
    }

    fn try_from_str(x: &str) -> Result<Self, anyhow::Error> {
        if x.len() == 4 {
            Self::try_from_bytes(x.as_bytes().try_into()?)
        } else {
            Err(anyhow!("Chunk type must be a 4-byte string of ASCII letters"))
        }
    }
}

pub trait IChunkType {
    fn bytes(&self) -> [u8; 4];
    fn is_valid(&self) -> bool;
    fn is_critical(&self) -> bool;
    fn is_public(&self) -> bool;
    fn is_reserved_bit_valid(&self) -> bool;
    fn is_safe_to_copy(&self) -> bool;
}

impl IChunkType for ChunkType {
    fn bytes(&self) -> [u8; 4] {
        self.content
    }

    fn is_valid(&self) -> bool {
        self.content.iter().all(ChunkType::is_valid_byte) && self.is_reserved_bit_valid()
    }

    fn is_critical(&self) -> bool {
        self.content[0] >> 5 & 1 == 0
    }

    fn is_public(&self) -> bool {
        self.content[1] >> 5 & 1 == 0
    }

    fn is_reserved_bit_valid(&self) -> bool {
        self.content[2] >> 5 & 1 == 0
    }

    fn is_safe_to_copy(&self) -> bool {
        self.content[3] >> 5 & 1 == 1
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = anyhow::Error;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        Self::try_from_bytes(&value)
    }
}

impl FromStr for ChunkType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(s)
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.content))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
