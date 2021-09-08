use dht_hal::{Dht22, Reading};
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::iso_8859_7::FONT_9X18;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, Rectangle};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    text::Text,
};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::prelude::_embedded_hal_blocking_i2c_Write;
use gpio_cdev::{Chip, EventRequestFlags, EventType, LineRequestFlags};
use linux_embedded_hal::i2cdev::core::I2CDevice;
use linux_embedded_hal::i2cdev::linux::LinuxI2CError;
use rusqlite::{Connection, params};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use linux_embedded_hal::{Delay, I2cdev};
use machine_ip;
use std::intrinsics::transmute;
use std::{env, mem, time};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

static IMG_DATA: &[u8; 512] = include_bytes!("../rust.raw");


const CCS811_ADDR: u8 = 0x5A; // or 0x5B

const CCS811_STATUS: u8 = 0x00;
const CCS811_MEAS_MODE: u8 = 0x01;
const CCS811_ALG_RESULT_DATA: u8 = 0x02;
const CCS811_RAW_DATA: u8 = 0x03;
const CCS811_ENV_DATA: u8 = 0x05;
const CCS811_NTC: u8 = 0x06;
const CCS811_THRESHOLDS: u8 = 0x10;
const CCS811_BASELINE: u8 = 0x11;
const CCS811_HW_ID: u8 = 0x20;
const CCS811_HW_VERSION: u8 = 0x21;
const CCS811_FW_BOOT_VERSION: u8 = 0x23;
const CCS811_FW_APP_VERSION: u8 = 0x24;
const CCS811_ERROR_ID: u8 = 0xE0;
const CCS811_APP_START: u8 = 0xF4;
const CCS811_SW_RESET: u8 = 0xFF;

struct CCS811 {
    i2c: I2cdev,
    addr: u8
}

impl CCS811 {
    fn new(mut i2c: I2cdev, addr: u8) -> Self {
        i2c.set_slave_address(addr as u16).unwrap();
        Self {
            i2c,
            addr
        }
    }

    fn check_for_error(&mut self) -> Option<u8> {
        let x = self.i2c.smbus_read_byte_data(CCS811_STATUS).unwrap();
        if (x & 1) != 0 {
            let err_code = self.i2c.smbus_read_byte_data(CCS811_ERROR_ID).unwrap();
            Some(err_code)
        } else {
            None
        }
    }

    fn hardware_id(&mut self) -> u8 {
        self.i2c.smbus_read_byte_data(CCS811_HW_ID).unwrap()
    }

    fn app_valid(&mut self) -> bool {
        let x = self.i2c.smbus_read_byte_data(CCS811_STATUS).unwrap();
        x & 1 << 4 != 0
    }

    fn set_drive_mode(&mut self, mode: CCS811DriveMode) -> Result<(), Option<LinuxI2CError>> {
        self.i2c.smbus_write_byte(CCS811_APP_START).map_err(Some)?;
        if let Some(x) = self.check_for_error() {
            println!("error ignored {:b}", x);
        }
        let mut setting = self.i2c.smbus_read_byte_data(CCS811_MEAS_MODE).map_err(Some)?;
        setting &= !(0b00000111 << 4);
        setting |= (mode as u8) << 4;
        self.i2c.smbus_write_byte_data(CCS811_MEAS_MODE, setting).map_err(Some)?;
        Ok(())
    }

    fn get_baseline(&mut self) -> u16 {
        let x = self.i2c.smbus_read_i2c_block_data(CCS811_BASELINE, 2).unwrap();
        ((x[0] as u16) << 8) | (x[1] as u16)
    }

    /// Returns (eCO2, tVOC)
    fn get_reading(&mut self) -> (u16, u16) {
        let x = self.i2c.smbus_read_i2c_block_data(CCS811_ALG_RESULT_DATA, 4).unwrap();
        (((x[0] as u16) << 8) | (x[1] as u16), ((x[2] as u16) << 8) | (x[3] as u16))
    }
}

enum CCS811DriveMode {
    Idle = 0,
    EverySecond = 1,
    Every10Seconds = 2,
    Every60Seconds = 3,
    /// Note the English manual states this is calculated every 10 ms!
    Every250Milliseconds = 4,
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        panic!("missing argument: database path");
    }
    let database = Connection::open(&args[1]).expect("failed to open database");
    database.execute("
        CREATE TABLE IF NOT EXISTS sensor_readings(
            time INTEGER PRIMARY KEY,
            humidity INTEGER NOT NULL,
            celsius INTEGER NOT NULL
        )", []).unwrap();

    /*
    let mut ccs = CCS811::new(i2c, CCS811_ADDR);
    println!("HW ID, should be 0x81 {:x}", ccs.hardware_id());
    println!("Error code, should be None: {:?}", ccs.check_for_error());
    println!("app valid = {:?}", ccs.app_valid());
    println!("baseline = {:x}", ccs.get_baseline());
    println!("reading {:?}", ccs.get_reading());
    println!("setting drive mode to 1: {:?}", ccs.set_drive_mode(CCS811DriveMode::EverySecond));
    */
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    let line = chip.get_line(26).unwrap();
    for _attempt in 0..5 {
        let time = std::time::SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        if let Ok((rh, temp)) = raspi_oled::am2302_reading(&line) {
            database.execute("INSERT INTO sensor_readings (time, humidity, celsius) VALUES (?1, ?2, ?3)", params![time, rh, temp]).unwrap();
            display_on_ssd1306(rh, temp);
            break;
        }
    }
}

    
fn display_on_ssd1306(rh: u16, temp: u16) {
    let i2c = I2cdev::new("/dev/i2c-1").unwrap();
    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();

    disp.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X18)
        .text_color(BinaryColor::On)
        .build();

    let text = format!("{}.{}% {}.{}°C", rh / 10, rh % 10, temp / 10, temp % 10);
    Text::new(&text, Point::new(0, 10), text_style)
            .draw(&mut disp)
            .unwrap();
    disp.flush().unwrap();
    /*
    sleep(Duration::from_secs(2));
    disp.clear();

    let base_y = 0.0;
    let max_dy = 32.0;
    let mut tick = 0;
    loop {
        let y = if tick % 32 < 16 {
            base_y + (tick % 16) as f32 / 16.0 * max_dy
        } else {
            base_y + max_dy - (tick % 16) as f32 / 16.0 * max_dy
        } as i32;
        tick += 1;
        Line::new(Point::new(8, y + 16), Point::new(8 + 16, y + 16))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut disp).unwrap();
        Line::new(Point::new(8, y + 16), Point::new(8 + 8, y))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut disp).unwrap();
        
        Line::new(Point::new(8 + 16, y + 16), Point::new(8 + 8, y))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut disp).unwrap();

        Rectangle::new(Point::new(48, y), Size::new(16, 16))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut disp).unwrap();
            

        Circle::new(Point::new(88, y), 16)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut disp).unwrap();

        /*
        Text::new(&format!("Hello from frame {}", tick), Point::new(0, 56), text_style)
            .draw(&mut disp)
            .unwrap();
        */
        disp.flush().unwrap();

        sleep(Duration::from_millis(10));

        disp.clear();

        /*
        let im: ImageRaw<BinaryColor> = ImageRaw::new(IMG_DATA, 64);
        let img = Image::new(&im, Point::new(32, 0));
        img.draw(&mut disp).unwrap();
        disp.flush().unwrap();

        sleep(Duration::from_secs(2));
        disp.clear();
        */
    }
    */
}

struct LineWrapper(gpio_cdev::Line);

impl InputPin for LineWrapper {
    type Error = gpio_cdev::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        let handle = self.0.request(LineRequestFlags::INPUT, 0, "rust-line-wrapper")?;
        Ok(handle.get_value()? == 1)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        let handle = self.0.request(LineRequestFlags::INPUT, 0, "rust-line-wrapper")?;
        Ok(handle.get_value()? == 0)
    }
}

impl OutputPin for LineWrapper {
    type Error = gpio_cdev::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.request(LineRequestFlags::OUTPUT, 1, "rust-line-wrapper")?.set_value(0)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.request(LineRequestFlags::OUTPUT, 1, "rust-line-wrapper")?.set_value(1)
    }
}
