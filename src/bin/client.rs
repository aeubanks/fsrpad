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
    loop {
        stream.write(&[0]).unwrap();
        let mut res = [0];
        stream.read(&mut res).unwrap();
        fd.write(&[sextext_from_reading(res[0]), 0x0A]).unwrap();
    }
}
