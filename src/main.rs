use plotters::prelude::*;
use rustfft::num_complex::{Complex64, ComplexFloat};
use std::f64::consts::PI;
use std::{fs::File, io::BufReader};

use crate::audio::WavReader;
use crate::complex::Complex;
use crate::error::Result;

mod audio;
mod complex;
mod error;

const PATH: &str = "/Users/nikolai369/Downloads/dj co.kr & h4rdy - ill e sam sa (vip).wav";
const WINDOW_SIZE: usize = 1024;

fn dft(samples: &[i32]) -> Vec<Complex> {
    let n = samples.len();
    let theta = -2.0 * PI / n as f64;
    // r = 1 and theta = -2.0 * PI / n gives as the unit circle in the complex plain which is equal to e^-2PIi/N
    // this will help us wrap our signal around the circle in different frequencies and find the ones that oscilate at the same rate in our input signal
    let twiddle = Complex::from_polar(1.0, theta);

    let mut result: Vec<Complex> = Vec::with_capacity(n);
    for f in 0..n {
        let mut sum = Complex::new(0.0, 0.0);
        for n in 0..n {
            let sample = Complex::new(samples[n] as f64, 0.0);
            let angle = twiddle.powi((n * f) as i32);
            sum += sample * angle;
        }

        result.push(sum);
    }

    result
} // TODO: use only i32 values for now

fn plot_dft_magnitude(
    output_path: &str,
    dft_result: Vec<f64>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_mag = dft_result.iter().map(|c| c.clone()).fold(0.0, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("DFT Magnitude Spectrum", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..dft_result.len(), 0.0..max_mag)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        dft_result.iter().enumerate().map(|(i, c)| (i, c.clone())),
        &BLUE,
    ))?;

    root.present()?;
    Ok(())
}

// Frequency resolution
// For a 44100 sample rate file and 1024 samples per window, frequency_resolution = 44100 / 1024 ~ 21.5Hz
// Which means each freq bin coresponds to a 21.5Hz range
// For 1024 samples in a window there are 512 bins since n of samples give n/2 bins
// Larger window size gives us a greater frequency resolution
//
// Time resoulution
// For a 44100 sample rate file and 1024 samples per window, time_resolution = 1024 / 44100 ~ 23ms
// Which means we get an update every 23ms
// Smaller window size gives us greater time resolution
//
// Greater time resolution gives us more updates per second hence it is better for fast changing audio like speach or drums
// Greater frequency resolution gives a more precise frequency response for better pitch accuracy and slow signals
fn main() -> Result<()> {
    let file = File::open(PATH)?;
    let reader = BufReader::new(file);

    // Reads all headers unil the actual audio data
    let mut wav_reader = WavReader::new(reader)?;

    // Read vector of i32 samples
    let data = wav_reader.samples()?;

    // Testing and comparing my naive dft vs rustfft
    let mut windows = data.windows(WINDOW_SIZE);
    let target_window = windows.nth_back(3000).unwrap();
    let result = dft(target_window);
    let mut planner: rustfft::FftPlanner<f64> = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(WINDOW_SIZE);
    let mut buff = Vec::with_capacity(target_window.len());
    for sample in target_window {
        buff.push(Complex64 {
            re: *sample as f64,
            im: 0.0,
        })
    }

    fft.process(&mut buff);

    match (
        plot_dft_magnitude(
            "./dft_plot.png",
            result
                .iter()
                .take(result.len() / 2)
                .map(|x| x.abs())
                .collect(),
        ),
        plot_dft_magnitude(
            "./fft_plot.png",
            buff.iter().take(buff.len() / 2).map(|x| x.abs()).collect(),
        ),
    ) {
        _ => return Ok(()),
    }

    // Transform the amplitude data in time into frequency spectrum sices

    // Analyze peaks

    // Hash peaks
}
