use std::net::UdpSocket;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // init default host
    let host = cpal::default_host();

    // get output device and config
    let output_device = host
        .output_devices()
        .expect("No output device available")
        .next()
        .expect("No output device available");
    let supported_output_config = output_device
        .default_output_config()
        .expect("No default output config available")
        .config();

    let socket = UdpSocket::bind("0.0.0.0:54321").expect("Failed to bind UDP socket");

    // socket -> output stream -> speaker
    let output_stream = output_device
        .build_output_stream(
            &supported_output_config,
            move |output_data, _: &_| {
                // find out how many samples we can receive from socket
                let mut recv_buffer = [0; 44100];
                match socket.recv(&mut recv_buffer) {
                    Ok(bytes_received) => {
                        let samples_received = bytes_received / std::mem::size_of::<f32>();
                        let samples = match samples_received {
                            0 => &[],
                            _ => {
                                let ptr = recv_buffer.as_ptr() as *const f32;
                                create_slice_from_raw_ptr(ptr, samples_received)
                            }
                        };
                        let len = output_data.len().min(samples.len());
                        output_data[..len].copy_from_slice(&samples[..len]);
                    }
                    Err(err) => {
                        eprintln!("Failed to receive UDP packet: {}", err);
                    }
                }
            },
            move |err| {
                eprintln!("An error occurred on output stream: {}", err);
            },
            None,
        )
        .expect("Failed to build output stream");

    // start output stream if not already started
    output_stream.play().expect("Failed to play output stream");

    // keep the output stream running until ctrl-c
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

fn create_slice_from_raw_ptr(ptr: *const f32, len: usize) -> &'static [f32] {
    unsafe {
        std::slice::from_raw_parts(ptr, len)
    }
}