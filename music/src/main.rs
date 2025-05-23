#![feature(round_char_boundary, hash_extract_if)]

// build using
// nix build .#packages.x86_64-linux-cross-muslpi.music && nix store sign -k ~/.local/share/nix-store-binary-cache-key-secret $(readlink -f result) && nix copy --to ssh://pi@himbeere-mit-musik $(readlink -f result) && echo $(readlink -f result)

use std::{env, time::Duration};

use command::{get_volume, list_folders, set_volume, start_mpv};
use display_interface_spi::SPIInterface;
use gpiocdev::{
	Request,
	line::{Bias, EdgeDetection},
};
use mpv_status::MpvStatus;
use playerctl_rust_wrapper::Playerctl;
use raspi_lib::{BLACK, Draw, DrawTarget, Rng, TimeDisplay, new_rng};
use rppal::{
	gpio::Gpio,
	hal::Delay,
	spi::{Bus, Mode, SimpleHalSpiDevice, SlaveSelect, Spi},
};
use ssd1351::display::display::Ssd1351;
use ui::Ui;

mod command;
mod mpv_status;
mod ui;

const BUTTON_PINS: &[u32] = &[17, 22, 5, 6, 26, 16];

fn main() {
	let args: Vec<_> = env::args().map(|x| x.to_string()).collect();
	let mut rng = new_rng();

	if rppal::system::DeviceInfo::new().is_ok() {
		// Boot up pulseaudio socket
		let _ = get_volume();

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
		let _ = disp.clear(BLACK);

		let mut lines = gpiocdev::Request::builder();
		lines.on_chip("/dev/gpiochip0");
		for &line in &BUTTON_PINS[0..2] {
			lines
				.with_line(line)
				.with_edge_detection(EdgeDetection::RisingEdge)
				.with_debounce_period(Duration::from_millis(50))
				.with_bias(Bias::PullDown);
		}
		for &line in &BUTTON_PINS[2..6] {
			lines
				.with_line(line)
				.with_edge_detection(EdgeDetection::FallingEdge)
				.with_debounce_period(Duration::from_millis(50))
				.with_bias(Bias::PullUp);
		}
		let lines = lines.request().unwrap();

		real_main(disp, &mut rng, lines);
	} else {
		pc_main();
	}
}

#[cfg(not(feature = "pc"))]
fn pc_main() {}

#[cfg(feature = "pc")]
fn pc_main() {
	const FRAME_INTERVAL: u64 = 66;

	use std::{
		num::NonZeroU32,
		time::{Duration, Instant},
	};

	use frame_output::FrameOutput;
	use mpv_status::MpvStatus;
	use winit::{
		dpi::LogicalSize,
		event::{Event, WindowEvent},
		event_loop::EventLoop,
		window::WindowBuilder,
	};

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

	let mut rng = new_rng();
	let mpv = MpvStatus::new();

	event_loop.run(move |event, _, control_flow| {
		// ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
		// dispatched any events. This is ideal for games and similar applications.
		control_flow.set_poll();

		match event {
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => {
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
					// buffer_dirty = ctx.loop_iter(&mut disp, &mut rng);
					if let Ok(x) = mpv.draw(&mut disp, &mut rng) {
						buffer_dirty |= x;
					}
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
					if buffer_dirty {
						let _ = disp.buffer.save(format!("/tmp/iter{}.png", iters));
					}
					buffer_dirty = false;
				}
			},
			_ => (),
		}
	});
}

fn real_main(mut disp: Ssd1351<SPIInterface<SimpleHalSpiDevice, rppal::gpio::OutputPin>>, rng: &mut Rng, lines: Request) {
	let mut mpv = MpvStatus::new();
	let mut time = TimeDisplay::new();
	let mut active_ui: Option<Ui> = None;
	loop {
		// check user input
		while lines.has_edge_event() == Ok(true) {
			let ev = lines.read_edge_event().unwrap();
			let idx = BUTTON_PINS.iter().position(|&offset| offset == ev.offset).unwrap();
			if let Some(mut ai) = active_ui {
				let res = ai.handle(idx);
				match res {
					ui::UiResult::Ignore => active_ui = Some(ai),
					ui::UiResult::Close => {
						active_ui = None;
						time.redraw();
					},
					ui::UiResult::Replace(new_id) => {
						if new_id == "volume" {
							if let Ok(it) = get_volume() {
								active_ui = Some(Ui::new_aux1("volume", it));
							} else {
								active_ui = None;
							}
						} else {
							active_ui = Some(Ui::new(new_id));
						}
					},
					ui::UiResult::Play(folder) => {
						active_ui = None;
						time.redraw();
						start_mpv(&folder);
						mpv.re_request();
					},
				}
			} else {
				let mut show_vol = false;
				match idx {
					0 => {
						let dirs = list_folders();
						active_ui = Some(Ui::new_aux2("select", dirs));
					},
					1 if !mpv.active() => active_ui = Some(Ui::new("exit")),
					1 => {
						let _ = Playerctl::stop();
						mpv.re_request();
					},
					2 => {},
					3 => {
						// volume up
						set_volume("+5%");
						show_vol = true;
					},
					4 => {},
					5 => {
						// volume down
						set_volume("-5%");
						show_vol = true;
					},
					_ => unreachable!(),
				}
				if show_vol {
					if let Ok(vol) = get_volume() {
						active_ui = Some(Ui::new_aux1("volume", vol));
					}
				}
			}
		}
		let mut buffer_dirty = false;
		if let Some(d) = &active_ui {
			buffer_dirty |= d.draw(&mut disp, rng).unwrap();
			if d.should_close() {
				active_ui = None;
				time.redraw();
			}
		} else {
			let mpv_active = mpv.active();
			buffer_dirty |= mpv.draw(&mut disp, rng).unwrap();
			if !mpv.active() {
				if mpv_active {
					time.redraw();
				}
				buffer_dirty |= time.draw(&mut disp, rng).unwrap();
			}
		}
		if buffer_dirty {
			let _ = disp.flush();
		}
		sleep_ms(100);
	}
}

fn sleep_ms(ms: u32) {
	std::thread::sleep(Duration::from_millis(ms as _));
}
