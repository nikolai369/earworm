use std::{fs::File, io::BufReader};

use crate::audio::{ReaderExt, read_until_data};
use crate::error::{Result, WavError};

mod audio;
mod error;

const PATH: &str = "/Users/nikolai369/Downloads/dj co.kr & h4rdy - ill e sam sa (vip).wav";

fn main() -> Result<()> {
    let file = File::open(PATH)?;
    let mut reader = BufReader::new(file);

    let (wav, size) = read_until_data(&mut reader)?;

    let bytes_per_sample = (wav.bits_per_sample / 8) as u32;
    let samples = size / bytes_per_sample;

    if samples % wav.channels as u32 != 0 {
        return Err(WavError::Corrupted(
            "Number of samples per channel must be equal",
        ));
    }

    let duration = samples / wav.channels as u32 / wav.sample_rate;
    println!("Size: {size}");
    println!("Samples: {samples}");
    println!("Track duration: {duration}s");
    println!("fmt: {:?}", wav);

    let data_size: usize = match samples.try_into() {
        Ok(val) => val,
        Err(_) => {
            return Err(WavError::UnsupportedFormat("Too many samples to process"));
        }
    };
    let mut data = vec![0; data_size];
    for _ in 0..samples {
        let sample = match wav.bits_per_sample {
            16 => reader.read_le_i16()? as i32, //cast all to i32 for now
            24 => reader.read_le_i24()? as i32,
            32 => reader.read_le_i32()? as i32,
            _ => {
                return Err(WavError::UnsupportedFormat("Sample size not supported"));
            }
        };

        data.push(sample);
    }

    Ok(())
}
