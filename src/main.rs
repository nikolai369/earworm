use plotters::prelude::*;
use rustfft::num_complex::{Complex64, ComplexFloat};
use std::f64::consts::{self, PI};
use std::{fs::File, io::BufReader};

use crate::audio::WavReader;
use crate::complex::Complex;
use crate::error::Result;

mod audio;
mod complex;
mod error;

const PATH: &str = "/Users/nikolai369/Downloads/dj co.kr & h4rdy - ill e sam sa (vip).wav";
const WINDOW_SIZE: usize = 1024;

fn dft(samples: &[f64]) -> Vec<Complex> {
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
} // TODO: use only f64 values for now

fn fft(samples: &[f64]) -> Vec<Complex64> {
    let mut planner: rustfft::FftPlanner<f64> = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(WINDOW_SIZE);
    let mut buff: Vec<Complex64> = samples
        .iter()
        .copied()
        .map(|x| Complex64 { re: x, im: 0.0 })
        .collect();
    fft.process(&mut buff);

    buff
}

// Naive linear slope window function
fn window_function(window_size: usize) -> Vec<f64> {
    let slope_size = (window_size * 5) / 100;

    // Need to apply a gradient slope on begining and end of the window by multiplying the by te generated window function

    let mut win_fn: Vec<f64> = Vec::with_capacity(window_size);
    for i in 0..window_size {
        if i < slope_size {
            win_fn.push(i as f64 / slope_size as f64);
            continue;
        }

        if window_size - i < slope_size {
            let temp = window_size as f64 - i as f64;
            win_fn.push(temp / slope_size as f64);
            continue;
        }

        win_fn.push(1.0);
    }

    win_fn
}

fn apply_win_fn(window: &[f64], win_fn: &Vec<f64>) -> Vec<f64> {
    let mut result: Vec<f64> = Vec::with_capacity(window.len());

    for i in 0..window.len() {
        result.push(window[i] as f64 * win_fn[i]);
    }

    result
}

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
// For a 44100 sample rate file and 1024 samples per window, frequency_resolution = 44100 / 1024 ~ 43Hz
// Which means each freq bin coresponds to a 43Hz range
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
    let data = wav_reader.mono()?;

    // How to minimize spectral leakage??
    // Testing and comparing my naive dft vs rustfft
    let mut windows = data.windows(WINDOW_SIZE);
    let target_window = windows.nth(40000).unwrap();

    // Hann window creates a cosine curve starting and ending at zero and peeking at one in the middle
    let hann_window = (0..WINDOW_SIZE)
        .map(|i| 0.5 - 0.5 * ((consts::PI * i as f64 * 2.0) / WINDOW_SIZE as f64).cos())
        .collect();

    let naive_window = window_function(WINDOW_SIZE);
    let windowed = apply_win_fn(target_window, &hann_window);
    let naive_windowed = apply_win_fn(target_window, &naive_window);

    // Naive DFT
    let dft_result = dft(&windowed);

    let naive_window_result = dft(&naive_windowed);

    // FFT
    let fft_result = fft(&windowed);

    match (
        plot_dft_magnitude(
            "./naive_window_fn_dft.png",
            naive_window_result
                .iter()
                .take(naive_window_result.len() / 2)
                .map(|x| x.abs())
                .collect(),
        ),
        plot_dft_magnitude(
            "./dft_plot.png",
            dft_result
                .iter()
                .take(dft_result.len() / 2)
                .map(|x| x.abs())
                .collect(),
        ),
        plot_dft_magnitude(
            "./fft_plot.png",
            fft_result
                .iter()
                .take(fft_result.len() / 2)
                .map(|x| x.abs())
                .collect(),
        ),
    ) {
        _ => return Ok(()),
    }

    // Transform the amplitude data in time into frequency spectrum sices

    // Analyze peaks

    // Hash peaks
}
