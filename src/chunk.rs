use std::{fmt::Display, slice::Iter};
use anyhow::{Result, anyhow, Ok};
use crate::chunk_type::{ChunkType, IChunkType};
use crc::{Crc, CRC_32_ISO_HDLC};
use lazycell::LazyCell;

#[derive(Debug)]
#[derive(Clone)]
pub struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: LazyCell<u32>,
}

impl Chunk {
    pub const CHUNK_TYPE_SIZE: usize = 4;
    pub const CHUNK_LENGTH_SIZE: usize = 4;
    pub const CRC_SIZE: usize = 4;
    pub const MIN_CHUNK_SIZE: usize = (
        Chunk::CHUNK_TYPE_SIZE + Chunk::CHUNK_LENGTH_SIZE + Chunk::CRC_SIZE
    );

    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        Self { chunk_type, data, crc: LazyCell::new() }
    }

    fn compute_crc(chunk_code: &[u8; 4], data: &Vec<u8>) -> u32 {
        let crc: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();

        digest.update(chunk_code);
        digest.update(data);

        digest.finalize()
    }

    fn to_string(&self) -> String {
        String::from_utf8_lossy(
            self.as_bytes().as_slice()
        ).to_string()
    }
}

pub trait IChunk {
    fn length(&self) -> usize;
    fn chunk_type(&self) -> &ChunkType;
    fn data(&self) -> &[u8];
    fn crc(&self) -> u32;
    fn data_as_string(&self) -> Result<String>;
    fn as_bytes(&self) -> Vec<u8>;
}

impl IChunk for Chunk {
    fn length(&self) -> usize {
        self.data.len()
    }

    fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn crc(&self) -> u32 {
        *self.crc.borrow_with(
            || Chunk::compute_crc(&self.chunk_type.bytes(), &self.data)
        )
    }

    fn data_as_string(&self) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.data).to_string())
    }

    fn as_bytes(&self) -> Vec<u8> {
        let chunk_length_bytes = self.length().to_be_bytes()[4..].to_vec();
        let chunk_type = self.chunk_type().bytes().to_vec();
        let data = self.data.clone();
        let crc = self.crc().to_be_bytes().to_vec();

        [chunk_length_bytes, chunk_type, data, crc].concat()
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        fn next_n_bytes(it: &mut Iter<u8>, n: usize) -> Result<Vec<u8>, anyhow::Error> {
            let res = it.take(n)
                .map(|x| *x)
                .collect::<Vec<u8>>();

            if res.len() != n {
                Err(anyhow!(
                    "Tried to retrieve next {} bytes, could only retrieve {} instead", n, res.len(),
                ))?;
            }

            Ok(res)
        }

        fn next_four_bytes(it: &mut Iter<u8>) -> Result<[u8; 4], anyhow::Error> {
            next_n_bytes(it, 4)
                .and_then(|x| x.try_into().map_err(|e| {
                    anyhow!("Expected 4-byte vector, got: {:?}", e)
                }))

                // Another way:
                //
                // .and_then(|x| {
                //     <[u8; 4]>::try_from(x).map_err(|e| anyhow!("..."))
                // })
        }

        if value.len() < Chunk::MIN_CHUNK_SIZE {
            Err(anyhow!("Input array is too short"))?;
        }

        let mut it = value.iter();

        let content_size_x: [u8; 4] = next_four_bytes(&mut it)?;
        let content_size: usize = u32::from_be_bytes(content_size_x).try_into()?;

        if value.len() != content_size + Chunk::MIN_CHUNK_SIZE {
            Err(anyhow!(
                "{} {}. Got chunk of length {}, expected from chunk header: {}",
                "Malformed input byte array: input size as from the",
                "byte array doesn't match the actual chunk size",
                value.len(),
                content_size
            ))?;
        }
    
        let chunk_type_x: [u8; 4] = next_four_bytes(&mut it)?;
        let chunk_type = ChunkType::try_from(chunk_type_x)?;

        let content: Vec<u8> = next_n_bytes(&mut it, content_size)?;

        let crc_x: [u8; 4] = next_four_bytes(&mut it)?;
        let expected_crc: u32 = u32::from_be_bytes(crc_x).try_into()?;

        if it.next().is_some() {
            Err(anyhow!("Malformed input byte array"))?;
        } 
        
        let computed_crc = Chunk::compute_crc(&chunk_type.bytes(), &content);
        if computed_crc != expected_crc {
             Err(anyhow!(
                "CRC mismatch. Computed from input data: {}, expected: {}", 
                computed_crc, 
                expected_crc
            ))?;
        }

        Ok(Chunk::new(chunk_type, content))
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        
        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        
        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();
        
        let _chunk_string = format!("{}", chunk);
    }
}
