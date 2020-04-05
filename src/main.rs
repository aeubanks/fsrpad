use gnuplot::*;
use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::{thread, time};

const ADDR: u16 = 0x48;

fn plot(times: &Vec<u64>, vals: &Vec<u16>) {
    let mut fg = Figure::new();
    fg.axes2d()
        .set_x_label("time", &[])
        .set_y_label("reading", &[])
        .lines(times, vals, &[]);
    fg.save_to_png("/tmp/readings.png", 640, 480).unwrap();
}

fn main() -> Result<(), LinuxI2CError> {
    let mut dev = LinuxI2CDevice::new("/dev/i2c-1", ADDR)?;

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
    //
    // bit order is weird: [7-0,15-8]
    dev.smbus_write_word_data(1, 0b111_00000__0_100_000_0)?;

    let mut times = Vec::new();
    let mut vals = Vec::new();

    let iterations = 500;
    let rate = time::Duration::from_millis(10);

    let start = time::Instant::now();

    for _ in 0..iterations {
        let response = dev.smbus_read_word_data(0).unwrap();
        times.push(start.elapsed().as_millis() as u64);
        vals.push(response);

        println!("Reading: {:?}", response);
        thread::sleep(rate);
    }

    plot(&times, &vals);

    Ok(())
}
