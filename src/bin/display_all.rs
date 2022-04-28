use std::fs;

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
	draw_target::DrawTarget,
	mono_font::{
		ascii::{FONT_10X20, FONT_5X8, FONT_6X9, FONT_9X15},
		MonoTextStyleBuilder,
	},
	pixelcolor::{BinaryColor, Rgb565},
	prelude::{OriginDimensions, Point, Primitive, Size},
	primitives::{PrimitiveStyleBuilder, Rectangle},
	text::Text,
	Drawable,
};
use image::{ImageBuffer, Rgb};
use linux_embedded_hal::I2cdev;
use rppal::{
	gpio::Gpio,
	hal::Delay,
	spi::{Bus, Mode, SlaveSelect, Spi},
};
use rusqlite::Connection;
use serde_derive::Deserialize;
//use ssd1306::{I2CDisplayInterface, Ssd1306, size::DisplaySize128x64, rotation::DisplayRotation, mode::DisplayConfig};
use time::{format_description, OffsetDateTime, PrimitiveDateTime};
use time_tz::{timezones::db::europe::BERLIN, OffsetDateTimeExt, PrimitiveDateTimeExt};

#[derive(Deserialize)]
struct Events {
	events: Vec<Event>,
	weekly: Vec<Weekly>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Event {
	name: String,
	start_time: String,
}

#[derive(Deserialize)]
struct Weekly {
	name: String,
	day: i32,
	hour: i32,
	minute: i32,
	duration: i32,
}

fn main() {
	let args = std::env::args().collect::<Vec<_>>();
	if args.len() < 4 {
		panic!("missing argument: database path, event JSON data file, events / temps");
	}
	let database = Connection::open(&args[1]).expect("failed to open database");
	let events = fs::read_to_string(&args[2]).expect("failed to read events.json");
	let events: Events = serde_json::from_str(&events).unwrap();

	let (rh, temp): (i64, i64) = database
		.query_row(
			"SELECT humidity, celsius FROM sensor_readings ORDER BY sensor_readings.time DESC LIMIT 1",
			[],
			|row| Ok((row.get(0).unwrap(), row.get(1).unwrap())),
		)
		.unwrap();

	let time = OffsetDateTime::now_utc().to_timezone(BERLIN);

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
		let mut min = hour[1];
		let mut max = hour[hour.len() - 2];
		println!("min {} max {}", min, max);
		// sanity check value
		if max > 300 {
			if vals.is_empty() {
				max = min;
			} else {
				max = vals.last().unwrap().1;
			}
			min = min.min(max);
		}

		global_min = min.min(global_min);
		global_max = max.max(global_max);
		vals.push((min, max));
	}
	println!("global {} | {}", global_min, global_max);

	let hour = time.hour();
	let minute = time.minute();
	//let i2c = I2cdev::new("/dev/i2c-1").unwrap();
	//let interface = I2CDisplayInterface::new(i2c);
	//let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();
	//disp.init().unwrap();
	//let mut disp = FrameOutput { buffer: ImageBuffer::new(128, 64) };
	let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 19660800, Mode::Mode0).unwrap();
	let gpio = Gpio::new().unwrap();
	let dc = gpio.get(25).unwrap().into_output();
	let mut rst = gpio.get(27).unwrap().into_output();

	// Init SPI
	let spii = SPIInterfaceNoCS::new(spi, dc);
	let mut disp = ssd1351::display::display::Ssd1351::new(spii);

	// Reset & init
	//disp.reset(&mut rst, &mut Delay).unwrap();
	//disp.turn_on().unwrap();

	let text_style_clock = MonoTextStyleBuilder::new()
		.font(&FONT_10X20)
		.text_color(Rgb565::new(0xff, 0xff, 0xff))
		.build();
	let text_style2 = MonoTextStyleBuilder::new()
		.font(&FONT_9X15)
		.text_color(Rgb565::new(0xff, 0xff, 0xff))
		.build();
	let text_style3 = MonoTextStyleBuilder::new()
		.font(&FONT_6X9)
		.text_color(Rgb565::new(0xff, 0xff, 0xff))
		.build();
	let text_style4 = MonoTextStyleBuilder::new()
		.font(&FONT_5X8)
		.text_color(Rgb565::new(0xff, 0xff, 0xff))
		.build();
	let rect_style = PrimitiveStyleBuilder::new()
		.fill_color(Rgb565::new(0xff, 0xff, 0xff))
		.build();

	//let text = format!("{}.{}% {}.{}°C", rh / 10, rh % 10, temp / 10, temp % 10);
	//Text::new(&text, Point::new(0, 10), text_style).draw(&mut disp).unwrap();
	let hour = format!("{:02}", hour);
	Text::new(&hour, Point::new(64 + 10, 6 + 20), text_style_clock)
		.draw(&mut disp)
		.unwrap();
	let minute = format!("{:02}", minute);
	Text::new(&minute, Point::new(64 + 10 + 20 + 4, 6 + 20), text_style_clock)
		.draw(&mut disp)
		.unwrap();
	Rectangle::new((95, 17).into(), (2, 2).into())
		.into_styled(rect_style)
		.draw(&mut disp)
		.unwrap();
	Rectangle::new((95, 22).into(), (2, 2).into())
		.into_styled(rect_style)
		.draw(&mut disp)
		.unwrap();

	let rh = format!("{:02}", rh / 10);
	Text::new(&rh, Point::new(64 + 3, 64 - 4), text_style2)
		.draw(&mut disp)
		.unwrap();
	Text::new("%", Point::new(64 + 3 + 18, 64 - 4), text_style3)
		.draw(&mut disp)
		.unwrap();
	let temp_int = format!("{:02}", temp / 10);
	Text::new(&temp_int, Point::new(64 + 32 + 3, 64 - 4), text_style2)
		.draw(&mut disp)
		.unwrap();
	Rectangle::new((64 + 32 + 3 + 18, 64 - 4).into(), (1, 1).into())
		.into_styled(rect_style)
		.draw(&mut disp)
		.unwrap();
	let temp_fract = format!("{}", temp % 10);
	Text::new(&temp_fract, Point::new(64 + 32 + 3 + 18 + 2, 64 - 4), text_style3)
		.draw(&mut disp)
		.unwrap();

	for (x, y) in [
		(118, 49),
		(119, 49),
		(117, 50),
		(117, 51),
		(120, 50),
		(120, 51),
		(118, 52),
		(119, 52),
		(122, 50),
		(122, 51),
		(122, 52),
		(123, 49),
		(124, 49),
		(123, 53),
		(124, 53),
	] {
		Rectangle::new((x, y).into(), (1, 1).into())
			.into_styled(rect_style)
			.draw(&mut disp)
			.unwrap();
	}

	let x = 0;
	let y = 0;
	Rectangle::new((x + 2, y + 8).into(), (1, 24).into())
		.into_styled(rect_style)
		.draw(&mut disp)
		.unwrap();
	Rectangle::new((x + 1, y + 16).into(), (1, 1).into())
		.into_styled(rect_style)
		.draw(&mut disp)
		.unwrap();
	Rectangle::new((x + 1, y + 28).into(), (1, 1).into())
		.into_styled(rect_style)
		.draw(&mut disp)
		.unwrap();
	let mut day = time.weekday().number_days_from_monday() as usize;
	let days = [
		("M", "o"),
		("D", "i"),
		("M", "i"),
		("D", "o"),
		("F", "r"),
		("S", "a"),
		("S", "o"),
	];
	for i in 0..5 {
		Text::new(days[day].0, (x + 12 * i + 4, y + 6).into(), text_style3)
			.draw(&mut disp)
			.unwrap();
		Text::new(days[day].1, (x + 12 * i + 10, y + 6).into(), text_style4)
			.draw(&mut disp)
			.unwrap();
		day += 1;
		day %= days.len();
	}
	let mut bits = vec![];
	// events
	let mut all_events = vec![];
	for event in events.weekly {
		all_events.push((event.day, event.hour, event.minute, event.duration, event.name));
	}
	let today = time.date().to_julian_day();
	let format = format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]").unwrap();
	for event in events.events {
		let dt = PrimitiveDateTime::parse(&event.start_time, &format)
			.unwrap()
			.assume_timezone(BERLIN)
			.unwrap();
		let julian_day = dt.to_julian_day();
		if dt < time || julian_day - today > 4 {
			continue;
		}
		all_events.push((
			dt.weekday().number_days_from_monday() as _,
			dt.hour() as _,
			dt.minute() as _,
			30,
			event.name,
		)); // TODO length
	}
	let weekday = time.weekday().number_days_from_monday() as i32;
	all_events.sort_by_key(|x| (((x.0 + 7) - weekday) % 7, x.1, x.2));
	println!("{:?}", all_events);
	let mut time_until_first = None;
	for i in 0..5 {
		let day = (weekday + i) % 7;
		for hour in 0..24 {
			for minute in 0..60 {
				if minute % 6 != 0 {
					continue;
				}

				if i == 0 && hour == time.hour() as i32 && minute == (time.minute() as i32 / 6) * 6 {
					bits.push((i, hour, minute / 6, Rgb565::new(0xff, 0x00, 0xff)));
				}

				for event in &all_events {
					if event.0 != day {
						continue;
					}
					let event_start = event.1 * 60 + event.2;
					let event_end = event_start + event.3;
					let now = hour * 60 + minute;
					let now2 = hour * 60 + minute + 6;
					if now2 > event_start && now < event_end {
						bits.push((i, hour, minute / 6, Rgb565::new(0xff, 0xff, 0x10)));
					}
					if time_until_first.is_none() {
						time_until_first = Some(
							((i * 24 + event.1) * 60 + event.2) * 60
								- (time.hour() as i32 * 60 + time.minute() as i32) * 60,
						);
					}
				}
			}
		}
	}
	for (d, h, m, color) in bits {
		// calculate position
		let x = x + 4 + d * 12 + m;
		let y = y + 8 + h;
		disp.fill_solid(&Rectangle::new((x, y).into(), (1, 1).into()), color)
			.unwrap();
		//Rectangle::new((x, y).into(), (1, 1).into()).into_styled(rect_style).draw(&mut disp).unwrap();
	}
	if args[3] == "events" {
		for (i, event) in all_events.iter().take(7).enumerate() {
			let text = if event.4.len() > 10 { &event.4[0..10] } else { &event.4 };
			Text::new(text, (x + 2, y + 64 + 9 * i as i32 + 5).into(), text_style3)
				.draw(&mut disp)
				.unwrap();
		}
	} else {
		let diff = global_max - global_min;
		let x = 0;
		let y = 64;
		let scaley = 63;
		let scalex = 2;
		vals.reverse();
		for (i, (a, b)) in vals.into_iter().enumerate() {
			let x = x + i as i32 * scalex;
			let y1 = y + (global_max - b) * scaley / diff;
			let y2 = y + (global_max - a) * scaley / diff;
			let height = y2 - y1 + 1;
			let rect = Rectangle::new((x, y1).into(), (scalex as u32, height as u32).into());
			disp.fill_solid(&rect, Rgb565::new(0xff, 0xff, 0xff)).unwrap();
		}
		Text::new(
			&format!("{}", global_max as f32 / 10.0),
			(100, 64 + 10).into(),
			text_style3,
		)
		.draw(&mut disp)
		.unwrap();
		Text::new(
			&format!("{}", global_min as f32 / 10.0),
			(100, 64 + 50).into(),
			text_style3,
		)
		.draw(&mut disp)
		.unwrap();
	}
	if let Some(secs) = time_until_first {
		let days = secs / (24 * 60 * 60);
		let hours = secs / (60 * 60) % 24;
		let minutes = secs / 60 % 60;
		let text = if days > 0 {
			String::new()
		} else if hours > 0 {
			format!("{}h{}m", hours, minutes)
		} else if minutes > 0 {
			format!("{}m", minutes)
		} else {
			"?".into()
		};
		Text::new(&text, (x + 2, y + 60).into(), text_style2)
			.draw(&mut disp)
			.unwrap();
	}
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

	let _ = disp.flush();
	//disp.buffer.save("/tmp/frame.png").unwrap();
}
