use std::time::{Duration, SystemTime};

use gpio_cdev::Chip;
use rusqlite::{params, Connection};

fn main() {
	let args = std::env::args().collect::<Vec<_>>();
	if args.len() < 2 {
		panic!("missing argument: database path");
	}
	let database = Connection::open(&args[1]).expect("failed to open database");
	database
		.execute(
			"
		CREATE TABLE IF NOT EXISTS sensor_readings(
			time INTEGER PRIMARY KEY,
			humidity INTEGER NOT NULL,
			celsius INTEGER NOT NULL
		)",
			[],
		)
		.unwrap();

	let mut chip = Chip::new("/dev/gpiochip0").unwrap();
	let line = chip.get_line(26).unwrap();
	let mut attempts = 0;
	let mut temps = vec![];
	let mut rhs = vec![];
	let time = std::time::SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap();
	while temps.len() < 5 && attempts < 10 {
		if let Ok((rh, temp)) = raspi_oled::am2302_reading(&line) {
			if rh > 0 && temp < 500 {
				rhs.push(rh);
				temps.push(temp);
			}
		}
		std::thread::sleep(Duration::from_secs(5));
		attempts += 1;
	}
	if !temps.is_empty() {
		// median = hopefully no faulty readings
		temps.sort();
		rhs.sort();
		database
			.execute(
				"INSERT INTO sensor_readings (time, humidity, celsius) VALUES (?1, ?2, ?3)",
				params![time.as_secs(), rhs[rhs.len() / 2], temps[temps.len() / 2]],
			)
			.unwrap();
	}
}
