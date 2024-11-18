use std::cell::RefCell;

use color_space::{Hsv, ToRgb};
use embedded_graphics::{
	mono_font::{iso_8859_10::FONT_8X13, MonoTextStyleBuilder},
	pixelcolor::Rgb565,
	prelude::{DrawTarget, Point, RgbColor},
	text::Text,
	Drawable,
};
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
			let max_line_length = 16;
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
						let mut desc = format!(" {}", x.subject.title);
						desc.truncate(desc.floor_char_boundary(max_line_length));
						lines.push(desc);
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
			}));
		}
	}
}

struct GithubNotificationsDraw {
	calls: RefCell<usize>,
	screen: &'static SimpleScreensaver,
	lines: Vec<String>,
}

impl<D: DrawTarget<Color = Rgb565>> Draw<D> for GithubNotificationsDraw {
	fn draw(&self, disp: &mut D, _rng: &mut crate::Rng) -> Result<bool, D::Error> {
		let calls = *self.calls.borrow();
		if calls < 40 {
			let hue = calls as f64 / 40.0 * 360.0;
			let hsv = Hsv::new(hue, 1.0, 1.0);
			let rgb = hsv.to_rgb();
			let r = rgb.r as u8 >> 3;
			let g = rgb.g as u8 >> 2;
			let b = rgb.b as u8 >> 3;
			self.screen.draw_all_colored(disp, Rgb565::new(r, g, b))?;
		} else {
			disp.clear(Rgb565::BLACK)?;
			// fit 9 lines
			let text_style_clock = MonoTextStyleBuilder::new()
				.font(&FONT_8X13)
				.text_color(Rgb565::WHITE)
				.build();
			for (y, line) in self.lines.iter().enumerate() {
				Text::new(line, Point::new(0, (12 + y * 14) as _), text_style_clock).draw(disp)?;
			}
		}
		*self.calls.borrow_mut() += 1;
		Ok(calls < 90)
	}

	fn expired(&self) -> bool {
		*self.calls.borrow() > 90
	}

	fn as_any(&self) -> &dyn std::any::Any {
		&*self
	}

	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		&mut *self
	}
}
