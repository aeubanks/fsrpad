use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::{thread, time};

const ADDR: u16 = 0x48;

fn main() -> Result<(), LinuxI2CError> {
    let mut dev = LinuxI2CDevice::new("/dev/i2c-1", ADDR)?;

    // http://www.ti.com/lit/ds/symlink/ads1114.pdf
    // [15]: 0 (only useful when sleeping, which we don't use)
    // [14-12]: configuring A0-A3, 000 for measuring A0
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
    dev.smbus_write_word_data(1, 0b0_100_000_0_111_00000).unwrap();

    //let mut vals = Vec::new();

    let iterations = 200;
    let rate = time::Duration::from_millis(10);

    for _ in 0..iterations {
        let response = dev.smbus_read_word_data(0).unwrap();
        //vals.push(response);

        println!("Reading: {:?}", response);
        thread::sleep(rate);
    }

    Ok(())
}
