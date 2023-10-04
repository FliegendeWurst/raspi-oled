use std::{
	process::{Command, Stdio},
	time::SystemTime,
};

use rusqlite::Connection;

fn main() {
	let args = std::env::args().collect::<Vec<_>>();
	if args.len() < 2 {
		panic!("missing argument: database path");
	}
	let database = Connection::open(&args[1]).expect("failed to open database");

	let timestamp: i64 = database
		.query_row(
			"SELECT time FROM sensor_readings ORDER BY time DESC LIMIT 1",
			[],
			|row| Ok(row.get(0).unwrap()),
		)
		.unwrap();

	let time = std::time::SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs() as i64;

	let recent_reading = time - timestamp <= 12 * 60;
	let pi_up = Command::new("ping")
		.args(["-c1", "raspberrypi.fritz.box"])
		.spawn()
		.unwrap()
		.wait()
		.unwrap()
		.success();
	let traffic_good = if pi_up {
		let x = Command::new("ssh")
			.args(["pi@raspberrypi", "vnstat --json h 2"])
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()
			.unwrap()
			.wait_with_output()
			.unwrap();
		if x.status.success() {
			if let Ok(x) = serde_json::from_slice::<serde_json::Value>(&x.stdout) {
				let it = x.pointer("/interfaces/0/traffic/hour/0/tx");
				it.map(|x| x.as_u64()).flatten().unwrap_or(0) / (60 * 60 * 1000 * 1000 / 8) > 5
			} else {
				false
			}
		} else {
			false
		}
	} else {
		false
	};
	let nixos_up = Command::new("ping")
		.args(["-c1", "nixos.fritz.box"])
		.spawn()
		.unwrap()
		.wait()
		.unwrap()
		.success();
	let sync_good = if nixos_up {
		if let Ok(x) = ureq::get("http://nixos.fritz.box:12783/").call() {
			x.status() < 400
		} else {
			false
		}
	} else {
		false
	};
	let status = format!(
		"{} {} {} {} {}",
		recent_reading, pi_up, traffic_good, nixos_up, sync_good
	);
	std::fs::write("/run/user/1000/status.json", status).unwrap();
}
