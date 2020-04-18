use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "fsrpad-client")]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    file: PathBuf,

    #[structopt(short, long)]
    server: String,
}

fn main() {
    let opts = Opt::from_args();
    println!("{:?}", opts);
}
