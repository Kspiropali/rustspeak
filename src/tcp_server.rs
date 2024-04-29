use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{io, thread};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // Initialize the default host
    let host = cpal::default_host();

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

    // Create input stream from microphone
    let input_stream = input_device
        .build_input_stream(
            &supported_input_config,
            move |data: &[f32], _: &_| {
                // Convert input data to bytes
                let bytes: Vec<u8> = data
                    .iter()
                    .flat_map(|&sample| sample.to_ne_bytes().to_vec())
                    .collect();

                // Send mic data to client
                if let Some(ref mut client_stream) = *CLIENT_STREAM.lock().unwrap() {
                    client_stream.write_all(&bytes);
                }
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

    // Start listening for incoming TCP connections
    let listener = TcpListener::bind("0.0.0.0:54321").expect("Failed to bind TCP listener");

    // Accept incoming connections and handle them
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New client connected");
                let client_stream = stream.try_clone().expect("Failed to clone client stream");
                // Store client stream in shared variable
                *CLIENT_STREAM.lock().unwrap() = Some(client_stream);

                // Handle client in a separate thread
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept incoming connection: {}", e);
            }
        }
    }
}

// Shared client stream wrapped in Arc and Mutex
lazy_static::lazy_static! {
    static ref CLIENT_STREAM: Arc<Mutex<Option<TcpStream>>> = Arc::new(Mutex::new(None));
}

// Function to handle client communication
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read_exact(&mut buffer) {
            // handle disconnection and print it
            Ok(_) => {
                println!("Received data from client: {:?}", &buffer);
            },
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof
            || e.kind() == io::ErrorKind::ConnectionReset => {
                println!("Client disconnected");
                break;
            },
            Err(e) => {
                println!("Failed to read from client: {}", e);
                break;
            }
        }
    }
}
