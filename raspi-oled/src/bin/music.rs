use std::env;

use display_interface_spi::SPIInterfaceNoCS;
use rand_xoshiro::rand_core::SeedableRng;
use raspi_oled::{
	context::{Draw, Rng},
	screensaver::TimeDisplay,
};
use rppal::{
	gpio::Gpio,
	hal::Delay,
	spi::{Bus, Mode, SlaveSelect, Spi},
};
use ssd1351::display::display::Ssd1351;

fn main() {
	let args: Vec<_> = env::args().map(|x| x.to_string()).collect();
	let mut rng = Rng::seed_from_u64(17381);

	let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 19660800, Mode::Mode0).unwrap();
	let gpio = Gpio::new().unwrap();
	let dc = gpio.get(25).unwrap().into_output();
	let mut rst = gpio.get(27).unwrap().into_output();

	// Init SPI
	let spii = SPIInterfaceNoCS::new(spi, dc);
	let mut disp = Ssd1351::new(spii);

	// Reset & init display
	disp.reset(&mut rst, &mut Delay).unwrap();
	disp.turn_on().unwrap();

	let time = TimeDisplay::new();
	time.draw(&mut disp, &mut rng).unwrap();
}
