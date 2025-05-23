#![feature(array_windows)]

use std::{
	env, thread,
	time::{Duration, Instant},
};

use display_interface_spi::SPIInterface;
use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};
use gpiocdev::line::{Bias, EdgeDetection, Value};
use rand_xoshiro::{rand_core::SeedableRng, Xoroshiro128StarStar};
use raspi_oled::draw::Totp;
use raspi_oled::{
	action::Action,
	context::{Context, ContextDefault},
};
use raspi_oled::{disable_pwm, enable_pwm, PWM_ON};
use rppal::{
	gpio::{Gpio, OutputPin},
	hal::Delay,
	spi::{Bus, Mode, SimpleHalSpiDevice, SlaveSelect, Spi},
};
use ssd1351::display::display::Ssd1351;

pub type Oled = Ssd1351<SPIInterface<SimpleHalSpiDevice, OutputPin>>;

static BLACK: Rgb565 = Rgb565::new(0, 0, 0);
/// Delay after drawing a frame in milliseconds.
const FRAME_INTERVAL: u64 = 66;

fn main() {
	if rppal::system::DeviceInfo::new().is_ok() {
		rpi_main();
	} else {
		pc_main();
	}
}

#[cfg(not(feature = "pc"))]
fn pc_main() {}

#[cfg(feature = "pc")]
fn pc_main() {
	use std::num::NonZeroU32;

	use winit::{
		dpi::LogicalSize,
		event::{Event, WindowEvent},
		event_loop::EventLoop,
		window::WindowBuilder,
	};

	use raspi_oled::{
		context::{Context, ContextDefault},
		screensaver, FrameOutput,
	};

	let args: Vec<_> = env::args().map(|x| x.to_string()).collect();
	for [key, val] in args.array_windows() {
		match key.as_str() {
			"--speed" => {
				screensaver::SPEED.store(val.parse().unwrap(), std::sync::atomic::Ordering::Relaxed);
			},
			_ => {},
		}
	}

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
	let mut buffer_dirty = true;

	let mut ctx = ContextDefault::new();
	if args.iter().any(|x| x == "--totp") {
		let pw = rpassword::prompt_password("TOTP password: ").unwrap();
		let totps = andotp_import::read_from_file("./otp_accounts_2023-10-02_18-58-25.json.aes", &pw).unwrap();
		ctx.add(Totp::new(totps));
		ctx.do_action(Action::Screensaver("totp"));
	}
	let mut rng = Xoroshiro128StarStar::seed_from_u64(17381);

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

				// ignore window.inner_size() for HiDPI scaling
				let width = 128;
				let height = 128;
				surface
					.resize(NonZeroU32::new(width).unwrap(), NonZeroU32::new(height).unwrap())
					.unwrap();

				// redraw
				if Instant::now().duration_since(start) > Duration::from_millis(iters * FRAME_INTERVAL) {
					iters += 1;
					buffer_dirty = ctx.loop_iter(&mut disp, &mut rng);
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
					let _ = disp.buffer.save(format!("/tmp/iter{}.png", iters));
				}
			},
			_ => (),
		}
	});
}

fn rpi_main() {
	let args: Vec<_> = env::args().map(|x| x.to_string()).collect();

	let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 19660800, Mode::Mode0).unwrap();
	let gpio = Gpio::new().unwrap();
	let dc = gpio.get(25).unwrap().into_output();
	let mut rst = gpio.get(27).unwrap().into_output();

	// Init SPI
	let spii = SPIInterface::new(SimpleHalSpiDevice::new(spi), dc);
	let mut disp = Ssd1351::new(spii);

	// Reset & init display
	disp.reset(&mut rst, &mut Delay).unwrap();
	disp.turn_on().unwrap();

	// Init PWM handling
	let pwm = thread::spawn(handle_pwm);

	let mut ctx = ContextDefault::new();
	if args.iter().any(|x| x == "--totp") {
		let pw = rpassword::prompt_password("TOTP password: ").unwrap();
		let totps = andotp_import::read_from_file("./otp_accounts_2023-10-02_18-58-25.json.aes", &pw).unwrap();
		ctx.add(Totp::new(totps));
	}

	main_loop(disp, ctx);

	let _ = pwm.join();
}

fn handle_pwm() {
	let pwm = gpiocdev::Request::builder()
		.on_chip("/dev/gpiochip0")
		.with_line(12)
		.as_output(Value::Inactive)
		.request()
		.unwrap();
	loop {
		thread::sleep(Duration::from_millis(500));
		let on = PWM_ON.load(std::sync::atomic::Ordering::Relaxed);
		if !on {
			let _ = pwm.set_value(12, Value::Inactive);
			continue;
		}
		for _ in 0..100 {
			let _ = pwm.set_value(12, Value::Active);
			thread::sleep(Duration::from_millis(1));
			let _ = pwm.set_value(12, Value::Inactive);
			thread::sleep(Duration::from_millis(1));
		}
	}
}

fn main_loop(mut disp: Oled, mut ctx: ContextDefault<Oled>) {
	disp.clear(BLACK).unwrap();

	let mut rng = Xoroshiro128StarStar::seed_from_u64(17381);
	let mut last_button = Instant::now();

	let mut menu = vec![];
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
				5 => {
					menu.push(1);
				},
				6 => {
					menu.push(2);
				},
				19 => {
					menu.push(3);
				},
				_ => {
					println!("unknown offset: {}", e.offset);
				},
			}
			let mut pop_last = false;
			let mut clear = false;
			match &*menu {
				[1] => {
					ctx.do_action(Action::Screensaver("measurements"));
				},
				[1, 2] => {
					let _ = ctx.pop_action_and_clear(&mut disp);
					ctx.do_action(Action::Screensaver("measurements_temps"));
					pop_last = true;
				},
				[1, 3] => {
					let _ = ctx.pop_action_and_clear(&mut disp);
					ctx.do_action(Action::Screensaver("measurements_events"));
					pop_last = true;
				},
				[2] => {
					if ctx.active_count() > 1 {
						let _ = ctx.pop_action_and_clear(&mut disp);
						disable_pwm().unwrap();
						let _ = disp.flush();
						clear = true;
					}
				},
				[3] => {
					ctx.do_action(Action::Screensaver("totp"));
				},
				[3, 1] => {
					if let Some(x) = ctx.active.borrow_mut().last_mut() {
						let totp: Option<&mut Totp> = x.as_any_mut().downcast_mut();
						if let Some(x) = totp {
							x.next_page();
						}
					}
					pop_last = true;
				},
				[3, 2, 1] => {
					enable_pwm().unwrap();
					pop_last = true;
				},
				[3, 2, 3] => {
					disable_pwm().unwrap();
					pop_last = true;
				},
				[3, 3] => {
					ctx.do_action(Action::Screensaver("rpi"));
					clear = true;
				},
				_ => {},
			}
			if pop_last {
				menu.pop();
			}
			if clear {
				menu.clear();
			}
		}
		// clean up stale menu selection
		if !menu.is_empty() && Instant::now().duration_since(last_button).as_secs() >= 10 {
			menu.clear();
		}
		// run context loop
		let dirty = ctx.loop_iter(&mut disp, &mut rng);
		if dirty {
			let _ = disp.flush(); // ignore bus write errors, they are harmless
		}
		thread::sleep(Duration::from_millis(FRAME_INTERVAL));
	}
}
