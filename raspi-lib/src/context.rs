use std::any::Any;

use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};

use crate::Rng;

pub trait Draw<D: DrawTarget<Color = Rgb565>> {
	fn draw(&self, disp: &mut D, rng: &mut Rng) -> Result<bool, D::Error>;
	fn expired(&self) -> bool {
		false
	}
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}