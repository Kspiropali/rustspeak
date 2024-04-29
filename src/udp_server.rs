use std::net::UdpSocket;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // init default host
    let host = cpal::default_host();

    // get output device and config
    let input_device = host
        .input_devices()
        .expect("No input device available")
        .next()
        .expect("No input device available");
    let supported_input_config = input_device
        .default_input_config()
        .expect("No default input config available")
        .config();

    // create udp socket at random unused port
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");

    // Create a mutex-protected buffer to store input data, TODO: thread it later
    let input_buffer: Arc<Mutex<Vec<f32>>> =
        Arc::new(Mutex::new(vec![0.0; supported_input_config.sample_rate.0 as usize]));

    // Clone input_buffer for use in the input thread safely
    let input_buffer_clone = input_buffer.clone();

    // Build input stream from microphone
    let input_stream = input_device
        .build_input_stream(
            &supported_input_config,
            move |data, _: &_| {
                // Store input data in the input buffer
                let mut input_buffer = input_buffer_clone.lock().unwrap();
                *input_buffer = data.to_vec();

                // Convert input data to bytes
                let bytes: Vec<u8> = input_buffer
                    .iter()
                    .flat_map(|&sample| sample.to_ne_bytes().to_vec())
                    .collect();

                // send mic data to udp client socket address
                socket
                    .send_to(&bytes, "192.168.1.42:54321")
                    .expect("Failed to send UDP packet");
            },
            move |err| {
                // React to errors here
                eprintln!("An error occurred on input stream: {}", err);
            },
            None, // No timeout
        )
        .expect("Failed to build input stream");

    // Start the input stream
    input_stream.play().expect("Failed to play input stream");

    // Keep the main thread alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}