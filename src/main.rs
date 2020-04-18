mod i2c;

use gnuplot::*;
use i2c::{I2CDev, I2CError};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI16, Ordering};
use std::thread;
use std::time;
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
    read_period: u64,

    /// File to plot graph
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Quit when there is an i2c error
    #[structopt(short, long)]
    quit_on_error: bool,

    /// Read all four sensors
    #[structopt(short, long)]
    all_sensors: bool,

    /// Number of iterations before reading stable value
    /// Ignored if --thresholds is set
    #[structopt(short, long, default_value = "100")]
    warmup_iterations: u64,

    /// Added to stable value to calculate threshold
    /// Ignored if --thresholds is set
    #[structopt(long, default_value = "1000")]
    warmup_threshold_offset: i32,

    /// Threshold for each sensor
    #[structopt(long)]
    threshold: Option<i32>,

    /// Port to listen on
    #[structopt(short, long)]
    port: Option<u16>,

    /// Offset from stable required to change state
    #[structopt(short, long, default_value = "10")]
    debounce: i32,
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

fn read_sensor(dev: &mut I2CDev, sensor_number: usize) -> Result<i16, I2CError> {
    if sensor_number >= 4 {
        panic!("sensor_number should be less than 4, got {}", sensor_number);
    }

    // http://www.ti.com/lit/ds/symlink/ads1114.pdf
    // [15]: 0 (only useful when sleeping, which we don't use)
    // [14-12]: configuring A0-A3, 100 for measuring A0
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
    let config: u16 = 0b0_100_001_0__111_00000 | ((sensor_number as u16) << 12);
    dev.write_u16(1, config)?;

    dev.read_i16(0)
}

static LATEST_VALUE: AtomicI16 = AtomicI16::new(0);

fn server(port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;

    for stream in listener.incoming() {
        let mut stream = stream?;
        stream.read(&mut [0; 1])?;
        let val = LATEST_VALUE.load(Ordering::Relaxed);
        stream.write(&val.to_be_bytes())?;
    }
    Ok(())
}

fn main() -> Result<(), I2CError> {
    let opts = Opt::from_args();

    if let Some(port) = opts.port {
        thread::spawn(move || server(port));
    }

    let mut dev = I2CDev::new(&opts.i2c_interface, opts.i2c_address)?;

    let mut times = Vec::new();
    let mut vals = Vec::new();

    let iterations = opts.iterations.unwrap_or(std::u64::MAX);
    let read_period = time::Duration::from_millis(opts.read_period);

    let start = time::Instant::now();
    let num_sensors = if opts.all_sensors { 4 } else { 1 };
    let mut thresholds = vec![opts.threshold; num_sensors as usize];
    let mut states = vec![false; num_sensors as usize];

    for i in 0..iterations {
        thread::sleep(read_period);
        if opts.verbose {
            println!();
        }
        for sensor_number in 0..num_sensors {
            let res = read_sensor(&mut dev, sensor_number);
            let reading = match res {
                Ok(r) => r,
                Err(e) => {
                    if opts.quit_on_error {
                        return Err(e);
                    } else {
                        continue;
                    }
                }
            };

            if sensor_number == 0 && opts.output != None {
                times.push(start.elapsed().as_millis() as u64);
                vals.push(reading);
            }

            if sensor_number == 0 {
                LATEST_VALUE.store(reading, Ordering::Relaxed);
            }

            if opts.verbose {
                println!("Sensor {}: {:?}", sensor_number, reading);
            }

            if let Some(threshold) = thresholds[sensor_number] {
                states[sensor_number] = (reading as i32)
                    > (threshold + opts.debounce * (if states[sensor_number] { -1 } else { 1 }));

                if opts.verbose {
                    println!("{}", if states[sensor_number] { "On" } else { "Off" });
                }
            } else if i == opts.warmup_iterations {
                thresholds[sensor_number] = Some(reading as i32 + opts.warmup_threshold_offset)
            }
        }
    }

    if let Some(plot_path) = opts.output {
        plot(&plot_path, &times, &vals);
    }

    Ok(())
}
