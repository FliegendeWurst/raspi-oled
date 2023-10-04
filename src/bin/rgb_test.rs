use std::{
	thread,
	time::{Duration, Instant},
};

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
	draw_target::DrawTarget,
	mono_font::{ascii::FONT_10X20, MonoTextStyleBuilder},
	pixelcolor::Rgb565,
	text::Text,
	Drawable,
};

use rppal::{
	gpio::Gpio,
	hal::Delay,
	spi::{Bus, Mode, SlaveSelect, Spi},
};
//use ssd1351::{properties::DisplaySize, mode::{GraphicsMode, displaymode::DisplayModeTrait}};

//use time_tz::{timezones::db::europe::BERLIN, OffsetDateTimeExt, PrimitiveDateTimeExt};

fn main() {
	//let i2c = I2cdev::new("/dev/i2c-1").unwrap();
	//let interface = I2CDisplayInterface::new(i2c);
	//let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();
	//disp.init().unwrap();
	// Configure gpio
	let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 19660800, Mode::Mode0).unwrap();
	let gpio = Gpio::new().unwrap();
	let dc = gpio.get(25).unwrap().into_output();
	let mut rst = gpio.get(27).unwrap().into_output();

	// Init SPI
	let spii = SPIInterfaceNoCS::new(spi, dc);
	let mut disp = ssd1351::display::display::Ssd1351::new(spii);

	// Reset & init
	disp.reset(&mut rst, &mut Delay).unwrap();
	disp.turn_on().unwrap();

	/*
	thread::sleep(Duration::from_secs(5));
	disp.reset(&mut rst, &mut Delay).unwrap();
	disp.turn_off().unwrap();
	panic!("done!");
	*/

	// Clear the display
	disp.clear(Rgb565::new(0x00, 0x00, 0x00)).unwrap();

	//disp.flush().unwrap();
	//disp.flush().unwrap();

	let text_style_clock = MonoTextStyleBuilder::new()
		.font(&FONT_10X20)
		.text_color(Rgb565::new(0xff, 0x00, 0x00))
		.build();

	//let text = format!("{}.{}% {}.{}°C", rh / 10, rh % 10, temp / 10, temp % 10);
	//Text::new(&text, Point::new(0, 10), text_style).draw(&mut disp).unwrap();
	Text::new("Abc", (0, 30).into(), text_style_clock)
		.draw(&mut disp)
		.unwrap();

	let _ = disp.flush();

	thread::sleep(Duration::from_secs(15));
	Text::new("5 seconds to off!", (0, 60).into(), text_style_clock)
		.draw(&mut disp)
		.unwrap();
	let start = Instant::now();
	let _ = disp.flush();
	println!("{:?} ms", start.elapsed().as_millis());
	thread::sleep(Duration::from_secs(5));

	disp.reset(&mut rst, &mut Delay).unwrap();
	disp.turn_off().unwrap();
}
