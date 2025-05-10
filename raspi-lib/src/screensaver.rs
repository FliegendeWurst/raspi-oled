use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};

use crate::Draw;

pub trait Screensaver<D: DrawTarget<Color = Rgb565>>: Draw<D> {
	fn id(&self) -> &'static str;
	fn convert_draw(&self) -> Box<dyn Draw<D>>;
}