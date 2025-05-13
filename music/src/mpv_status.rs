use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
	time::{Duration, Instant},
};

use base64::{Engine, prelude::BASE64_STANDARD};
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use playerctl_rust_wrapper::{PlayerMetadata, Playerctl};
use raspi_lib::{BLACK, Draw, DrawTarget, Drawable, FONT, Pixel, Point, Rectangle, Rgb565, Screensaver, Text};

#[derive(Clone)]
pub struct MpvStatus {
	prev_art: Arc<Mutex<Option<String>>>,
	fetched: Arc<Mutex<Instant>>,
	metadata: Arc<Mutex<HashMap<String, PlayerMetadata>>>,
	positions: Arc<Mutex<HashMap<String, u64>>>,
	start: Instant,
}

impl MpvStatus {
	pub fn new() -> Self {
		MpvStatus {
			prev_art: Arc::new(Mutex::new(None)),
			fetched: Arc::new(Mutex::new(Instant::now().checked_sub(Duration::from_secs(60)).unwrap())),
			metadata: Arc::new(Mutex::new(HashMap::new())),
			positions: Arc::new(Mutex::new(HashMap::new())),
			start: Instant::now(),
		}
	}

	pub fn re_request(&mut self) {
		let mut fetch = self.fetched.lock().unwrap();
		*fetch = fetch.checked_sub(Duration::from_secs(60)).unwrap();
	}

	pub fn active(&self) -> bool {
		self.metadata.lock().unwrap().contains_key("mpv")
	}
}

impl<D: DrawTarget<Color = Rgb565>> Screensaver<D> for MpvStatus {
	fn id(&self) -> &'static str {
		"mpv"
	}

	fn convert_draw(&self) -> Box<dyn raspi_lib::Draw<D>> {
		Box::new(self.clone())
	}
}

trait GenericCopyFrom {
	type Error;
	fn copy_from(&mut self, other: &DynamicImage, x: u32, y: u32) -> Result<(), Self::Error>;
}

impl<D: DrawTarget<Color = Rgb565>> GenericCopyFrom for D {
	type Error = D::Error;
	fn copy_from(&mut self, other: &DynamicImage, x: u32, y: u32) -> Result<(), D::Error> {
		self.draw_iter(
			(0..other.height())
				.map(|iy| (0..other.width()).map(move |ix| (ix, iy, other.get_pixel(ix, iy))))
				.flatten()
				.map(|(ix, iy, rgb)| {
					let r = rgb.0[0];
					let g = rgb.0[1];
					let b = rgb.0[2];
					Pixel(
						((x + ix) as i32, (y + iy) as i32).into(),
						Rgb565::new(r >> 3, g >> 2, b >> 3),
					)
				}),
		)?;
		Ok(())
	}
}

impl<D: DrawTarget<Color = Rgb565>> Draw<D> for MpvStatus {
	fn draw(&self, disp: &mut D, _rng: &mut raspi_lib::Rng) -> Result<bool, <D as raspi_lib::DrawTarget>::Error> {
		let now = Instant::now();
		let iters = now.duration_since(self.start).as_millis() as u64 / 200;
		let mut prev_art = self.prev_art.lock().unwrap();
		let mut metadata = self.metadata.lock().unwrap();
		let mut positions = self.positions.lock().unwrap();
		let mut fetched = self.fetched.lock().unwrap();
		let mut buffer_dirty = false;
		let mut metadata_changed = false;
		let mut positions_changed = false;
		if now.duration_since(*fetched).as_secs() > 10 {
			*fetched = now;
			if let Ok(mut new_metadata) = Playerctl::metadata() {
				new_metadata.extract_if(|k, _v| k != "mpv").for_each(|_| {});
				if new_metadata != *metadata {
					metadata_changed = true;
				}
				*metadata = new_metadata;
			} else {
				metadata.clear();
			}
			if let Ok(mut new_positions) = Playerctl::get_position() {
				new_positions.extract_if(|k, _v| k != "mpv").for_each(|_| {});
				if new_positions != *positions {
					positions_changed = true;
				}
				*positions = new_positions;
			} else {
				positions.clear();
			}
		}
		if let Some(d) = metadata.get("mpv") {
			if let Some(art) = &d.mpris_art_url {
				if prev_art.is_none() || prev_art.as_ref().unwrap() != art {
					*prev_art = Some(art.clone());
					if let Some(file) = art.strip_prefix("file://") {
						if let Ok(img) = ImageReader::open(file) {
							if let Ok(img) = img.decode() {
								let thumb = img.thumbnail(64, 64);
								buffer_dirty = true;
								disp.copy_from(&thumb, 4, (128 - thumb.height()) / 2)?; // or (128 - thumb.width()) / 2
							}
						}
					} else if let Some(encoded) = art.strip_prefix("data:image/jpeg;base64,") {
						if let Ok(decoded) = BASE64_STANDARD.decode(encoded) {
							if let Ok(img) = image::load_from_memory_with_format(&decoded, ImageFormat::Jpeg) {
								let thumb = img.thumbnail(64, 64);
								buffer_dirty = true;
								disp.copy_from(&thumb, 4, (128 - thumb.height()) / 2)?; // or (128 - thumb.width()) / 2
							}
						}
					}
				}
			} else {
				if prev_art.is_some() {
					*prev_art = None;
					disp.fill_solid(&Rectangle::new((4, 64).into(), (64, 64).into()), BLACK)?;
					buffer_dirty = true;
				}
			}
			macro_rules! draw_scrolling {
				($string:expr, $y:expr) => {
					let max_len = 128 / 10;
					if $string.len() <= max_len {
						let size = $string.len() * 10;
						let pad = (max_len * 10 - size) / 2;
						Text::new($string, Point::new(4 + pad as i32, $y), FONT).draw(disp)?;
						#[allow(unused_assignments)]
						{
							buffer_dirty |= metadata_changed;
						}
					} else {
						let pad = " ".repeat(max_len);
						let full = format!("{pad}{}{pad}  ", $string);
						let idx_start = iters as usize % (full.len() - max_len);
						Text::new(
							&full[full.ceil_char_boundary(idx_start)..full.floor_char_boundary(idx_start + max_len)],
							Point::new(4, $y),
							FONT,
						)
						.draw(disp)?;
						#[allow(unused_assignments)]
						{
							buffer_dirty = true;
						}
					}
				};
			}
			disp.fill_solid(&Rectangle::new((4, 20 - 15).into(), (128 - 4, 20).into()), BLACK)?;
			disp.fill_solid(&Rectangle::new((4, 116 - 15).into(), (128 - 4, 20).into()), BLACK)?;
			if let Some(artist) = &d.xesam_artist {
				draw_scrolling!(artist, 20);
				if let Some(title) = &d.xesam_title {
					draw_scrolling!(title, 116);
				}
			} else if let Some(title) = &d.xesam_title {
				if let Some((artist, song)) = title.split_once(" - ") {
					draw_scrolling!(artist, 20);
					draw_scrolling!(song, 116);
				} else {
					draw_scrolling!(title, 116);
				}
			}
			if let Some(prog) = positions.get("mpv") {
				disp.fill_solid(&Rectangle::new((68, 64 - 20 - 15).into(), (128 - 68, 20).into()), BLACK)?;
				let secs = prog / 1_000_000;
				let len = format!("{}:{:0>2}", secs / 60, secs % 60);
				Text::new(&len, Point::new(124 - 10 * len.len() as i32, 64 - 20), FONT).draw(disp)?;
				buffer_dirty |= positions_changed || metadata_changed;
			}
			if let Some(len) = d.mpris_length {
				disp.fill_solid(&Rectangle::new((68, 64 - 0 - 15).into(), (128 - 68, 20).into()), BLACK)?;
				let secs = len / 1_000_000;
				let len = format!("{}:{:0>2}", secs / 60, secs % 60);
				Text::new(&len, Point::new(124 - 10 * len.len() as i32, 64 - 0), FONT).draw(disp)?;
			}
		}
		Ok(buffer_dirty)
	}

	fn as_any(&self) -> &dyn std::any::Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		self
	}
}
