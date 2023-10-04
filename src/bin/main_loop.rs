use std::{
	cell::RefCell,
	time::{Duration, Instant},
};

use action::Action;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget, Drawable};
use gpiocdev::line::{Bias, EdgeDetection, Value};
use rand_xoshiro::{rand_core::SeedableRng, Xoroshiro128StarStar};
use raspi_oled::FrameOutput;
use rppal::{
	gpio::{Gpio, OutputPin},
	hal::Delay,
	spi::{Bus, Mode, SlaveSelect, Spi},
};
use schedule::Schedule;
use screensaver::{Screensaver, TimeDisplay};
use ssd1351::display::display::Ssd1351;
use time::OffsetDateTime;
use time_tz::{timezones::db::europe::BERLIN, OffsetDateTimeExt};

mod action;
mod schedule;
mod screensaver;

pub type Oled = Ssd1351<SPIInterfaceNoCS<Spi, OutputPin>>;
pub type Rng = Xoroshiro128StarStar;

static BLACK: Rgb565 = Rgb565::new(0, 0, 0);
static TIME_COLOR: Rgb565 = Rgb565::new(0b01_111, 0b011_111, 0b01_111);

fn main() {
	if rppal::system::DeviceInfo::new().is_ok() {
		rpi_main();
	} else {
		pc_main();
	}
}

pub trait Context {
	fn do_action(&self, action: Action);
}

struct ContextDefault {
	screensavers: Vec<Box<dyn Screensaver<Oled>>>,
	scheduled: Vec<Box<dyn Schedule>>,
	active: RefCell<Vec<Box<dyn Draw<Oled>>>>,
}

impl ContextDefault {
	fn new() -> Self {
		ContextDefault {
			screensavers: screensaver::screensavers(),
			scheduled: schedule::reminders(),
			active: RefCell::new(vec![Box::new(TimeDisplay::new())]),
		}
	}

	fn loop_iter(&mut self, disp: &mut Oled, rng: &mut Rng) -> bool {
		let time = OffsetDateTime::now_utc().to_timezone(BERLIN);
		// check schedules
		for s in &self.scheduled {
			s.check_and_do(&*self, time);
		}
		let active = self.active.borrow();
		if active.is_empty() {
			return false;
		}
		let a = active.last().unwrap();
		a.draw(disp, rng).unwrap_or(true)
	}
}

impl Context for ContextDefault {
	fn do_action(&self, action: Action) {
		match action {
			Action::Screensaver(id) => {
				for s in &self.screensavers {
					if s.id() == id {
						self.active.borrow_mut().push(s.convert_draw());
					}
				}
			},
		}
	}
}

#[cfg(not(feature = "pc"))]
fn pc_main() {}

#[cfg(feature = "pc")]
fn pc_main() {
	use std::num::NonZeroU32;

	use rand_xoshiro::{rand_core::SeedableRng, Xoroshiro128StarStar};
	use winit::{
		dpi::LogicalSize,
		event::{Event, WindowEvent},
		event_loop::EventLoop,
		window::WindowBuilder,
	};

	use crate::screensaver::{Screensaver, DUOLINGO};

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_inner_size(LogicalSize::new(128, 128))
		.build(&event_loop)
		.unwrap();
	let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
	let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

	let start = Instant::now();
	let mut iters = 0;
	let mut disp = FrameOutput::new(128, 128);
	//disp.buffer.save("/tmp/x.png").unwrap();
	let mut rng = Xoroshiro128StarStar::seed_from_u64(0);
	let mut buffer_dirty = true;

	event_loop.run(move |event, _, control_flow| {
		// ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
		// dispatched any events. This is ideal for games and similar applications.
		control_flow.set_poll();

		match event {
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => {
				println!("The close button was pressed; stopping");
				control_flow.set_exit();
			},
			Event::MainEventsCleared => {
				// Application update code.

				// Queue a RedrawRequested event.
				//
				// You only need to call this if you've determined that you need to redraw, in
				// applications which do not always need to. Applications that redraw continuously
				// can just render here instead.
				window.request_redraw();
			},
			Event::RedrawRequested(window_id) if window_id == window.id() => {
				// Redraw the application.
				//
				// It's preferable for applications that do not render continuously to render in
				// this event rather than in MainEventsCleared, since rendering in here allows
				// the program to gracefully handle redraws requested by the OS.
				let (width, height) = {
					let size = window.inner_size();
					(size.width, size.height)
				};
				surface
					.resize(NonZeroU32::new(width).unwrap(), NonZeroU32::new(height).unwrap())
					.unwrap();

				// redraw
				if Instant::now().duration_since(start) > Duration::from_millis(iters * 1000) {
					iters += 1;
					//loop_iter(&mut disp).unwrap();
					/*
					let mut time = OffsetDateTime::now_utc().to_timezone(BERLIN);
					//time += Duration::new(iters * 60, 0);
					disp.clear(Rgb565::new(0, 0, 0)).unwrap();
					display_clock(&mut disp, &time).unwrap();
					*/
					DUOLINGO.draw(&mut disp, &mut rng).unwrap();
					/*
					let iters = iters % 300;
					let (s, c) = (iters as f32 * 0.1).sin_cos();
					let (mut x, mut y) = (s * iters as f32 * 0.005, c * iters as f32 * 0.005);
					x *= 64.;
					y *= 64.;
					x += 64.;
					y += 64.;
					let variation = iters as u32 / 16 + 1;
					for _ in 0..16 {
						let dx = (rng.next_u32() % variation) as i32 - variation as i32 / 2;
						let dy = (rng.next_u32() % variation) as i32 - variation as i32 / 2;
						let color = rng.next_u32();
						let p = Rectangle::new(Point::new(x as i32 + dx, y as i32 + dy), Size::new(1, 1));
						let s = PrimitiveStyleBuilder::new()
							.fill_color(Rgb565::new(
								color as u8 & 0b11111,
								((color >> 8) & 0b111111) as u8,
								((color >> 16) & 0b111111) as u8,
							))
							.build();
						p.draw_styled(&s, &mut disp).unwrap();
					}
					if iters % 300 == 0 {
						disp.clear(Rgb565::new(0, 0, 0)).unwrap();
					}
					*/
					/*
					for _ in 0..16 {
						let x = (rng.next_u32() % 128) as usize;
						let y = (rng.next_u32() % 128) as usize;
						let dx = (rng.next_u32() % 8) as i32 - 4;
						let dy = (rng.next_u32() % 8) as i32 - 4;
						let red = STAR[y * 128 * 3 + x * 3];
						let green = STAR[y * 128 * 3 + x * 3 + 1];
						if red == 0xff {
							let color = rng.next_u32();
							let r;
							let g;
							let b;
							// star
							r = (color as u8 & 0b11111).saturating_mul(2);
							g = (((color >> 8) & 0b111111) as u8).saturating_mul(2);
							b = ((color >> 16) & 0b111111) as u8 / 3;
							// rpi
							/*
							if red > green {
								r = (color as u8 & 0b11111).saturating_mul(2);
								g = ((color >> 8) & 0b111111) as u8 / 3;
								b = ((color >> 16) & 0b111111) as u8 / 3;
							} else {
								r = (color as u8 & 0b11111) / 2;
								g = (((color >> 8) & 0b111111) as u8).saturating_mul(2);
								b = ((color >> 16) & 0b111111) as u8 / 3;
							}
							*/
							let p = Rectangle::new(Point::new(x as i32 + dx, y as i32 + dy), Size::new(1, 1));
							let s = PrimitiveStyleBuilder::new()
								.fill_color(Rgb565::new(r as u8, g as u8, b as u8))
								.build();
							p.draw_styled(&s, &mut disp).unwrap();
						}
					}
					*/
					buffer_dirty = true;
				}

				let mut buffer = surface.buffer_mut().unwrap();
				if buffer_dirty {
					for index in 0..(width * height) {
						let y = index / width;
						let x = index % width;
						let pixel = disp.buffer.get_pixel(x, y);
						let red = pixel.0[0] << 0;
						let green = pixel.0[1] << 0;
						let blue = pixel.0[2] << 0;

						buffer[index as usize] = blue as u32 | ((green as u32) << 8) | ((red as u32) << 16);
					}
					buffer.present().unwrap();
					buffer_dirty = false;
				}
			},
			_ => (),
		}
	});
}

pub trait Draw<D: DrawTarget<Color = Rgb565>> {
	fn draw(&self, disp: &mut D, rng: &mut Rng) -> Result<bool, D::Error>;
}

fn rpi_main() {
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

	main_loop(disp);
}

fn main_loop(mut disp: Oled) {
	disp.clear(BLACK).unwrap();

	let mut ctx = ContextDefault::new();
	let mut rng = Xoroshiro128StarStar::seed_from_u64(17381);
	let mut last_button = Instant::now();

	let mut menu = vec![];
	// high pins for buttons
	let _high_outputs = gpiocdev::Request::builder()
		.on_chip("/dev/gpiochip0")
		.with_lines(&[23, 24])
		.as_output(Value::Active)
		.request()
		.unwrap();
	let lines = gpiocdev::Request::builder()
		.on_chip("/dev/gpiochip0")
		.with_line(19)
		.with_edge_detection(EdgeDetection::RisingEdge)
		.with_debounce_period(Duration::from_millis(5))
		.with_bias(Bias::PullDown)
		.with_line(6)
		.with_edge_detection(EdgeDetection::RisingEdge)
		.with_debounce_period(Duration::from_millis(5))
		.with_bias(Bias::PullDown)
		.with_line(5)
		.with_edge_detection(EdgeDetection::RisingEdge)
		.with_debounce_period(Duration::from_millis(5))
		.with_bias(Bias::PullDown)
		.request()
		.unwrap();

	loop {
		// respond to button presses
		while lines.wait_edge_event(Duration::from_millis(1)).unwrap() {
			let e = lines.read_edge_event().unwrap();
			last_button = Instant::now();
			match e.offset {
				19 => {
					menu.push(1);
				},
				6 => {
					menu.push(2);
				},
				5 => {
					menu.push(3);
				},
				_ => {
					println!("unknown offset: {}", e.offset);
				},
			}
			println!("menu: {menu:?}");
		}
		// clean up stale menu selection
		if !menu.is_empty() && Instant::now().duration_since(last_button).as_secs() >= 10 {
			menu.clear();
		}
		// check schedules
		let dirty = ctx.loop_iter(&mut disp, &mut rng);
		if dirty {
			let _ = disp.flush(); // ignore bus write errors, they are harmless
		}
	}
}
