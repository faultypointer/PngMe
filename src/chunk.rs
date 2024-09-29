use std::fmt::Display;
use std::io::{BufReader, Read};
use std::str::FromStr;

use anyhow::bail;

use crate::{chunk_type, Error};

use crate::chunk_type::ChunkType;

pub struct Chunk {
    length: u32,
    chunktype: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let mut temp = chunk_type.bytes().to_vec();
        temp.extend(&data);
        let crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        let crc = crc.checksum(&temp);
        Chunk {
            length: data.len() as u32,
            chunktype: chunk_type,
            data,
            crc,
        }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunktype
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_as_string(&self) -> crate::Result<String> {
        match String::from_utf8(self.data.clone()) {
            Ok(str) => Ok(str),
            Err(_) => bail!("couldn't convert chunk data to string"),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.length as usize + 12);
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes.extend_from_slice(&self.chunktype.bytes());
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.crc.to_be_bytes());

        bytes
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut len: [u8; 4] = [0, 0, 0, 0];
        let mut chunk_type: [u8; 4] = [0, 0, 0, 0];
        let mut crc: [u8; 4] = [0, 0, 0, 0];
        let mut reader = BufReader::new(value);

        reader.read_exact(&mut len)?;
        let len = u32::from_be_bytes(len);

        reader.read_exact(&mut chunk_type)?;
        let chunk_type = ChunkType::try_from(chunk_type)?;

        let mut data = Vec::try_from(&value[8..len as usize + 8])?;
        reader.read_exact(data.as_mut())?;

        reader.read_exact(&mut crc)?;
        let crc = u32::from_be_bytes(crc);

        let valid_crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        let valid_crc = valid_crc.checksum(&value[4..len as usize + 8]);

        if crc != valid_crc {
            bail!(
                "Invalid CRC in the given value: {}. calculated crc: {}",
                crc,
                valid_crc
            )
        }

        Ok(Chunk {
            length: len,
            chunktype: chunk_type,
            data,
            crc,
        })
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        // Append the length (4 bytes, big-endian)
        output.extend(
            self.length
                .to_be_bytes()
                .iter()
                .map(|b| format!("{:02X}", b)),
        );

        // Append the chunk type (4 bytes)
        output.push_str(&self.chunktype.to_string());

        // Append the data bytes
        output.extend(self.data.iter().map(|b| format!("{:02X}", b)));

        // Append the CRC (4 bytes, big-endian)
        output.extend(self.crc.to_be_bytes().iter().map(|b| format!("{:02X}", b)));

        // Write the entire output at once
        f.write_str(&output)
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
    //
    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
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
