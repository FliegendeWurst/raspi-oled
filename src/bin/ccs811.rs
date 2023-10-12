use linux_embedded_hal::i2cdev::core::I2CDevice;
use linux_embedded_hal::i2cdev::linux::LinuxI2CError;
use linux_embedded_hal::I2cdev;

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
	addr: u8,
}

impl CCS811 {
	fn new(mut i2c: I2cdev, addr: u8) -> Self {
		i2c.set_slave_address(addr as u16).unwrap();
		Self { i2c, addr }
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
		self.i2c
			.smbus_write_byte_data(CCS811_MEAS_MODE, setting)
			.map_err(Some)?;
		Ok(())
	}

	fn get_baseline(&mut self) -> u16 {
		let x = self.i2c.smbus_read_i2c_block_data(CCS811_BASELINE, 2).unwrap();
		((x[0] as u16) << 8) | (x[1] as u16)
	}

	/// Returns (eCO2, tVOC)
	fn get_reading(&mut self) -> (u16, u16) {
		let x = self.i2c.smbus_read_i2c_block_data(CCS811_ALG_RESULT_DATA, 4).unwrap();
		(
			((x[0] as u16) << 8) | (x[1] as u16),
			((x[2] as u16) << 8) | (x[3] as u16),
		)
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
	let i2c = I2cdev::new("/dev/i2c-1").unwrap();
	let mut ccs = CCS811::new(i2c, CCS811_ADDR);
	println!("HW ID, should be 0x81 {:x}", ccs.hardware_id());
	println!("Error code, should be None: {:?}", ccs.check_for_error());
	println!("app valid = {:?}", ccs.app_valid());
	println!("baseline = {:x}", ccs.get_baseline());
	println!("reading {:?}", ccs.get_reading());
	println!(
		"setting drive mode to 1: {:?}",
		ccs.set_drive_mode(CCS811DriveMode::EverySecond)
	);

	/*
	let args = std::env::args().collect::<Vec<_>>();
	if args.len() < 2 {
		panic!("missing argument: database path");
	}

	let mut disp = FrameOutput {
		buffer: ImageBuffer::new(128, 64),
	};

	let database = Connection::open(&args[1]).expect("failed to open database");

	let mut query = database
		.prepare("SELECT celsius FROM sensor_readings ORDER BY sensor_readings.time DESC LIMIT 288")
		.unwrap();
	let mut temps: Vec<i32> = query
		.query_map([], |r| Ok(r.get(0)))
		.unwrap()
		.map(Result::unwrap)
		.map(Result::unwrap)
		.collect();
	let mut global_min = 1000;
	let mut global_max = 0;
	let mut vals: Vec<(i32, i32)> = vec![];
	for hour in temps.chunks_mut(6) {
		hour.sort();
		let min = hour[1];
		let mut max = hour[hour.len() - 2];
		println!("min {} max {}", min, max);
		// sanity check value
		if max > 300 {
			if vals.is_empty() {
				max = min;
			} else {
				max = vals.last().unwrap().1;
			}
		}

		global_min = min.min(global_min);
		global_max = max.max(global_max);
		vals.push((min, max));
	}
	println!("global {} | {}", global_min, global_max);
	let diff = global_max - global_min;
	let x = 1;
	let y = 1;
	let scaley = 64;
	let scalex = 2;
	vals.reverse();
	for (i, (a, b)) in vals.into_iter().enumerate() {
		let x = x + i as i32 * scalex;
		let y1 = y + (global_max - b) * scaley / diff;
		let y2 = y + (global_max - a) * scaley / diff;
		let height = y2 - y1 + 1;
		disp.fill_solid(
			&Rectangle::new((x, y1).into(), (scalex as u32, height as u32).into()),
			BinaryColor::On,
		)
		.unwrap();
	}

	disp.buffer.save("/tmp/frame.png").unwrap();
	*/
}
