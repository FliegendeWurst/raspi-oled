use std::{
	sync::atomic::AtomicBool,
	thread::sleep,
	time::{self, Duration},
};

use embedded_graphics::{
	draw_target::DrawTarget,
	pixelcolor::Rgb565,
	prelude::{OriginDimensions, RgbColor, Size},
};
use gpiocdev::line::{Bias, EdgeKind, Value};
#[cfg(feature = "pc")]
use image::{ImageBuffer, Rgb};

#[cfg(feature = "pc")]
pub struct FrameOutput {
	pub buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

#[cfg(feature = "pc")]
impl FrameOutput {
	pub fn new(width: u32, height: u32) -> Self {
		FrameOutput {
			buffer: ImageBuffer::new(width, height),
		}
	}
}

#[cfg(feature = "pc")]
impl DrawTarget for FrameOutput {
	type Color = Rgb565;

	type Error = ();

	fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
	where
		I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
	{
		for pos in pixels {
			if pos.0.x < 0
				|| pos.0.y < 0 || pos.0.x as u32 >= self.buffer.width()
				|| pos.0.y as u32 >= self.buffer.height()
			{
				continue;
			}
			self.buffer.put_pixel(
				pos.0.x as u32,
				pos.0.y as u32,
				Rgb([pos.1.r() << 3, pos.1.g() << 2, pos.1.b() << 3]),
			);
		}
		Ok(())
	}
}

#[cfg(feature = "pc")]
impl OriginDimensions for FrameOutput {
	fn size(&self) -> Size {
		Size::new(self.buffer.width(), self.buffer.height())
	}
}

fn read_events(timeout: std::time::Duration) -> Result<Vec<(u64, EdgeKind)>, SensorError> {
	let input = gpiocdev::Request::builder()
		.on_chip("/dev/gpiochip0")
		.with_line(26)
		.as_input()
		//.with_edge_detection(EdgeDetection::BothEdges)
		//.with_debounce_period(Duration::ZERO)
		.with_kernel_event_buffer_size(1024)
		.with_bias(Bias::PullDown)
		.request()?;

	let start = time::Instant::now();
	let mut last_value = Value::Active;

	let mut events = Vec::with_capacity(81);
	while start.elapsed() < timeout && events.len() < 81 {
		let new_value = input.value(26)?;
		if new_value != last_value {
			match new_value {
				Value::Inactive => events.push((start.elapsed().as_micros() as u64, EdgeKind::Falling)),
				Value::Active => events.push((start.elapsed().as_micros() as u64, EdgeKind::Rising)),
			}
			last_value = new_value;
		}
		/*
		if input.wait_edge_event(timeout)? {
			let event = input.read_edge_event()?;
			events.push((start.elapsed().as_micros() as u64, event.kind));
		}
		*/
	}
	if events.len() < 81 {
		println!("error: only got {} events: {:?}", events.len(), events);
		return Err(SensorError::Timeout);
	}
	Ok(events)
}

fn events_to_data(events: Vec<(u64, EdgeKind)>) -> Vec<u8> {
	events[1..]
		.windows(2)
		.map(|pair| {
			let prev = pair.get(0).unwrap();
			let next = pair.get(1).unwrap();
			match next.1 {
				EdgeKind::Falling => Some(next.0 - prev.0),
				EdgeKind::Rising => None,
			}
		})
		.filter(|&d| d.is_some())
		.map(|elapsed| if elapsed.unwrap() > 35 { 1 } else { 0 })
		.collect()
}

const MAX_HUMIDITY: u16 = 1000;

fn process_data(mut bits: &[u8]) -> Result<(u16, u16), SensorError> {
	if bits[0] == 1 {
		// definitely incorrect first bit
		// (the humidity can't be this big..)
		bits = &bits[1..];
	}
	let bytes: Vec<u8> = bits
		.chunks(8)
		.map(|chunk| {
			chunk
				.iter()
				.enumerate()
				// 8 bits, starting with the MSB
				.map(|(bit_idx, &x)| x << (7 - bit_idx))
				.sum()
		})
		.collect();
	let rh = (bytes[0] as u16) << 8 | bytes[1] as u16;
	if rh > MAX_HUMIDITY {
		return Err(SensorError::HumidityTooHigh);
	}
	let celsius = (bytes[2] as u16) << 8 | bytes[3] as u16;

	if bits.len() >= 40 {
		let cksum: u8 = bits[32..40].iter().enumerate().map(|(idx, &x)| x << (7 - idx)).sum();
		let actual_sum = (bytes[0]
			.wrapping_add(bytes[1])
			.wrapping_add(bytes[2])
			.wrapping_add(bytes[3]))
			& 0xff;
		if actual_sum != cksum {
			return Err(SensorError::ChecksumMismatch);
		}
	}
	Ok((rh, celsius))
}

#[test]
fn test_process_data() {
	let x = process_data(&[
		1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0,
		0, 1, 1,
	])
	.unwrap();
	assert_eq!(471, x.0);
	assert_eq!(268, x.1);
}

#[derive(Debug)]
pub enum SensorError {
	Io(gpiocdev::Error),
	ChecksumMismatch,
	HumidityTooHigh,
	Timeout,
}

impl From<gpiocdev::Error> for SensorError {
	fn from(error: gpiocdev::Error) -> Self {
		SensorError::Io(error)
	}
}

pub fn am2302_reading() -> Result<(u16, u16), SensorError> {
	let out = gpiocdev::Request::builder()
		.on_chip("/dev/gpiochip0")
		.with_line(26)
		.as_output(Value::Active)
		.request()?;
	out.set_value(26, Value::Active)?;
	sleep(Duration::from_millis(500));
	set_max_priority();
	out.set_value(26, Value::Inactive)?;
	sleep(Duration::from_millis(4));
	drop(out);
	/*
	// set low for 20 ms
	out.set_value(26, Value::Inactive)?;
	sleep(Duration::from_millis(3));
	drop(out);
	*/

	let events = read_events(Duration::from_secs(1));
	println!("{:?} {:?}", events, events.as_ref().map(|x| x.len()));
	set_normal_priority();
	let events = events?;
	let data = events_to_data(events);
	process_data(&data)
}

fn set_max_priority() {
	unsafe {
		let mut sched_para: libc::sched_param = std::mem::transmute([0u8; std::mem::size_of::<libc::sched_param>()]);
		sched_para.sched_priority = libc::sched_get_priority_max(libc::SCHED_FIFO);
		libc::sched_setscheduler(0, libc::SCHED_FIFO, (&sched_para) as *const libc::sched_param);
	}
}

fn set_normal_priority() {
	unsafe {
		let sched_para: libc::sched_param = std::mem::transmute([0u8; std::mem::size_of::<libc::sched_param>()]);
		libc::sched_setscheduler(0, libc::SCHED_OTHER, (&sched_para) as *const libc::sched_param);
	}
}

pub fn disable_pwm() -> Result<(), rppal::pwm::Error> {
	/*
	let pwm = Pwm::new(rppal::pwm::Channel::Pwm0)?;
	if pwm.is_enabled()? {
		pwm.disable()?;
	}
	*/
	PWM_ON.store(false, std::sync::atomic::Ordering::Relaxed);
	Ok(())
}

pub fn enable_pwm() -> Result<(), rppal::pwm::Error> {
	PWM_ON.store(true, std::sync::atomic::Ordering::Relaxed);
	/*
	let mut pwm = Pwm::with_period(
		rppal::pwm::Channel::Pwm0,
		Duration::from_micros(500),
		Duration::from_micros(250),
		rppal::pwm::Polarity::Normal,
		true,
	)?;
	assert!(pwm.is_enabled()?);
	pwm.set_reset_on_drop(false);
	*/
	Ok(())
}

pub static PWM_ON: AtomicBool = AtomicBool::new(false);

use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Events {
	pub events: Vec<Event>,
	pub weekly: Vec<Weekly>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
	pub name: String,
	pub start_time: String,
	pub end_time: Option<String>,
}

#[derive(Deserialize)]
pub struct Weekly {
	pub name: String,
	pub day: i32,
	pub hour: i32,
	pub minute: i32,
	pub duration: i32,
}
