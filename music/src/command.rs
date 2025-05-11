use std::process::Command;

pub fn execute(cmd: &[&'static str]) {
	let mut args = &cmd[1..];
	let mut spawned = Command::new(cmd[0]).args(args).spawn().unwrap();
	let wait = spawned.wait().unwrap();
}
