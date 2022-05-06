use std::{error::Error, fs};

static WEEKLY: &'static str = include_str!("../../events_weekly.json");

fn main() {
	let url = "http://nixos.fritz.box:12783/custom/event_alerts";

	if let Ok(json) = get_json(url) {
		let mut buf = String::new();
		buf += "{";
		buf += "\"events\": ";
		buf += &json;
		buf += ",";
		buf += WEEKLY;
		buf += "}";
		fs::write("events.json", buf.as_bytes()).unwrap();
	}
}

fn get_json(url: &str) -> Result<String, Box<dyn Error>> {
	Ok(ureq::get(url).call()?.into_string()?)
}