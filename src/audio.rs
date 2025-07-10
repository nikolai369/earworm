use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::error::{Result, WavError};

enum ChunkKind {
    Fmt,
    Data,
}

struct ChunkHeader {
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

pub struct WavConfig {
    wav_fmt: WavFmt,
    size: u32,
    samples: u32,
    bytes_per_sample: u32,
}

impl WavConfig {
    pub fn duration(&self) -> u32 {
        self.samples / (self.wav_fmt.channels as u32) / self.wav_fmt.sample_rate
    }

    pub fn fmt(&self) -> &WavFmt {
        &self.wav_fmt
    }
}

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

pub struct WavReader {
    reader: BufReader<File>,
    config: WavConfig,
}

impl WavReader {
    pub fn new(reader: BufReader<File>) -> Result<Self> {
        let mut reader = reader;
        let (wav_fmt, size) = read_until_data(&mut reader)?;

        let bytes_per_sample = (wav_fmt.bits_per_sample / 8) as u32;
        let samples = size / bytes_per_sample;

        if samples % wav_fmt.channels as u32 != 0 {
            return Err(WavError::Corrupted(
                "Number of samples per channel must be equal",
            ));
        }
        let config = WavConfig {
            wav_fmt,
            size,
            bytes_per_sample,
            samples,
        };
        let wav_reader = WavReader { reader, config };
        Ok(wav_reader)
    }

    pub fn mono(&mut self) -> Result<Vec<i32>> {
        let data_size: usize = match self.config.samples.try_into() {
            Ok(val) => val,
            Err(_) => {
                return Err(WavError::UnsupportedFormat("Too many samples to process"));
            }
        };

        let channels = self.config.wav_fmt.channels;
        let mut data = Vec::with_capacity(data_size / channels as usize);
        for _ in 0..data.capacity() {
            let mut channel_sum = 0; //TODO: maybe use i64 to avoid data loss if sum overflows
            for _ in 0..channels {
                let sample = match self.config.wav_fmt.bits_per_sample {
                    16 => self.reader.read_le_i16()? as i32, // TODO: cast all to i32 for now
                    24 => self.reader.read_le_i24()? as i32,
                    32 => self.reader.read_le_i32()? as i32,
                    _ => {
                        return Err(WavError::UnsupportedFormat("Sample size not supported"));
                    }
                };

                channel_sum += sample;
            }

            // Average the amplitude to avoid clipping
            data.push(channel_sum / channels as i32);
        }

        Ok(data)
    }

    pub fn config(&self) -> &WavConfig {
        &self.config
    }

    pub fn size(&self) -> u32 {
        self.config.size
    }
}

// Returns the file size
fn read_riff_header(reader: &mut BufReader<File>) -> Result<u64> {
    // Read the chunk id RIFF header
    let riff_id = reader.read_4_bytes()?;
    if &riff_id != b"RIFF" {
        return Err(WavError::InvalidFormat("RIFF header not present"));
    }

    // Read the chuks size
    let chunks_size = reader.read_le_u32()?;

    // Read the WAVE format
    let format = reader.read_4_bytes()?;
    if &format != b"WAVE" {
        return Err(WavError::InvalidFormat("WAVE format not present"));
    }

    // File size is chuks_size + the 8 bytes we already read
    // Might not fit so cast to u64
    Ok(chunks_size as u64 + 8)
}

// Reads "data" and "fmt " chunk headers
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
        _ => Err(WavError::InvalidFormat(
            "Chunk header should be 'fmt ' or 'data'",
        )),
    }
}

// Reads the WAVE file fmt spec
fn read_fmt_subchunk(reader: &mut BufReader<File>, chunk_size: u32) -> Result<WavFmt> {
    if chunk_size < 16 {
        return Err(WavError::Corrupted("Invalid chunk size"));
    }

    let pcm = reader.read_le_u16()?;
    if pcm != 1 {
        return Err(WavError::UnsupportedFormat(
            "Unsupported compression format",
        ));
    }

    let channels = reader.read_le_u16()?;
    if channels == 0 {
        return Err(WavError::Corrupted("Channels cannot be 0"));
    }

    let sample_rate = reader.read_le_u32()?;
    let byte_rate = reader.read_le_u32()?;
    let block_align = reader.read_le_u16()?;
    let bits_per_sample = reader.read_le_u16()?;

    let expected_byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    if expected_byte_rate != byte_rate {
        return Err(WavError::Corrupted("Invalid byte rate"));
    }

    let expected_block_align = channels * bits_per_sample / 8;
    if expected_block_align != block_align {
        return Err(WavError::Corrupted("Invalid block align"));
    }

    Ok(WavFmt {
        channels,
        sample_rate,
        byte_rate,
        block_align,
        bits_per_sample,
    })
}

// Reads all chunks up until the actual audi data samples
fn read_until_data(reader: &mut BufReader<File>) -> Result<(WavFmt, u32)> {
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
                    return Ok((wav, data_size));
                } else {
                    return Err(WavError::InvalidFormat("WAVE \"fmt \" not present"));
                }
            } // Read until data, read it on demand later
        };
    }
}

mod tests {
    use super::ReaderExt;

    #[test]
    fn test_read_2_bytes() {
        let data = vec![0x12, 0x34];
        let mut cursor = std::io::Cursor::new(data);
        let result = cursor.read_2_bytes().unwrap();
        assert_eq!(result, [0x12, 0x34]);
    }

    #[test]
    fn test_read_3_bytes() {
        let data = vec![0x01, 0x02, 0x03];
        let mut cursor = std::io::Cursor::new(data);
        let result = cursor.read_3_bytes().unwrap();
        assert_eq!(result, [0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_read_4_bytes() {
        let data = vec![0x0A, 0x0B, 0x0C, 0x0D];
        let mut cursor = std::io::Cursor::new(data);
        let result = cursor.read_4_bytes().unwrap();
        assert_eq!(result, [0x0A, 0x0B, 0x0C, 0x0D]);
    }

    #[test]
    fn test_read_le_u16() {
        let data = vec![0x34, 0x12]; // Little endian for 0x1234 = 4660
        let mut cursor = std::io::Cursor::new(data);
        let result = cursor.read_le_u16().unwrap();
        assert_eq!(result, 0x1234);
    }

    #[test]
    fn test_read_le_i16() {
        let data = vec![0xFF, 0x7F]; // Little endian for i16 max: 32767
        let mut cursor = std::io::Cursor::new(data);
        let result = cursor.read_le_i16().unwrap();
        assert_eq!(result, 32767);

        let data_neg = vec![0x00, 0x80]; // Little endian for i16 min: -32768
        let mut cursor = std::io::Cursor::new(data_neg);
        let result = cursor.read_le_i16().unwrap();
        assert_eq!(result, -32768);
    }

    #[test]
    fn test_read_le_i24() {
        let data = vec![0x00, 0x00, 0x80]; // negative number (sign bit set)
        let mut cursor = std::io::Cursor::new(data);
        let result = cursor.read_le_i24().unwrap();
        assert_eq!(result, -8388608); // min 24-bit signed int

        let data_pos = vec![0xFF, 0xFF, 0x7F]; // positive max 24-bit int
        let mut cursor = std::io::Cursor::new(data_pos);
        let result = cursor.read_le_i24().unwrap();
        assert_eq!(result, 8_388_607);
    }

    #[test]
    fn test_read_le_u32() {
        let data = vec![0x78, 0x56, 0x34, 0x12]; // 0x12345678 little endian
        let mut cursor = std::io::Cursor::new(data);
        let result = cursor.read_le_u32().unwrap();
        assert_eq!(result, 0x12345678);
    }

    #[test]
    fn test_read_le_i32() {
        let data = vec![0xFF, 0xFF, 0xFF, 0x7F]; // max i32 (2147483647)
        let mut cursor = std::io::Cursor::new(data);
        let result = cursor.read_le_i32().unwrap();
        assert_eq!(result, 2147483647);

        let data_neg = vec![0x00, 0x00, 0x00, 0x80]; // min i32 (-2147483648)
        let mut cursor = std::io::Cursor::new(data_neg);
        let result = cursor.read_le_i32().unwrap();
        assert_eq!(result, -2147483648);
    }
}
