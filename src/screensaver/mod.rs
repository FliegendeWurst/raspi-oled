use std::any::Any;
use std::cell::RefCell;
use std::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, Ordering};

use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::{
	mono_font::MonoTextStyleBuilder,
	pixelcolor::Rgb565,
	prelude::{DrawTarget, Point, Size},
	primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
	text::Text,
	Drawable,
};
use rand_xoshiro::rand_core::RngCore;
use time::{Duration, OffsetDateTime};
use time_tz::{timezones::db::europe::BERLIN, OffsetDateTimeExt};

use crate::context::{Context, Draw, Rng};
use crate::schedule::Schedule;

pub static SPEED: AtomicU64 = AtomicU64::new(32);

pub trait Screensaver<D: DrawTarget<Color = Rgb565>>: Draw<D> {
	fn id(&self) -> &'static str;
	fn convert_draw(&self) -> Box<dyn Draw<D>>;
}

#[derive(Debug)]
pub struct SimpleScreensaver {
	id: &'static str,
	data: &'static [u8],
	iters: AtomicU32,
}

impl Clone for SimpleScreensaver {
	fn clone(&self) -> Self {
		Self {
			id: self.id,
			data: self.data,
			iters: AtomicU32::new(self.iters.load(std::sync::atomic::Ordering::Relaxed)),
		}
	}
}

impl<D: DrawTarget<Color = Rgb565>> Screensaver<D> for SimpleScreensaver {
	fn id(&self) -> &'static str {
		self.id
	}

	fn convert_draw(&self) -> Box<dyn Draw<D>> {
		Box::new(self.clone())
	}
}

impl<D: DrawTarget<Color = Rgb565>> Draw<D> for SimpleScreensaver {
	fn draw(&self, disp: &mut D, rng: &mut Rng) -> Result<bool, D::Error> {
		for _ in 0..SPEED.load(std::sync::atomic::Ordering::Relaxed) {
			let x = (rng.next_u32() % 128) as usize;
			let y = (rng.next_u32() % 128) as usize;
			let dx = (rng.next_u32() % 8) as i32 - 4;
			let dy = (rng.next_u32() % 8) as i32 - 4;
			let red = self.data[y * 128 * 3 + x * 3 + 0];
			let green = self.data[y * 128 * 3 + x * 3 + 1];
			let blue = self.data[y * 128 * 3 + x * 3 + 2];
			if red | green | blue != 0 {
				let color = rng.next_u32();
				let r;
				let g;
				let b;
				r = (red >> 3).overflowing_add(color as u8 & 0b11).0;
				g = (green >> 2).overflowing_add(((color >> 2) & 0b11) as u8).0;
				b = (blue >> 3).overflowing_add(((color >> 4) & 0b11) as u8).0;
				let p = Rectangle::new(Point::new(x as i32 + dx, y as i32 + dy), Size::new(1, 1));
				let s = PrimitiveStyleBuilder::new().fill_color(Rgb565::new(r, g, b)).build();
				p.draw_styled(&s, disp)?;
			}
		}
		self.iters.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
		Ok(true)
	}

	fn expired(&self) -> bool {
		self.iters.load(std::sync::atomic::Ordering::Relaxed) > 1000
	}

	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}

impl SimpleScreensaver {
	const fn new(id: &'static str, data: &'static [u8]) -> Self {
		if data.len() != 128 * 128 * 3 {
			panic!("invalid screensaver size");
		}
		SimpleScreensaver {
			id,
			data,
			iters: AtomicU32::new(0),
		}
	}

	pub fn get_pixel(&self, x: u32, y: u32) -> Rgb565 {
		let idx = y as usize * 128 + x as usize;
		let (red, green, blue) = (self.data[3 * idx], self.data[3 * idx + 1], self.data[3 * idx + 2]);
		Rgb565::new(red >> 3, green >> 2, blue >> 3)
	}

	pub fn draw_all_colored<D: DrawTarget<Color = Rgb565>>(&self, disp: &mut D, color: Rgb565) -> Result<(), D::Error> {
		disp.fill_contiguous(
			&Rectangle::new((0, 0).into(), (128, 128).into()),
			(0..128 * 128).map(|idx| {
				let (red, green, blue) = (self.data[3 * idx], self.data[3 * idx + 1], self.data[3 * idx + 2]);
				let r = red >> 3;
				let g = green >> 2;
				let b = blue >> 3;
				if (r, g, b) != (0, 0, 0) {
					color
				} else {
					Rgb565::BLACK
				}
			}),
		)?;
		Ok(())
	}

	pub fn draw_all<D: DrawTarget<Color = Rgb565>>(&self, disp: &mut D, flipped: bool) -> Result<(), D::Error> {
		disp.fill_contiguous(
			&Rectangle::new((0, 0).into(), (128, 128).into()),
			(0..128 * 128).map(|idx| {
				let (mut red, mut green, mut blue) =
					(self.data[3 * idx], self.data[3 * idx + 1], self.data[3 * idx + 2]);
				if flipped {
					red = 255 - red;
					green = 255 - green;
					blue = 255 - blue;
				}
				let r = red >> 3;
				let g = green >> 2;
				let b = blue >> 3;
				Rgb565::new(r, g, b)
			}),
		)?;
		Ok(())
	}
}

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

pub static STAR: SimpleScreensaver = SimpleScreensaver::new("star", include_bytes!("./star.raw"));
pub static RPI: SimpleScreensaver = SimpleScreensaver::new("rpi", include_bytes!("./rpi.raw"));
pub static DUOLINGO: SimpleScreensaver = SimpleScreensaver::new("duolingo", include_bytes!("./duolingo.raw"));
pub static SPAGHETTI: SimpleScreensaver = SimpleScreensaver::new("spaghetti", include_bytes!("./spaghetti.raw"));
pub static PLATE: SimpleScreensaver = SimpleScreensaver::new("plate", include_bytes!("./plate.raw"));
pub static GITHUB: SimpleScreensaver = SimpleScreensaver::new("github", include_bytes!("./github.raw"));
pub static TEDDY_BEAR: SimpleScreensaver = SimpleScreensaver::new("teddy_bear", include_bytes!("./teddy_bear.raw"));

pub fn screensavers<D: DrawTarget<Color = Rgb565>>() -> Vec<Box<dyn Screensaver<D>>> {
	vec![
		Box::new(STAR.clone()),
		Box::new(RPI.clone()),
		Box::new(DUOLINGO.clone()),
		Box::new(SPAGHETTI.clone()),
		Box::new(PLATE.clone()),
	]
}

pub struct BearReminder;

impl Default for BearReminder {
	fn default() -> Self {
		Self {}
	}
}

static LAST_REMINDER: AtomicI32 = AtomicI32::new(0);

impl<D: DrawTarget<Color = Rgb565>> Schedule<D> for BearReminder {
	fn check(&self, _ctx: &dyn Context<D>, time: OffsetDateTime) -> bool {
		let day = time.to_julian_day();
		let good_time = time.hour() == 22 && time.minute() == 0 && day % 2 == 1;
		if !good_time {
			return false;
		}
		let last_day = LAST_REMINDER.load(Ordering::Relaxed);
		if last_day == day {
			return false;
		}
		let do_it = LAST_REMINDER.compare_exchange(last_day, day, Ordering::Relaxed, Ordering::Relaxed);
		do_it.is_ok()
	}

	fn execute(&self, ctx: &dyn Context<D>, _time: OffsetDateTime) {
		ctx.do_draw(Box::new(BearDraw { calls: RefCell::new(0) }));
	}
}

struct BearDraw {
	calls: RefCell<usize>,
}

impl<D: DrawTarget<Color = Rgb565>> Draw<D> for BearDraw {
	fn draw(&self, disp: &mut D, _rng: &mut Rng) -> Result<bool, <D as DrawTarget>::Error> {
		let mut calls = self.calls.borrow_mut();
		*calls += 1;

		if *calls > 73 {
			return Ok(false);
		}

		TEDDY_BEAR.draw_all(disp, *calls % 8 >= 4)?;

		Ok(true)
	}

	fn expired(&self) -> bool {
		*self.calls.borrow() > 110
	}

	fn as_any(&self) -> &dyn Any {
		&*self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		&mut *self
	}
}
