use std::{
    fs::File,
    io::{BufReader, Result},
};

use crate::audio::{ReaderExt, read_until_data};

mod audio;

const PATH: &str = "/Users/nikolai369/Downloads/dj co.kr & h4rdy - ill e sam sa (vip).wav";

fn main() -> Result<()> {
    let file = File::open(PATH)?;
    let mut reader = BufReader::new(file);

    let (wav, size) = read_until_data(&mut reader)?;

    let bytes_per_sample = (wav.bits_per_sample / 8) as u32;
    let samples = size / bytes_per_sample;

    if samples % wav.channels as u32 != 0 {
        return Ok(()); // TODO: error out, create custom error kind and messages
    }

    let duration = samples / wav.channels as u32 / wav.sample_rate;
    println!("Size: {size}");
    println!("Samples: {samples}");
    println!("Track duration: {duration}s");
    println!("fmt: {:?}", wav);

    let data_size: usize = samples.try_into().expect("Handle a lot of samples"); // TODO: better
    let mut data = vec![0; data_size];
    for _ in 0..samples {
        let sample = match wav.bits_per_sample {
            8 => {
                eprintln!("quality too low");
                return Ok(()); // TODO: error out, create custom error kind and messages
            }
            16 => reader.read_le_i16()? as i32, //cast all to i32 for now
            24 => reader.read_le_i24()? as i32,
            32 => reader.read_le_i32()? as i32,
            _ => {
                eprintln!("Not supported");
                return Ok(()); // TODO: error out, create custom error kind and messages
            }
        };

        data.push(sample);
    }

    println!("Data: {:?}", data);
    Ok(())
}
