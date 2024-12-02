use std::{cell::RefCell, collections::HashSet};

use color_space::{Hsv, ToRgb};
use embedded_graphics::{
	mono_font::{iso_8859_10::FONT_8X13, MonoTextStyleBuilder},
	pixelcolor::Rgb565,
	prelude::{DrawTarget, Point, RgbColor},
	primitives::Rectangle,
	text::Text,
	Drawable, Pixel,
};
use rand_xoshiro::rand_core::RngCore;
use raspi_oled::github::get_new_notifications;

use crate::{
	screensaver::{SimpleScreensaver, GITHUB},
	Draw,
};

use super::Schedule;

pub struct GithubNotifications {
	pub pat: String,
	pub last_modified: RefCell<Option<String>>,
	pub last_call: RefCell<time::OffsetDateTime>,
}

impl<D: DrawTarget<Color = Rgb565>> Schedule<D> for GithubNotifications {
	fn check(&self, _ctx: &dyn crate::Context<D>, time: time::OffsetDateTime) -> bool {
		let time_since_last = time - *self.last_call.borrow();
		time_since_last.whole_minutes() >= 1
	}

	fn execute(&self, ctx: &dyn crate::Context<D>, time: time::OffsetDateTime) {
		*self.last_call.borrow_mut() = time;
		let last_modified = self.last_modified.borrow().clone();
		if let Ok((notifications, last_modified)) = get_new_notifications(&self.pat, last_modified.as_deref()) {
			*self.last_modified.borrow_mut() = last_modified;
			let relevant: Vec<_> = notifications
				.into_iter()
				.filter(|x| x.reason != "state_change" && x.unread)
				.collect();
			if relevant.is_empty() {
				return;
			}
			let max_lines = 8;
			let mut lines = vec![];
			let mut relevant = relevant.into_iter();
			while lines.len() < max_lines {
				if let Some(x) = relevant.next() {
					let url = x.subject.url;
					let Some(url) = url else {
						lines.push("no url".to_owned());
						continue;
					};
					let parts: Vec<_> = url.split('/').collect();
					if parts.len() < 8 {
						lines.push("too few url parts".to_owned());
						continue;
					}
					lines.push(format!("{} #{}", parts[5], parts[7]));
					if lines.len() < max_lines {
						lines.push(x.subject.title.clone());
					}
				} else {
					break;
				}
			}
			let remaining = relevant.count();
			if remaining != 0 {
				lines.push(format!("... {} more", remaining));
			}
			ctx.do_draw(Box::new(GithubNotificationsDraw {
				calls: RefCell::new(0),
				screen: &GITHUB,
				lines,
				circles: RefCell::new(vec![]),
			}));
		}
	}
}

struct GithubNotificationsDraw {
	calls: RefCell<usize>,
	screen: &'static SimpleScreensaver,
	lines: Vec<String>,
	circles: RefCell<Vec<((u32, u32), u32, Rgb565, Vec<(u32, u32)>)>>,
}

impl<D: DrawTarget<Color = Rgb565>> Draw<D> for GithubNotificationsDraw {
	fn draw(&self, disp: &mut D, rng: &mut crate::Rng) -> Result<bool, D::Error> {
		let calls = *self.calls.borrow();
		if calls == 0 {
			self.screen
				.draw_all_colored(disp, Rgb565::new(0xff >> 3, 0xff >> 2, 0xff >> 3))?;
		}
		if calls < 70 {
			let hue = (rng.next_u32() % 360) as f64;
			let hsv = Hsv::new(hue, 1.0, 1.0);
			let rgb = hsv.to_rgb();
			let r = rgb.r as u8 >> 3;
			let g = rgb.g as u8 >> 2;
			let b = rgb.b as u8 >> 3;
			let rgb = Rgb565::new(r, g, b);
			let mut x;
			let mut y;
			loop {
				x = rng.next_u32() % 128;
				y = rng.next_u32() % 128;
				if self.screen.get_pixel(x, y) == Rgb565::WHITE {
					break;
				}
			}
			disp.fill_contiguous(&Rectangle::new((x as i32, y as i32).into(), (1, 1).into()), Some(rgb))?;
			// advance other circles
			let mut circles = self.circles.borrow_mut();
			for ((ox, oy), radius, color, points) in &mut *circles {
				*radius += 1;
				if *radius >= 20 {
					continue;
				}
				let mut seen = HashSet::new();
				let mut next_points = vec![];
				loop {
					let mut new_points = vec![];
					for (x, y) in &*points {
						for (dx, dy) in [(-1, 0), (0, -1), (1, 0), (0, 1)] {
							let Some(x) = x.checked_add_signed(dx) else {
								continue;
							};
							let Some(y) = y.checked_add_signed(dy) else {
								continue;
							};
							if x >= 128 || y >= 128 {
								continue;
							}
							if self.screen.get_pixel(x, y) != Rgb565::WHITE {
								continue;
							}
							if seen.contains(&(x, y)) {
								continue;
							}
							seen.insert((x, y));
							let dist2 = x.abs_diff(*ox).pow(2) + y.abs_diff(*oy).pow(2);
							if dist2 > (*radius - 1).pow(2) && dist2 <= (*radius).pow(2) {
								new_points.push((x, y));
							}
						}
					}
					if new_points.is_empty() {
						break;
					}
					next_points.extend_from_slice(&new_points);
					*points = new_points;
				}
				disp.draw_iter(
					next_points
						.iter()
						.map(|&x| Pixel(Point::new(x.0 as _, x.1 as _), *color)),
				)?;
				*points = next_points;
			}
			circles.retain(|x| x.1 < 10);
			circles.push(((x, y), 0, rgb, vec![(x, y)]));
		} else {
			let idx = calls - 70;
			disp.clear(Rgb565::BLACK)?;
			// fit 9 lines
			let max_line_length = 16;
			let text_style_clock = MonoTextStyleBuilder::new()
				.font(&FONT_8X13)
				.text_color(Rgb565::WHITE)
				.build();
			for (y, line) in self.lines.iter().enumerate() {
				let mut line = if calls >= 119 {
					if y % 2 == 0 {
						line.clone()
					} else {
						format!(" {line}")
					}
				} else if line.len() > max_line_length {
					let line = format!("               {line} ");
					line[line.ceil_char_boundary(idx % line.len())..].to_string()
				} else {
					line.clone()
				};
				line.truncate(line.floor_char_boundary(max_line_length));
				Text::new(&line, Point::new(0, (12 + y * 14) as _), text_style_clock).draw(disp)?;
			}
		}
		*self.calls.borrow_mut() += 1;
		Ok(calls < 120)
	}

	fn expired(&self) -> bool {
		*self.calls.borrow() > 140
	}

	fn as_any(&self) -> &dyn std::any::Any {
		&*self
	}

	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		&mut *self
	}
}
