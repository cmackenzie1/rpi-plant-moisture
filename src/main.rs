use std::error::Error;
use std::{thread, time::Duration};

use axum::routing::get;
use axum::Router;
use clap::Parser;
use hyper::header;
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use lazy_static::lazy_static;
use prometheus::{opts, register_gauge, Encoder, Gauge, TextEncoder};

const SENSOR_ADDR: u16 = 0x36;

// Register addresses
const STATUS_BASE: u8 = 0x00;
const STATUS_TEMP: u8 = 0x04;
const TOUCH_BASE: u8 = 0x0F;
const TOUCH_READ: u8 = 0x10;

// SAMD10 temperature sensor returns raw 32-bit fixed-point values
// where each unit represents 1/65536 (≈0.00001525878) degrees Celsius.
const TEMP_CONVERSION_FACTOR: f32 = 0.00001525878;

lazy_static! {
    static ref TEMPERATURE_GAUGE: Gauge =
        register_gauge!(opts!("temperature", "Temperature in degrees celsius")).unwrap();
    static ref MOISTURE_GAUGE: Gauge = register_gauge!(opts!(
        "moisture",
        "Soil moisture ranging from 200 (very dry) to 2000 (very wet)"
    ))
    .unwrap();
}

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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "/dev/i2c-1")]
    device: String,
    #[arg(short, long, default_value = "0.0.0.0:3000")]
    metrics_addr: String,
    #[arg(short, long)]
    quiet: bool,
    #[arg(short, long, default_value_t = 60)]
    interval_seconds: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let i2c = LinuxI2CDevice::new(args.device, SENSOR_ADDR)?;
    let mut sensor = SoilSensor::new(i2c)?;

    let app = Router::new().route(
        "/metrics",
        get(|| async {
            let encoder = TextEncoder::new();
            let mut buffer = vec![];
            let metrics = prometheus::gather();
            encoder.encode(&metrics, &mut buffer).unwrap();

            (
                [(header::CONTENT_TYPE, "text/plain")],
                String::from_utf8(buffer).unwrap(),
            )
        }),
    );

    tokio::spawn(async move {
        let addr = args.metrics_addr;
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        println!("Prometheus metrics are available at http://{addr}/metrics",);
        axum::serve(listener, app).await.unwrap();
    });

    println!("Starting soil sensor readings...");

    loop {
        match sensor.read_temperature() {
            Ok(temp) => {
                TEMPERATURE_GAUGE.set(temp.into());
                if !args.quiet {
                    println!("Temperature: {:.2}°C", temp)
                }
            }
            Err(e) => eprintln!("Error reading temperature: {}", e),
        }

        match sensor.read_moisture() {
            Ok(moisture) => {
                MOISTURE_GAUGE.set(moisture.into());
                if !args.quiet {
                    println!("Moisture: {} (200 - 2000)", moisture)
                }
            }
            Err(e) => eprintln!("Error reading moisture: {}", e),
        }

        if !args.quiet {
            println!("---");
        }
        thread::sleep(Duration::from_secs(args.interval_seconds));
    }
}
