use std::{
    fs::File,
    io::{BufReader, Read},
};

const PATH: &str = "/Users/nikolai369/Downloads/dj co.kr & h4rdy - ill e sam sa (vip).wav";

fn main() {
    let file = File::open(PATH).unwrap();
    let mut reader = BufReader::new(file);

    // Read the RIFF header
    let mut header: [u8; 12] = [0; 12];
    let _ = reader.read_exact(&mut header).unwrap();
    let is_riff = &header[..4] == (b"RIFF");

    if !is_riff {
        eprintln!("RIFF not found");
        return;
    };

    // RIFF id
    let c_id = &header[0..4];
    // Size of the chunks minus 8 read bytes
    let c_size = u32::from_le_bytes(header[4..8].try_into().unwrap());
    // Should be WAVE format
    let format = &header[8..];
    let is_wave = format == b"WAVE";

    if !is_wave {
        return;
    }

    println!("WAVE file opened, size: {}", c_size)
}
