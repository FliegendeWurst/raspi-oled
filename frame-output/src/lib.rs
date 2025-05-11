use embedded_graphics::{
	pixelcolor::Rgb565,
	prelude::{Dimensions, DrawTarget, RgbColor},
	primitives::Rectangle,
};
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgb, Rgba};

pub struct FrameOutput {
	pub buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl FrameOutput {
	pub fn new(width: u32, height: u32) -> Self {
		FrameOutput {
			buffer: ImageBuffer::new(width, height),
		}
	}

	pub fn copy_from(&mut self, other_img: &DynamicImage, x: u32, y: u32) {
		let _ = self.buffer.copy_from(other_img, x, y);
	}
}

impl DrawTarget for FrameOutput {
	type Color = Rgb565;

	type Error = ();

	fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
	where
		I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
	{
		for pos in pixels {
			if pos.0.x < 0
				|| pos.0.y < 0
				|| pos.0.x as u32 >= self.buffer.width()
				|| pos.0.y as u32 >= self.buffer.height()
			{
				continue;
			}
			self.buffer.put_pixel(
				pos.0.x as u32,
				pos.0.y as u32,
				Rgba([pos.1.r() << 3, pos.1.g() << 2, pos.1.b() << 3, 0xff]),
			);
		}
		Ok(())
	}

	fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
		let mut sub = self.buffer.sub_image(
			area.top_left.x as _,
			area.top_left.y as _,
			area.size.width,
			area.size.height,
		);
		let rgb = Rgba([color.r() << 3, color.g() << 2, color.b() << 3, 0xff]);
		for y in 0..sub.height() {
			for x in 0..sub.width() {
				sub.put_pixel(x, y, rgb);
			}
		}
		Ok(())
	}
}

impl Dimensions for FrameOutput {
	fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
		Rectangle::new((0, 0).into(), (self.buffer.width(), self.buffer.height()).into())
	}
}
