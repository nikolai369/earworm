use std::{
    fs::File,
    io::{BufReader, Error, ErrorKind, Read, Result},
};

pub enum ChunkKind {
    Fmt,
    Data,
}

pub struct ChunkHeader {
    kind: ChunkKind,
    size: u32,
}

#[derive(Debug)]
pub struct WavFmt {
    pub channels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
}

impl WavFmt {}

// Helpers for reading WAVE file
pub trait ReaderExt {
    fn read_2_bytes(&mut self) -> Result<[u8; 2]>;

    fn read_3_bytes(&mut self) -> Result<[u8; 3]>;

    fn read_4_bytes(&mut self) -> Result<[u8; 4]>;

    fn read_le_u16(&mut self) -> Result<u16>;

    fn read_le_i16(&mut self) -> Result<i16>;

    fn read_le_i24(&mut self) -> Result<i32>;

    fn read_le_u32(&mut self) -> Result<u32>;

    fn read_le_i32(&mut self) -> Result<i32>;
}

// Extends any reader with these helper implementations
impl<R> ReaderExt for R
where
    R: Read,
{
    fn read_2_bytes(&mut self) -> Result<[u8; 2]> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_3_bytes(&mut self) -> Result<[u8; 3]> {
        let mut buf = [0u8; 3];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_4_bytes(&mut self) -> Result<[u8; 4]> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    // Heleper for reading an u16 bit little endian from buffer
    fn read_le_u16(&mut self) -> Result<u16> {
        let buf = self.read_2_bytes()?;
        Ok(u16::from_le_bytes(buf))
    }

    // Heleper for reading an i16 bit little endian from buffer
    fn read_le_i16(&mut self) -> Result<i16> {
        let buf = self.read_2_bytes()?;
        Ok(i16::from_le_bytes(buf))
    }

    // Heleper for reading an i24 bit little endian from buffer, note it will be an i32 sice there is no i24 in rust
    fn read_le_i24(&mut self) -> Result<i32> {
        let buf = self.read_3_bytes()?;
        if buf[2] & 0x80 == 0 {
            // 0x80 is equal to 10000000 in binary, use it as mask to extract the sign bit
            return Ok(i32::from_le_bytes([buf[0], buf[1], buf[2], 0x00])); // signed two complement num, padd with 00000000 to stay positive
        } else {
            return Ok(i32::from_le_bytes([buf[0], buf[1], buf[2], 0xFF])); // signed two complement num, padd with 11111111 to keep the sign
        }
    }

    // Heleper for reading an 32 bit little endian from buffer
    fn read_le_u32(&mut self) -> Result<u32> {
        let buf = self.read_4_bytes()?;
        Ok(u32::from_le_bytes(buf))
    }

    // Heleper for reading an 32 bit little endian from buffer
    fn read_le_i32(&mut self) -> Result<i32> {
        let buf = self.read_4_bytes()?;
        Ok(i32::from_le_bytes(buf))
    }
}

// Returns the file size
pub fn read_riff_header(reader: &mut BufReader<File>) -> Result<u64> {
    // Read the chunk id RIFF header
    let riff_id = reader.read_4_bytes()?;
    if &riff_id != b"RIFF" {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "RIFF header not present",
        ));
    }

    // Read the chuks size
    let chunks_size = reader.read_le_u32()?;

    // Read the WAVE format
    let format = reader.read_4_bytes()?;
    if &format != b"WAVE" {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "WAVE format not present",
        ));
    }

    // File size is chuks_size + the 8 bytes we already read
    // Might not fit so cast to u64
    Ok(chunks_size as u64 + 8)
}

fn read_chunk_header(reader: &mut BufReader<File>) -> Result<ChunkHeader> {
    let kind = reader.read_4_bytes()?;
    let size = reader.read_le_u32()?;
    match &kind {
        b"fmt " => Ok(ChunkHeader {
            kind: ChunkKind::Fmt,
            size,
        }),
        b"data" => Ok(ChunkHeader {
            kind: ChunkKind::Data,
            size,
        }),
        _ => Err(Error::new(
            ErrorKind::InvalidData,
            "Chunk header should be 'fmt ' or 'data'",
        )),
    }
}

fn read_fmt_subchunk(reader: &mut BufReader<File>, chunk_size: u32) -> Result<WavFmt> {
    if chunk_size < 16 {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid chunk size"));
    }

    let pcm = reader.read_le_u16()?;
    if pcm != 1 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Unsupported compression format",
        ));
    }

    let channels = reader.read_le_u16()?;
    if channels == 0 {
        return Err(Error::new(ErrorKind::InvalidData, "Channels cannot be 0"));
    }

    let sample_rate = reader.read_le_u32()?;
    let byte_rate = reader.read_le_u32()?;
    let block_align = reader.read_le_u16()?;
    let bits_per_sample = reader.read_le_u16()?;

    let expected_byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    if expected_byte_rate != byte_rate {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Invalid byte rate, expected {expected_byte_rate}, read {byte_rate}"),
        ));
    }

    let expected_block_align = channels * bits_per_sample / 8;
    if expected_block_align != block_align {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Invalid block align, expected {expected_block_align}, read {block_align}"),
        ));
    }

    Ok(WavFmt {
        channels,
        sample_rate,
        byte_rate,
        block_align,
        bits_per_sample,
    })
}

// pub fn read_audio_samples(reader: &mut BufReader<File>, samples: u32) {
//   let samples = wav_fmt
// }

pub fn read_until_data(reader: &mut BufReader<File>) -> Result<(WavFmt, u32)> {
    let size = read_riff_header(reader)?;

    println!("WAVE file opened, size: {}", size);

    let mut wav_fmt = None;
    loop {
        let chunk_header = read_chunk_header(reader)?;
        match chunk_header.kind {
            ChunkKind::Fmt => {
                wav_fmt = Some(read_fmt_subchunk(reader, chunk_header.size)?);
            }
            ChunkKind::Data => {
                let data_size = chunk_header.size;
                if let Some(wav) = wav_fmt {
                    println!(
                        "WAVE file chunks read, fmt: {:?}, data size: {data_size}",
                        wav
                    );
                    return Ok((wav, data_size));
                } else {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid block align, expected {expected_block_align}, read {block_align}",
                    ));
                }
            } // Read until data, read it on demand later
        };
    }
}

mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_2_bytes() {
        let data = vec![0x12, 0x34];
        let mut cursor = Cursor::new(data);
        let result = cursor.read_2_bytes().unwrap();
        assert_eq!(result, [0x12, 0x34]);
    }

    #[test]
    fn test_read_3_bytes() {
        let data = vec![0x01, 0x02, 0x03];
        let mut cursor = Cursor::new(data);
        let result = cursor.read_3_bytes().unwrap();
        assert_eq!(result, [0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_read_4_bytes() {
        let data = vec![0x0A, 0x0B, 0x0C, 0x0D];
        let mut cursor = Cursor::new(data);
        let result = cursor.read_4_bytes().unwrap();
        assert_eq!(result, [0x0A, 0x0B, 0x0C, 0x0D]);
    }

    #[test]
    fn test_read_le_u16() {
        let data = vec![0x34, 0x12]; // Little endian for 0x1234 = 4660
        let mut cursor = Cursor::new(data);
        let result = cursor.read_le_u16().unwrap();
        assert_eq!(result, 0x1234);
    }

    #[test]
    fn test_read_le_i16() {
        let data = vec![0xFF, 0x7F]; // Little endian for i16 max: 32767
        let mut cursor = Cursor::new(data);
        let result = cursor.read_le_i16().unwrap();
        assert_eq!(result, 32767);

        let data_neg = vec![0x00, 0x80]; // Little endian for i16 min: -32768
        let mut cursor = Cursor::new(data_neg);
        let result = cursor.read_le_i16().unwrap();
        assert_eq!(result, -32768);
    }

    #[test]
    fn test_read_le_i24() {
        let data = vec![0x00, 0x00, 0x80]; // negative number (sign bit set)
        let mut cursor = Cursor::new(data);
        let result = cursor.read_le_i24().unwrap();
        assert_eq!(result, -8388608); // min 24-bit signed int

        let data_pos = vec![0xFF, 0xFF, 0x7F]; // positive max 24-bit int
        let mut cursor = Cursor::new(data_pos);
        let result = cursor.read_le_i24().unwrap();
        assert_eq!(result, 8_388_607);
    }

    #[test]
    fn test_read_le_u32() {
        let data = vec![0x78, 0x56, 0x34, 0x12]; // 0x12345678 little endian
        let mut cursor = Cursor::new(data);
        let result = cursor.read_le_u32().unwrap();
        assert_eq!(result, 0x12345678);
    }

    #[test]
    fn test_read_le_i32() {
        let data = vec![0xFF, 0xFF, 0xFF, 0x7F]; // max i32 (2147483647)
        let mut cursor = Cursor::new(data);
        let result = cursor.read_le_i32().unwrap();
        assert_eq!(result, 2147483647);

        let data_neg = vec![0x00, 0x00, 0x00, 0x80]; // min i32 (-2147483648)
        let mut cursor = Cursor::new(data_neg);
        let result = cursor.read_le_i32().unwrap();
        assert_eq!(result, -2147483648);
    }
}
