use std::{
	io::{Error, Write},
	net::{TcpListener, TcpStream},
	thread,
	time::{Duration, Instant},
};

use raspi_oled::github::get_new_notifications;

fn main() {
	let pat = Box::leak(Box::new(
		std::env::var("GITHUB_PAT").expect("no env var GITHUB_PAT set"),
	));
	let listener = TcpListener::bind("169.254.1.2:26769").unwrap();
	for stream in listener.incoming().flat_map(|x| x.ok()) {
		let pat = &*pat;
		thread::spawn(move || {
			let _ = handle(stream, pat);
		});
	}
}

fn handle(mut socket: TcpStream, github_pat: &str) -> Result<(), Error> {
	let mut last_github_check = Instant::now();
	let mut last_modified: Option<String> = None;
	loop {
		let now = Instant::now();
		if now.duration_since(last_github_check).as_secs() >= 60 {
			last_github_check = now;
			let new = get_new_notifications(github_pat, last_modified.as_deref());
			if let Ok((notifications, last_modified_new)) = new {
				last_modified = last_modified_new;
				let relevant: Vec<_> = notifications.into_iter().filter(|x| x.unread).collect();
				if relevant.is_empty() {
					continue;
				}
				let max_lines = 8;
				let mut lines = vec![];
				let mut relevant = relevant.into_iter();
				while lines.len() < max_lines {
					if let Some(x) = relevant.next() {
						let url = x.subject.url;
						let Some(url) = url else {
							lines.push("no url".to_owned());
							continue;
						};
						let parts: Vec<_> = url.split('/').collect();
						if parts.len() < 8 {
							lines.push("too few url parts".to_owned());
							continue;
						}
						lines.push(format!("{} #{}", parts[5], parts[7]));
						if lines.len() < max_lines {
							lines.push(x.subject.title.clone());
						}
					} else {
						break;
					}
				}
				let remaining = relevant.count();
				if remaining != 0 {
					lines.push(format!("... {} more", remaining));
				}
				socket.write_all(format!("GITHUB {}\n", lines.len()).as_bytes())?;
				for line in lines {
					socket.write_all(line.as_bytes())?;
					socket.write_all(b"\n")?;
				}
			} else {
				eprintln!("error: {new:?}");
			}
		}

		thread::sleep(Duration::from_secs(1));
	}
}
