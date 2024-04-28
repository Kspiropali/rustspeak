use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // Initialize the default host
    let host = cpal::default_host();
    // default output device and config
    let output_device = host
        .output_devices()
        .expect("No output device available")
        .next()
        .expect("No output device available");
    let supported_output_config = output_device
        .default_output_config()
        .expect("No default output config available")
        .config();

    // Get the default input device and its supported input config
    let input_device = host
        .input_devices()
        .expect("No input device available")
        .next()
        .expect("No input device available");
    let supported_input_config = input_device
        .default_input_config()
        .expect("No default input config available")
        .config();

    // Create a mutex-protected buffer to store input data
    let input_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(vec![0.0; supported_input_config.sample_rate.0 as usize]));

    // Clone input_buffer for use in both input and output threads
    let input_buffer_clone = input_buffer.clone();

    let input_stream = input_device
        .build_input_stream(
            &supported_input_config,
            move |data, _: &_| {
                // Store input data in the input buffer
                let mut input_buffer = input_buffer_clone.lock().unwrap();
                *input_buffer = data.to_vec();
            },
            move |err| {
                eprintln!("An error occurred on input stream: {}", err);
            },
            None,
        )
        .expect("Failed to build input stream");

    // Start the input stream
    input_stream.play().expect("Failed to play input stream");

    // Build output stream to speaker
    let output_stream = output_device
        .build_output_stream(
            &supported_output_config,
            move |output_data, _: &_| {
                // Copy input data from the input buffer to the output buffer
                let input_buffer = input_buffer.lock().unwrap();
                let len = input_buffer.len().min(output_data.len());
                output_data[..len].copy_from_slice(&input_buffer[..len]);
            },
            move |err| {
                eprintln!("An error occurred on output stream: {}", err);
            },
            None,
        )
        .expect("Failed to build output stream");

    // Start the output stream
    output_stream.play().expect("Failed to play output stream");

    loop {}
}
