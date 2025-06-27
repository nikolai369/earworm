# 👂🏻🐛 Earworm

A small experimental project to **learn Rust** and explore **audio fingerprinting** using **Fast Fourier Transform (FFT)**.

---

## 🚀 Goal

Build a basic system that can analyze audio files, extract their frequency features via FFT, and experiment with simple fingerprinting or matching techniques — all while getting comfortable with Rust’s syntax, tooling, and ecosystem.

---

## 🧠 What I'm Learning

- 📦 Rust fundamentals: ownership, lifetimes, modules, error handling
- 🔊 Audio decoding and signal processing
- ⚡ Fast Fourier Transform (FFT)
- 🧬 Audio fingerprinting concepts (Shazam-style matching)
- 🧰 Rust libraries like:
  - [`hound`](https://crates.io/crates/hound) – reading `.wav` files
  - [`rustfft`](https://crates.io/crates/rustfft) – FFT processing

---

## 📝 TODO

- [ ] ✅ Read and decode WAV file
  - Resources:
    - [Wave File Format](http://soundfile.sapp.org/doc/WaveFormat/)
    - [WAV File Explained (YouTube)](https://www.youtube.com/watch?v=udbA7u1zYfc)
- [ ] Extract audio samples
- [ ] Apply FFT to windowed frames
- [ ] Identify and visualize frequency peaks
- [ ] Implement basic audio fingerprinting logic
- [ ] Experiment with fingerprint matching
- [ ] Add CLI interface for fingerprinting and lookup
- [ ] Write tests for core logic
