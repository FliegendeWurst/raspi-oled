use display_interface_spi::SPIInterface;
use rppal::{
	gpio::Gpio,
	hal::Delay,
	spi::{Bus, Mode, SimpleHalSpiDevice, SlaveSelect, Spi},
};

fn main() {
	let args = std::env::args().collect::<Vec<_>>();
	if args.len() < 2 {
		panic!("missing argument: on/off");
	}
	display_on_ssd1306(args[1] == "on");
}

fn display_on_ssd1306(on: bool) {
	let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 19660800, Mode::Mode0).unwrap();
	let gpio = Gpio::new().unwrap();
	let dc = gpio.get(25).unwrap().into_output();
	let mut rst = gpio.get(27).unwrap().into_output();

	// Init SPI
	let spii = SPIInterface::new(SimpleHalSpiDevice::new(spi), dc);
	let mut disp = ssd1351::display::display::Ssd1351::new(spii);

	// Reset & init
	disp.reset(&mut rst, &mut Delay).unwrap();
	if on {
		disp.turn_on().unwrap();
	} else {
		disp.turn_off().unwrap();
	}
}
