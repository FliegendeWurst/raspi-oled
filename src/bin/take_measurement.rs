use std::time::{Duration, SystemTime};

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

	let mut attempts = 0;
	let mut temps = vec![];
	let mut rhs = vec![];
	let time = std::time::SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap();
	while temps.len() < 5 && attempts < 10 {
		if let Ok((rh, temp)) = raspi_oled::am2302_reading() {
			// TODO: try out gpio_am2302_rs!
			//if let Ok(reading) = gpio_am2302_rs::try_read(26) {
			//let rh = reading.humidity as i64;
			//let temp = (reading.temperature * 10.0) as i64;
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
		let rh = rhs[rhs.len() / 2];
		let temp = temps[temps.len() / 2];
		println!(
			"info: acquired {} readings (temps {:?}, rhs {:?}), using rh {} and temp {}",
			temps.len(),
			temps,
			rhs,
			rh,
			temp
		);
		database
			.execute(
				"INSERT INTO sensor_readings (time, humidity, celsius) VALUES (?1, ?2, ?3)",
				params![time.as_secs(), rh, temp],
			)
			.unwrap();
	}
}
