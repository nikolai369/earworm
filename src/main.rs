use std::{fs::File, io::BufReader};

use crate::audio::WavReader;
use crate::error::Result;

mod audio;
mod error;

const PATH: &str = "/Users/nikolai369/Downloads/dj co.kr & h4rdy - ill e sam sa (vip).wav";

fn main() -> Result<()> {
    let file = File::open(PATH)?;
    let reader = BufReader::new(file);

    // Reads all headers unil the actual audio data
    let mut wav_reader = WavReader::new(reader)?;

    // Read vector of i32 samples
    let data = wav_reader.samples()?;

    // Each window is one second of audio
    let window_size = (wav_reader.config().fmt().channels as u32
        * wav_reader.config().fmt().sample_rate) as usize;
    let _ = data.windows(window_size);

    // Transform the air pressure data into frequency spectrum

    // Analyze peaks

    // Hash peaks
    Ok(())
}
