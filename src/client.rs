use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::net::UdpSocket;
use std::io::Error;

fn main() -> Result<(), Error> {
    println!("Hello, world!");
    // Create a UDP socket
    let socket = UdpSocket::bind("0.0.0.0:54321")?;

    // Initialize the default host
    let host = cpal::default_host();
    let mut buffer = [0; 1024];

    // Get the default output device and its supported output config
    let output_device = host
        .output_devices()
        .unwrap()
        .next()
        .expect("No output device available");
    let supported_output_config = output_device
        .default_output_config()
        .expect("No default output config available");

    // Build output stream to speaker
    let output_stream = output_device.build_output_stream(
        &supported_output_config.config(),
        move |output_data, _: &_| {

            match socket.recv_from(&mut buffer) {
                Ok((bytes_received, _)) => {
                    // Convert received bytes to audio samples and copy them to the output buffer
                    let samples_received = bytes_received / std::mem::size_of::<f32>();
                    let samples = unsafe {
                        let ptr = buffer.as_ptr() as *const f32;
                        std::slice::from_raw_parts(ptr, samples_received)
                    };
                    output_data.copy_from_slice(samples);
                }
                Err(err) => {
                    eprintln!("Failed to receive UDP packet: {}", err);
                },
            }
        },
        move |err| {
            // React to errors here
            eprintln!("An error occurred on input stream: {}", err);
        },
        None, // No timeout
    );

    // Start the output stream
    output_stream.unwrap().play().expect("TODO: panic message");

    // Keep the main thread alive
    loop {}
}
