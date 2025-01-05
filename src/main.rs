use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use std::error::Error;
use std::{thread, time::Duration};

const DEFAULT_I2C_PATH: &str = "/dev/i2c-1";

const SENSOR_ADDR: u16 = 0x36;

// Register addresses
const STATUS_BASE: u8 = 0x00;
const STATUS_TEMP: u8 = 0x04;
const TOUCH_BASE: u8 = 0x0F;
const TOUCH_READ: u8 = 0x10;

// SAMD10 temperature sensor returns raw 32-bit fixed-point values
// where each unit represents 1/65536 (≈0.00001525878) degrees Celsius.
const TEMP_CONVERSION_FACTOR: f32 = 0.00001525878;

struct SoilSensor<T: I2CDevice> {
    i2c: T,
}

impl<T> SoilSensor<T>
where
    T: I2CDevice,
{
    pub fn new(i2c: T) -> Result<Self, T::Error> {
        Ok(SoilSensor { i2c })
    }

    /// Returns the read temperature of this [`SoilSensor<T>`] in degrees Celsius (°C).
    /// The ambient temperature comes from the internal temperature sensor on the microcontroller,
    /// it's not high precision, maybe good to + or - 2 degrees Celsius.
    ///
    /// # Errors
    ///
    /// This function will return an error if the sensor fails to provide a value.
    pub fn read_temperature(&mut self) -> Result<f32, T::Error> {
        // Write the correct status registers for temperature
        let command = [STATUS_BASE, STATUS_TEMP];
        self.i2c.write(&command)?;

        // Wait for conversion
        thread::sleep(Duration::from_millis(5));

        // Read 4 bytes of temperature data
        let mut buf = [0u8; 4];
        self.i2c.read(&mut buf)?;

        // Apply mask to first byte as per Python implementation
        buf[0] &= 0x3F;

        // Convert to u32 using from_be_bytes and apply conversion factor
        let raw_temp = u32::from_be_bytes(buf);
        let temp = raw_temp as f32 * TEMP_CONVERSION_FACTOR;

        Ok(temp)
    }

    /// Returns the read moisture of this [`SoilSensor<T>`]. This value
    /// ranges from 200 (very dry) to 2000 (very wet).
    ///
    /// # Errors
    ///
    /// This function will return an error if the sensor fails to provide a value.
    pub fn read_moisture(&mut self) -> Result<u16, T::Error> {
        // Write both the base register and read command
        let command = [TOUCH_BASE, TOUCH_READ];
        self.i2c.write(&command)?;

        // Wait for conversion
        thread::sleep(Duration::from_millis(5));

        // Read 2 bytes of moisture data
        let mut buf = [0u8; 2];
        self.i2c.read(&mut buf)?;

        // Convert to moisture value using from_be_bytes
        let moisture = u16::from_be_bytes(buf);

        Ok(moisture)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let default_path = DEFAULT_I2C_PATH.to_string();
    let i2c_path = args.get(1).unwrap_or(&default_path).as_str();

    let i2c = LinuxI2CDevice::new(i2c_path, SENSOR_ADDR)?;
    let mut sensor = SoilSensor::new(i2c)?;

    println!("Starting soil sensor readings...");

    loop {
        match sensor.read_temperature() {
            Ok(temp) => println!("Temperature: {:.2}°C", temp),
            Err(e) => eprintln!("Error reading temperature: {}", e),
        }

        match sensor.read_moisture() {
            Ok(moisture) => println!("Moisture: {} (200 - 2000)", moisture),
            Err(e) => eprintln!("Error reading moisture: {}", e),
        }

        println!("---");
        thread::sleep(Duration::from_secs(1));
    }
}
