use std::any::Any;
use std::cell::RefCell;
use std::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, Ordering};

use embedded_graphics::prelude::RgbColor;
use embedded_graphics::{
	pixelcolor::Rgb565,
	prelude::{DrawTarget, Point, Size},
	primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
};
use rand_xoshiro::rand_core::RngCore;
use raspi_lib::{Draw, Screensaver};
use time::{OffsetDateTime, Weekday};

use crate::context::{Context, Rng};
use crate::schedule::Schedule;

pub static SPEED: AtomicU64 = AtomicU64::new(32);

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
				r = (red >> 3).saturating_add(color as u8 & 0b11).min(0b11111);
				g = (green >> 2).saturating_add(((color >> 2) & 0b11) as u8).min(0b111111);
				b = (blue >> 3).saturating_add(((color >> 4) & 0b11) as u8).min(0b11111);
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
		let day = time.weekday();
		let day_match = day == Weekday::Monday || day == Weekday::Wednesday || day == Weekday::Friday;
		let good_time =
			time.hour() == 20 && (time.minute() == 0 || time.minute() == 30 || time.minute() == 55) && day_match;
		if !good_time {
			return false;
		}
		let day_j = time.to_julian_day();
		let last_day = LAST_REMINDER.load(Ordering::Relaxed);
		if last_day == day_j {
			return false;
		}
		let do_it = LAST_REMINDER.compare_exchange(last_day, day_j, Ordering::Relaxed, Ordering::Relaxed);
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
