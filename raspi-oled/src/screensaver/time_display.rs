use std::any::Any;
use std::cell::RefCell;

use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_graphics::{mono_font::MonoTextStyleBuilder, pixelcolor::Rgb565, prelude::DrawTarget};
use time::{Duration, OffsetDateTime};
use time_tz::{timezones::db::europe::BERLIN, OffsetDateTimeExt};

use crate::context::{Draw, Rng};

use super::Screensaver;

static TIME_COLOR: Rgb565 = Rgb565::new(0b01_111, 0b011_111, 0b01_111);

#[derive(Debug, Clone)]
pub struct TimeDisplay {
	last_min: RefCell<OffsetDateTime>,
}

impl TimeDisplay {
	pub fn new() -> Self {
		TimeDisplay {
			last_min: RefCell::new(
				OffsetDateTime::now_utc()
					.to_timezone(BERLIN)
					.checked_sub(Duration::minutes(2))
					.unwrap(),
			),
		}
	}
}

impl<D: DrawTarget<Color = Rgb565>> Screensaver<D> for TimeDisplay {
	fn id(&self) -> &'static str {
		"time"
	}

	fn convert_draw(&self) -> Box<dyn Draw<D>> {
		Box::new(self.clone())
	}
}

impl<D: DrawTarget<Color = Rgb565>> Draw<D> for TimeDisplay {
	fn draw(&self, disp: &mut D, _rng: &mut Rng) -> Result<bool, D::Error> {
		let time = OffsetDateTime::now_utc().to_timezone(BERLIN);
		if time.minute() == self.last_min.borrow().minute() {
			return Ok(false);
		}
		*self.last_min.borrow_mut() = time;
		disp.clear(Rgb565::new(0, 0, 0))?;
		let text_style_clock = MonoTextStyleBuilder::new().font(&FONT_10X20);
		let hour = time.hour();
		let text_style_clock = match hour {
			// red, ?, bright green, blue, dark blue, purple
			0 => text_style_clock.text_color(Rgb565::new(0b01_111, 0b000_000, 0b00_000)),
			1 => text_style_clock.text_color(Rgb565::new(0b01_111, 0b011_111, 0b00_000)),
			2 => text_style_clock.text_color(Rgb565::new(0b00_000, 0b011_111, 0b00_000)),
			3 => text_style_clock.text_color(Rgb565::new(0b00_000, 0b011_111, 0b01_111)),
			4 => text_style_clock.text_color(Rgb565::new(0b00_000, 0b000_000, 0b01_111)),
			5 => text_style_clock.text_color(Rgb565::new(0b01_111, 0b000_000, 0b01_111)),
			// repeats
			// another blue, another red/brown, another green
			6 => text_style_clock.text_color(Rgb565::new(0b00_111, 0b001_111, 0b01_111)),
			7 => text_style_clock.text_color(Rgb565::new(0b01_111, 0b001_111, 0b00_111)),
			8 => text_style_clock.text_color(Rgb565::new(0b00_111, 0b011_111, 0b00_111)),
			_ => text_style_clock.text_color(TIME_COLOR),
		}
		.build();
		let minute = time.minute();
		let unix_minutes = minute as i32 * 5 / 3; // (time.unix_timestamp() / 60) as i32;
		let dx = ((hour % 3) as i32 - 1) * 40 - 2;
		let hour = format!("{:02}", hour);
		Text::new(
			&hour,
			Point::new(64 - 20 + dx, 20 + unix_minutes % 100),
			text_style_clock,
		)
		.draw(disp)?;
		Text::new(&":", Point::new(64 - 3 + dx, 18 + unix_minutes % 100), text_style_clock).draw(disp)?;
		let minute = format!("{:02}", minute);
		Text::new(
			&minute,
			Point::new(64 + 5 + dx, 20 + unix_minutes % 100),
			text_style_clock,
		)
		.draw(disp)?;
		Ok(true)
	}

	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}
