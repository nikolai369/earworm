use std::{
    fs::File,
    io::{BufReader, Error, ErrorKind, Read, Result},
};

const PATH: &str = "/Users/nikolai369/Downloads/dj co.kr & h4rdy - ill e sam sa (vip).wav";

enum ChunkKind {
    Fmt,
    Data,
}

struct ChunkHeader {
    kind: ChunkKind,
    size: u32,
}
#[derive(Debug)]
struct WavFmt {
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

fn read_2_bytes(reader: &mut BufReader<File>) -> Result<[u8; 2]> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

fn read_4_bytes(reader: &mut BufReader<File>) -> Result<[u8; 4]> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

// Heleper for reading an 32 bit little endian from buffer
fn read_le_u32(reader: &mut BufReader<File>) -> Result<u32> {
    let buf = read_4_bytes(reader)?;
    let n = u32::from_le_bytes(buf);
    Ok(n)
}

fn read_le_u16(reader: &mut BufReader<File>) -> Result<u16> {
    let buf = read_2_bytes(reader)?;
    let n = u16::from_le_bytes(buf);
    Ok(n)
}

// Returns the file size
fn read_riff_header(reader: &mut BufReader<File>) -> Result<u64> {
    // Read the chunk id RIFF header
    let riff_id = read_4_bytes(reader)?;
    if &riff_id != b"RIFF" {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "RIFF header not present",
        ));
    }

    // Read the chuks size
    let chunks_size = read_le_u32(reader)?;

    // Read the WAVE format
    let format = read_4_bytes(reader)?;
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
    let kind = read_4_bytes(reader)?;
    let size = read_le_u32(reader)?;
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

    let pcm = read_le_u16(reader)?;
    if pcm != 1 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Unsupported compression format",
        ));
    }

    let channels = read_le_u16(reader)?;
    if channels == 0 {
        return Err(Error::new(ErrorKind::InvalidData, "Channels cannot be 0"));
    }

    let sample_rate = read_le_u32(reader)?;
    let byte_rate = read_le_u32(reader)?;
    let block_align = read_le_u16(reader)?;
    let bits_per_sample = read_le_u16(reader)?;

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

fn read_data_subchunk(reader: &mut BufReader<File>) {}

fn main() -> Result<()> {
    let file = File::open(PATH)?;
    let mut reader = BufReader::new(file);

    let size = read_riff_header(&mut reader)?;

    println!("WAVE file opened, size: {}", size);

    let mut wav_fmt = None;
    let data_size;
    loop {
        let chunk_header = read_chunk_header(&mut reader)?;
        match chunk_header.kind {
            ChunkKind::Fmt => {
                wav_fmt = Some(read_fmt_subchunk(&mut reader, chunk_header.size)?);
            }
            ChunkKind::Data => {
                data_size = chunk_header.size;
                break;
            } // If data break, read it on demand later
        };
    }

    if let Some(wav) = wav_fmt {
        println!(
            "WAVE file chunks read, fmt: {:?}, data size: {data_size}",
            wav
        );
    }
    Ok(())
}
