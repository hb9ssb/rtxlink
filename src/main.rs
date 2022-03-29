use std::env;
use std::process;
use std::fs::{File, metadata};
use std::time::Duration;
use std::thread;
use thread_control::*;
use ymodem::xmodem;

const OUTPUT_PATH: &str = "./flash_dump.bin";

fn main() {

    let args: Vec<String> = env::args().collect();

    // Print usage information
    if args.len() < 3 {
        println!("rtxlink: OpenRTX Communication Protocol");
        println!("usage: {} COMMAND SERIALPORT", args[0]);
        println!("commands:");
        println!(" dump                       Read the device flash and save it to flash_dump.bin");
        println!(" flash                      Write an image to the device flash");
        process::exit(0);
    }

    let command = args[1].clone();
    let serial_port = args[2].clone();

    if command == "dump" { dump(serial_port); }
    else if command == "flash" { flash(serial_port); }
}

fn dump(serial_port: String) {
    let mut port = serialport::new(serial_port, 115_200)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open serial port");
    let mut output_file = File::create(OUTPUT_PATH)
        .expect("Failed to open output file");

    // Create xmodem with 1K blocks
    let mut xmodem = xmodem::Xmodem::new();
    xmodem.block_length = xmodem::BlockLength::OneK;

    // Workaround for missing handle.is_running()
    // https://github.com/DenisKolodin/thread-control
    let (flag, control) = make_pair();
    let handle = thread::spawn(move || {
        if !flag.alive() { return; }
        println!("XMODEM transfer started");
        xmodem.recv(&mut port, &mut output_file, xmodem::Checksum::CRC16)
              .expect("Failed to receive xmodem transfer");
        println!("XMODEM transfer finished");
    });

    // handle.is_running() is not available yet, use it when it is released
    // https://github.com/rust-lang/rust/issues/90470
    while !control.is_done() {
        let output_size = metadata(OUTPUT_PATH).unwrap().len();
        println!("{} size: {} Bytes", OUTPUT_PATH, output_size);
        thread::sleep(Duration::from_millis(1000));
    }
    // Wait for xmodem thread to finish
    handle.join().unwrap();
}

fn flash(serial_port: String) {
    let mut port = serialport::new(serial_port, 115_200)
        .timeout(Duration::from_secs(60))
        .open().expect("Failed to open serial port");
    let mut output_file = File::open(OUTPUT_PATH)
        .expect("Failed to open input file");

    // Create xmodem with 1K blocks
    let mut xmodem = xmodem::Xmodem::new();
    xmodem.block_length = xmodem::BlockLength::OneK;

    // Workaround for missing handle.is_running()
    // https://github.com/DenisKolodin/thread-control
    let (flag, control) = make_pair();
    let handle = thread::spawn(move || {
        if !flag.alive() { return; }
        println!("XMODEM transfer started");
        xmodem.send(&mut port, &mut output_file)
              .expect("Failed to send xmodem transfer");
        println!("XMODEM transfer finished");
    });

    thread::sleep(Duration::from_millis(500));
    println!("Press PTT on the radio to start XMODEM transfer");
    // Wait for xmodem thread to finish
    handle.join().unwrap();
}
