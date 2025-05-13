use std::{
	error::Error,
	fmt::Display,
	io::{self, Read},
	process::{Command, Stdio},
};

pub fn execute(cmd: &[&'static str]) {
	let args = &cmd[1..];
	let mut spawned = Command::new(cmd[0]).args(args).spawn().unwrap();
	let _wait = spawned.wait().unwrap();
}

pub fn get_volume() -> Result<i32, io::Error> {
	let mut spawned = Command::new("pactl")
		.args(["get-sink-volume", "@DEFAULT_SINK@"])
		.stdout(Stdio::piped())
		.spawn()
		.unwrap();
	let _wait = spawned.wait().unwrap();
	let mut out = spawned.stdout.unwrap();
	let mut buf = String::new();
	out.read_to_string(&mut buf)?;
	let mut it = buf.splitn(3, " / ");
	if let Some(perc) = it.nth(1) {
		if let Some(it) = perc.trim().strip_suffix('%').map(|x| x.parse::<i32>().ok()).flatten() {
			return Ok(it);
		}
	}
	Err(io::Error::new(io::ErrorKind::Other, IoError {}))
}

pub fn set_volume(delta: &str) {
	let mut spawned = Command::new("pactl")
		.args(["set-sink-volume", "@DEFAULT_SINK@", delta])
		.spawn()
		.unwrap();
	let _wait = spawned.wait().unwrap();
}

#[derive(Debug, Clone, Copy)]
struct IoError {}

impl Display for IoError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl Error for IoError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		None
	}

	fn description(&self) -> &str {
		"description() is deprecated; use Display"
	}

	fn cause(&self) -> Option<&dyn Error> {
		self.source()
	}
}
