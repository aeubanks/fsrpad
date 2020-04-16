use i2cdev::core::*;
use i2cdev::linux::LinuxI2CDevice;
pub use i2cdev::linux::LinuxI2CError;

pub use LinuxI2CError as I2CError;

pub struct I2CDev {
    dev: LinuxI2CDevice,
}

impl I2CDev {
    pub fn new(interface: &str, address: u16) -> Result<Self, I2CError> {
        let dev = LinuxI2CDevice::new(interface, address)?;
        Ok(Self { dev })
    }
}

impl I2CDev {
    pub fn write_u16(&mut self, reg: u8, val: u16) -> Result<(), I2CError> {
        self.dev.smbus_write_word_data(reg, val.swap_bytes())
    }

    pub fn read_u16(&mut self, reg: u8) -> Result<u16, I2CError> {
        Ok(self.dev.smbus_read_word_data(reg)?.swap_bytes())
    }

    pub fn read_i16(&mut self, reg: u8) -> Result<i16, I2CError> {
        self.read_u16(reg).map(|v| v as i16)
    }
}
