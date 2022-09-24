use std::process::Command;

fn main() {
	// adjust as needed

	let github_up = Command::new("ping")
		.args(["-c1", "github.com"])
		.spawn()
		.unwrap()
		.wait()
		.unwrap()
		.success();
	let gitlab_up = Command::new("ping")
		.args(["-c1", "gitlab.com"])
		.spawn()
		.unwrap()
		.wait()
		.unwrap()
		.success();

	let status = format!("{} {} {} {} {}", true, github_up, true, gitlab_up, true);
	std::fs::write("/run/user/1000/status.json", status).unwrap();
}
