use std::{any::Any, fs, ops::Sub, sync::atomic::AtomicBool, time::Duration};

use embedded_graphics::{
	image::ImageRaw,
	mono_font::{
		ascii::{FONT_4X6, FONT_5X8, FONT_6X9, FONT_9X15},
		mapping::StrGlyphMapping,
		DecorationDimensions, MonoFont, MonoTextStyleBuilder,
	},
	pixelcolor::Rgb565,
	prelude::*,
	primitives::{Primitive, PrimitiveStyleBuilder, Rectangle},
	text::{renderer::CharacterStyle, Text},
	Drawable,
};
use raspi_oled::Events;
use time::{format_description, Date, OffsetDateTime, PrimitiveDateTime};

use crate::{screensaver::Screensaver, Context, ContextDefault, Draw, BLACK};
use time_tz::{timezones::db::europe::BERLIN, OffsetDateTimeExt, PrimitiveDateTimeExt};

static CLOCK_FONT: MonoFont = MonoFont {
	image: ImageRaw::new(include_bytes!("font_15x30.raw"), 165),
	glyph_mapping: &StrGlyphMapping::new("0123456789:", 0),
	character_size: Size::new(15, 30),
	character_spacing: 0,
	baseline: 22,
	underline: DecorationDimensions::default_underline(30),
	strikethrough: DecorationDimensions::default_strikethrough(30),
};

#[derive(Debug)]
pub struct Measurements {
	drawn: AtomicBool,
	mode: MeasurementsMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MeasurementsMode {
	Default,
	Temps,
	Events,
}

impl Default for Measurements {
	fn default() -> Self {
		Self {
			drawn: AtomicBool::new(false),
			mode: MeasurementsMode::Default,
		}
	}
}

impl Measurements {
	pub fn temps() -> Self {
		Self {
			drawn: AtomicBool::new(false),
			mode: MeasurementsMode::Temps,
		}
	}

	pub fn events() -> Self {
		Self {
			drawn: AtomicBool::new(false),
			mode: MeasurementsMode::Events,
		}
	}
}

impl<D: DrawTarget<Color = Rgb565>> Screensaver<D> for Measurements {
	fn id(&self) -> &'static str {
		match self.mode {
			MeasurementsMode::Default => "measurements",
			MeasurementsMode::Temps => "measurements_temps",
			MeasurementsMode::Events => "measurements_events",
		}
	}

	fn convert_draw(&self) -> Box<dyn Draw<D>> {
		Box::new(Measurements {
			drawn: AtomicBool::new(false),
			mode: self.mode,
		})
	}
}

impl<D: DrawTarget<Color = Rgb565>> Draw<D> for Measurements {
	fn draw_with_ctx(&self, ctx: &ContextDefault<D>, disp: &mut D, _rng: &mut crate::Rng) -> Result<bool, D::Error> {
		if self.drawn.load(std::sync::atomic::Ordering::Relaxed) {
			return Ok(false);
		}
		disp.clear(BLACK)?;
		let events = fs::read_to_string("events.json").expect("failed to read events.json");
		let events: Events = serde_json::from_str(&events).unwrap();
		let database = ctx.database();
		let database = database.borrow_mut();

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
			//println!("min {} max {}", min, max);
			// sanity check value
			if max > 400 {
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

		let hour = time.hour();
		let minute = time.minute();

		let text_style_clock = MonoTextStyleBuilder::new()
			.font(&CLOCK_FONT)
			.text_color(Rgb565::new(0xff, 0xff, 0xff))
			.build();
		let text_style2 = MonoTextStyleBuilder::new()
			.font(&FONT_9X15)
			.text_color(Rgb565::new(0xff, 0xff, 0xff))
			.build();
		let mut text_style_6x9 = MonoTextStyleBuilder::new()
			.font(&FONT_6X9)
			.text_color(Rgb565::new(0xff, 0xff, 0xff))
			.build();
		let text_style_4x6 = MonoTextStyleBuilder::new()
			.font(&FONT_4X6)
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
		//Text::new(&text, Point::new(0, 10), text_style).draw(disp).unwrap();
		let hour = format!("{:02}", hour);
		Text::new(&hour, Point::new(64 - 2, 6 + 20), text_style_clock).draw(disp)?;
		let minute = format!("{:02}", minute);
		Text::new(&minute, Point::new(64 + 30 + 4, 6 + 20), text_style_clock).draw(disp)?;
		Rectangle::new((93, 14).into(), (4, 4).into())
			.into_styled(rect_style)
			.draw(disp)?;
		Rectangle::new((93, 22).into(), (4, 4).into())
			.into_styled(rect_style)
			.draw(disp)?;

		let rh = format!("{:02}", rh / 10);
		Text::new(&rh, Point::new(64 + 3, 64 - 4), text_style2).draw(disp)?;
		Text::new("%", Point::new(64 + 3 + 18, 64 - 4), text_style_6x9).draw(disp)?;
		let temp_int = format!("{:02}", temp / 10);
		Text::new(&temp_int, Point::new(64 + 32 + 3, 64 - 4), text_style2).draw(disp)?;
		Rectangle::new((64 + 32 + 3 + 18, 64 - 4).into(), (1, 1).into())
			.into_styled(rect_style)
			.draw(disp)?;
		let temp_fract = format!("{}", temp % 10);
		Text::new(&temp_fract, Point::new(64 + 32 + 3 + 18 + 2, 64 - 4), text_style_6x9).draw(disp)?;

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
				.draw(disp)?;
		}

		let x = 0;
		let y = 0;
		Rectangle::new((x + 2, y + 8).into(), (1, 24).into())
			.into_styled(rect_style)
			.draw(disp)?;
		Rectangle::new((x + 1, y + 16).into(), (1, 1).into())
			.into_styled(rect_style)
			.draw(disp)?;
		Rectangle::new((x + 1, y + 28).into(), (1, 1).into())
			.into_styled(rect_style)
			.draw(disp)?;
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
			Text::new(days[day].0, (x + 12 * i + 4, y + 6).into(), text_style_6x9).draw(disp)?;
			Text::new(days[day].1, (x + 12 * i + 10, y + 6).into(), text_style4).draw(disp)?;
			day += 1;
			day %= days.len();
		}
		let mut bits = vec![];
		// events
		let mut all_events = vec![];
		for event in events.weekly {
			let mut event_time = time.clone();
			while event_time.weekday().number_days_from_monday() as i32 != event.day {
				event_time += Duration::from_secs(24 * 60 * 60);
			}
			all_events.push((
				event.day,
				event.hour,
				event.minute,
				event.duration,
				event.name,
				event_time.to_julian_day(),
			));
		}
		let format = format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]").unwrap();
		for event in events.events {
			let dt = PrimitiveDateTime::parse(&event.start_time, &format)
				.unwrap()
				.assume_timezone(BERLIN)
				.unwrap();
			let julian_day = dt.to_julian_day();
			if dt < time {
				continue;
			}
			let duration = if let Some(end_time) = event.end_time.as_ref() {
				let dt2 = PrimitiveDateTime::parse(end_time, &format)
					.unwrap()
					.assume_timezone(BERLIN)
					.unwrap();
				(dt2.sub(dt).as_seconds_f32() / 60.0) as i32
			} else {
				30
			};
			all_events.push((
				dt.weekday().number_days_from_monday() as _,
				dt.hour() as _,
				dt.minute() as _,
				duration,
				event.name,
				julian_day,
			));
		}
		let today = time.date().to_julian_day();
		let weekday = time.weekday().number_days_from_monday() as i32;
		all_events.sort_by_key(|x| (x.5, ((x.0 + 7) - weekday) % 7, x.1, x.2));
		//println!("{:?}", all_events);
		let mut time_until_first = None;
		let colors = vec![
			Rgb565::new(0xff >> 3, 0xff >> 2, 0x00 >> 3),
			Rgb565::new(0xff >> 3, 0x00 >> 2, 0xff >> 3),
			Rgb565::new(0x00 >> 3, 0xff >> 2, 0xff >> 3),
			Rgb565::new(0xff >> 3, 0x00 >> 2, 0x00 >> 3),
			Rgb565::new(0x00 >> 3, 0xff >> 2, 0x00 >> 3),
			Rgb565::new(0x00 >> 3, 0x00 >> 2, 0xff >> 3),
			Rgb565::new(0xff >> 3, 0xff >> 2, 0xff >> 3),
		];
		for i in 0..5 {
			let day = (weekday + i) % 7;
			for hour in 0..24 {
				for minute in 0..60 {
					if minute % 6 != 0 {
						continue;
					}

					if i == 0 && hour == time.hour() as i32 && minute == (time.minute() as i32 / 6) * 6 {
						bits.push((i, hour, minute / 6, Some(Rgb565::new(0xff, 0x00, 0xff))));
					}

					for (event_idx, event) in all_events.iter().enumerate() {
						if event.0 != day || event.5 < today || event.5 - today > 4 {
							continue;
						}
						let event_start = event.1 * 60 + event.2;
						let event_end = event_start + event.3;
						let now = hour * 60 + minute;
						let now2 = hour * 60 + minute + 6;
						if now2 > event_start && now < event_end {
							bits.push((i, hour, minute / 6, colors.get(event_idx).copied()));
						}
						if time_until_first.is_none()
							&& (i > 0
								|| event.1 > time.hour() as i32 || (event.1 == time.hour() as i32
								&& event.2 >= time.minute() as i32))
						{
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
			disp.fill_solid(
				&Rectangle::new((x, y).into(), (1, 1).into()),
				color.unwrap_or(Rgb565::new(0xff, 0xff, 0x10)),
			)?;
			//Rectangle::new((x, y).into(), (1, 1).into()).into_styled(rect_style).draw(disp).unwrap();
		}
		if self.mode == MeasurementsMode::Events {
			for (i, event) in all_events.iter().take(7).enumerate() {
				let text = if event.4.len() > 19 {
					&event.4[0..event.4.floor_char_boundary(19)]
				} else {
					&event.4
				};
				let day = event.0 as usize;
				let y = y + 64 + 9 * i as i32 + 5;
				if event.5 > today && event.5 - today > 7 {
					let dt = Date::from_julian_day(event.5).unwrap();
					Text::new(
						&format!("{}.{}.", dt.day(), dt.month() as u8),
						(0, y).into(),
						text_style_4x6,
					)
					.draw(disp)?;
				} else {
					text_style_6x9.set_text_color(Some(Rgb565::new(0xff, 0xff, 0xff)));
					Text::new(days[day].0, (x, y).into(), text_style_6x9).draw(disp)?;
					Text::new(days[day].1, (x + 6, y).into(), text_style4).draw(disp)?;
				}
				text_style_6x9.set_text_color(Some(colors[i]));
				Text::new(text, (x + 14, y).into(), text_style_6x9).draw(disp)?;
			}
		} else if self.mode == MeasurementsMode::Temps {
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
				disp.fill_solid(&rect, Rgb565::new(0xff, 0xff, 0xff))?;
			}
			Text::new(
				&format!("{}", global_max as f32 / 10.0),
				(100, 64 + 10).into(),
				text_style_6x9,
			)
			.draw(disp)?;
			Text::new(
				&format!("{}", global_min as f32 / 10.0),
				(100, 64 + 50).into(),
				text_style_6x9,
			)
			.draw(disp)?;
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
			Text::new(&text, (x + 2, y + 60).into(), text_style2).draw(disp)?;
		}

		self.drawn.store(true, std::sync::atomic::Ordering::Relaxed);

		Ok(true)
	}

	fn draw(&self, _disp: &mut D, _rng: &mut crate::Rng) -> Result<bool, <D as DrawTarget>::Error> {
		panic!("draw without ctx");
	}

	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}
