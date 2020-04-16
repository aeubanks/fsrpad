use gnuplot::*;
use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::{path::PathBuf, thread, time};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "fsrpad")]
struct Opt {
    /// I2C interface
    #[structopt(long, default_value = "/dev/i2c-1")]
    i2c_interface: String,

    /// I2C address
    #[structopt(long, default_value = "72")]
    i2c_address: u16,

    /// Print readings
    #[structopt(short, long)]
    verbose: bool,

    /// Iterations until exit
    #[structopt(short, long)]
    iterations: Option<u64>,

    /// Time between readings
    #[structopt(short, long, default_value = "10")]
    period: u64,

    /// File to plot graph
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Quit when there is an i2c error
    #[structopt(short, long)]
    quit_on_error: bool,
}

fn plot<Tx: DataType, X: IntoIterator<Item = Tx>, Ty: DataType, Y: IntoIterator<Item = Ty>>(
    path: &PathBuf,
    times: X,
    vals: Y,
) {
    let mut fg = Figure::new();
    fg.axes2d()
        .set_x_label("time", &[])
        .set_y_label("reading", &[])
        .lines(times, vals, &[]);
    fg.save_to_png(path, 1280, 720).unwrap();
}

fn read_sensor(dev: &mut LinuxI2CDevice, quit_on_error: bool) -> Option<i16> {
        let res = dev.smbus_read_word_data(0);
        let val = if quit_on_error {
            res.unwrap()
        } else {
            match res {
                Ok(v) => v,
                Err(_) => return None,
            }
        };
        Some(u16::from_le(val) as i16)
}

fn main() -> Result<(), LinuxI2CError> {
    let opts = Opt::from_args();

    let mut dev = LinuxI2CDevice::new(opts.i2c_interface, opts.i2c_address)?;

    // http://www.ti.com/lit/ds/symlink/ads1114.pdf
    // [15]: 0 (only useful when sleeping, which we don't use)
    // [14-12]: configuring A0-A3, 000 for measuring difference between A0 and A1
    // [11-9]:
    //   000: +/- 6.144V
    //   001: +/- 4.096V
    //   010: +/- 2.048V
    //   011: +/- 1.024V
    //   100: +/- 0.512V
    //   101: +/- 0.256V
    //   110: +/- 0.256V
    //   111: +/- 0.256V
    // [8]:
    //   0: continuous version mode
    //   1: single-shot then sleep mode
    // [7-5]: data rate (per second)
    //   000: 8
    //   001: 16
    //   010: 32
    //   011: 64
    //   100: 128
    //   101: 250
    //   110: 475
    //   111: 860
    // [4-0]: comparator stuff (don't care)
    let config: u16 = 0b0_100_000_0__111_00000;
    dev.smbus_write_word_data(1, config.to_le())?;

    let mut times = Vec::new();
    let mut vals = Vec::new();

    let iterations = opts.iterations.unwrap_or(std::u64::MAX);
    let period = time::Duration::from_millis(opts.period);

    let start = time::Instant::now();

    for _ in 0..iterations {
        thread::sleep(period);
        let reading = match read_sensor(&mut dev, opts.quit_on_error) {
            Some(r) => r,
            None => continue,
        };
        times.push(start.elapsed().as_millis() as u64);
        vals.push(reading);

        if opts.verbose {
            println!("Reading: {:?}", reading);
        }
    }

    if let Some(plot_path) = opts.output {
        plot(&plot_path, &times, &vals);
    }

    Ok(())
}
