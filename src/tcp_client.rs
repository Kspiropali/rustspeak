use std::io::Read;
use std::net::TcpStream;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // Initialize the default host
    let host = cpal::default_host();

    // Get the default output device and its supported output config
    let output_device = host
        .output_devices()
        .expect("No output device available")
        .next()
        .expect("No output device available");
    let supported_output_config = output_device
        .default_output_config()
        .expect("No default output config available")
        .config();

    // Connect to the server and check for errors
    let mut stream = TcpStream::connect("0.0.0.0:54321").expect("Failed to connect to server");

    // Build output stream to speaker
    let output_stream = output_device
        .build_output_stream(
            &supported_output_config,
            move |output_data, _: &_| {
                // Read data from the server
                let mut recv_buffer = [0; 44100];
                match stream.read(&mut recv_buffer) {
                    Ok(bytes_received) => {
                        let samples_received = bytes_received / std::mem::size_of::<f32>();
                        let samples = match samples_received {
                            0 => &[],
                            _ => {
                                let ptr = recv_buffer.as_ptr() as *const f32;
                                unsafe { std::slice::from_raw_parts(ptr, samples_received) }
                            }
                        };
                        let len = output_data.len().min(samples.len());
                        output_data[..len].copy_from_slice(&samples[..len]);
                    }
                    Err(err) => {
                        eprintln!("Failed to receive data from server: {}", err);
                    }
                }

            },
            move |err| {
                eprintln!("An error occurred on output stream: {}", err);
            },
            None,
        )
        .expect("Failed to build output stream");

    // Start the output stream
    output_stream.play().expect("Failed to play output stream");

    // Keep the main thread alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}