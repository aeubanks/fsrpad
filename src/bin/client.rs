use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "fsrpad-client")]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    file: PathBuf,

    #[structopt(short, long, default_value = "raspberrypi:8000")]
    server: String,

    #[structopt(short, long)]
    verbose: bool,

    #[structopt(short, long)]
    wait_duration: u64,

    #[structopt(long)]
    quit_on_file_error: bool,

    #[structopt(long)]
    quit_on_network_error: bool,
}

fn sextext_from_reading(reading: u8) -> u8 {
    ((reading + 0x10) & 0x3F) + 0x30
}

fn main() -> std::io::Result<()> {
    let opts = Opt::from_args();
    let mut stream = TcpStream::connect(opts.server).unwrap();
    let mut fd = std::fs::OpenOptions::new()
        .write(true)
        .open(opts.file)
        .unwrap();
    let wait_duration = std::time::Duration::from_millis(opts.wait_duration);
    loop {
        std::thread::sleep(wait_duration);
        let start = std::time::Instant::now();
        let ret = stream.write(&[0]);
        if opts.quit_on_network_error {
            let _ = ret.unwrap();
        }
        let mut res = [0];
        let ret2 = stream.read(&mut res);
        if opts.quit_on_network_error {
            let _ = ret2.unwrap();
        }
        let ret3 = fd.write(&[sextext_from_reading(res[0]), 0x0A]);
        if opts.quit_on_file_error {
            let _ = ret3.unwrap();
        }
        if opts.verbose {
            println!("{:07?} microseconds", start.elapsed().as_micros());
        }
    }
}
