use embedded_graphics::mono_font::MonoTextStyle;
pub use rand_xoshiro::Xoroshiro128StarStar as Rng;
use rand_xoshiro::rand_core::SeedableRng;

pub use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};

pub use embedded_graphics::mono_font::ascii::FONT_10X20;

pub use embedded_graphics::mono_font::MonoTextStyleBuilder;

pub const BLACK: Rgb565 = Rgb565::new(0, 0, 0);
pub const WHITE: Rgb565 = Rgb565::new(0xff, 0xff, 0xff);

pub static FONT: MonoTextStyle<'static, Rgb565> =
	MonoTextStyleBuilder::new().font(&FONT_10X20).text_color(WHITE).build();

pub use embedded_graphics::Pixel;
pub use embedded_graphics::prelude::Point;
pub use embedded_graphics::text::Text;

pub use embedded_graphics::Drawable;
pub use embedded_graphics::primitives::Rectangle;

mod context;
pub use context::Draw;

mod screensaver;
pub use screensaver::Screensaver;

mod time_display;
pub use time_display::TimeDisplay;

pub fn new_rng() -> Rng {
	let seed = getrandom::u64().unwrap();
	Rng::seed_from_u64(seed)
}
