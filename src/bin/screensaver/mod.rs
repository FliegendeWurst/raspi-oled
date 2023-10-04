use embedded_graphics::{
	pixelcolor::Rgb565,
	prelude::{DrawTarget, Point, Size},
	primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
};
use rand_xoshiro::rand_core::RngCore;

use crate::Rng;

pub trait Screensaver {
	fn id(&self) -> &'static str;

	fn draw<D: DrawTarget<Color = Rgb565>>(&self, disp: &mut D, rng: &mut Rng) -> Result<(), D::Error>;
}

pub struct SimpleScreensaver {
	id: &'static str,
	data: &'static [u8],
}

impl Screensaver for SimpleScreensaver {
	fn id(&self) -> &'static str {
		self.id
	}

	fn draw<D: DrawTarget<Color = Rgb565>>(&self, disp: &mut D, rng: &mut Rng) -> Result<(), D::Error> {
		for _ in 0..512 {
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
		Ok(())
	}
}

impl SimpleScreensaver {
	const fn new(id: &'static str, data: &'static [u8]) -> Self {
		if data.len() != 128 * 128 * 3 {
			panic!("invalid screensaver size");
		}
		SimpleScreensaver { id, data }
	}
}

pub static STAR: SimpleScreensaver = SimpleScreensaver::new("star", include_bytes!("./star.raw"));
pub static RPI: SimpleScreensaver = SimpleScreensaver::new("rpi", include_bytes!("./rpi.raw"));
pub static DUOLINGO: SimpleScreensaver = SimpleScreensaver::new("duolingo", include_bytes!("./duolingo.raw"));
