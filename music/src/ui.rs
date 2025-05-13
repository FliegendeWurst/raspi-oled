use std::cell::RefCell;

use raspi_lib::{BLACK, Draw, DrawTarget, Drawable, FONT, FONT_RED, Point, Rgb565, Text};

pub enum UiResult {
	Ignore,
	Close,
	Replace(&'static str),
	Play(String),
}
use UiResult::*;

use crate::command::{execute, set_volume};

pub struct Ui {
	id: &'static str,
	drawn: RefCell<u32>,
	aux1: i32,
	aux2: Vec<String>,
	aux3: usize,
}

impl Ui {
	pub fn new(id: &'static str) -> Self {
		Ui {
			id,
			drawn: RefCell::new(0),
			aux1: 0,
			aux2: vec![],
			aux3: 0,
		}
	}

	pub fn new_aux1(id: &'static str, aux1: i32) -> Self {
		Ui {
			id,
			drawn: RefCell::new(0),
			aux1,
			aux2: vec![],
			aux3: 0,
		}
	}

	pub fn new_aux2(id: &'static str, aux2: Vec<String>) -> Self {
		Ui {
			id,
			drawn: RefCell::new(0),
			aux1: 0,
			aux2,
			aux3: 0,
		}
	}

	pub fn handle(&mut self, button: usize) -> UiResult {
		match (self.id, button) {
			("exit", 1) => {
				println!("[CMD] shutdown");
				execute(&["bash", "-c", "(sleep 2 && sudo shutdown now) &"]);
				Replace("exit_confirmed")
			},
			("exit", _) => Close,
			("exit_confirmed", _) => Ignore,
			("volume", 3) => {
				set_volume("+5%");
				Replace("volume")
			},
			("volume", 5) => {
				set_volume("-5%");
				Replace("volume")
			},
			("select", 0) => Play(self.aux2[self.aux3].clone()),
			("select", 2) => {
				if self.aux3 == 0 {
					self.aux3 = self.aux2.len() - 1;
				} else {
					self.aux3 -= 1;
				}
				*self.drawn.get_mut() = 0;
				Ignore
			},
			("select", 4) => {
				self.aux3 += 1;
				if self.aux3 == self.aux2.len() {
					self.aux3 = 0;
				}
				*self.drawn.get_mut() = 0;
				Ignore
			},
			_ => Close,
		}
	}

	pub fn should_close(&self) -> bool {
		matches!(self.id, "volume") && *self.drawn.borrow() > 0
	}
}

impl<D: DrawTarget<Color = Rgb565>> Draw<D> for Ui {
	fn draw(&self, disp: &mut D, _rng: &mut raspi_lib::Rng) -> Result<bool, <D as DrawTarget>::Error> {
		*self.drawn.borrow_mut() += 1;
		if *self.drawn.borrow() > 1 && self.id != "select" {
			return Ok(false);
		}
		let iters = *self.drawn.borrow() - 1;
		disp.clear(BLACK)?;

		macro_rules! draw_scrolling {
			($string:expr, $y:expr, $styl:expr) => {
				let max_len = 128 / 10;
				if $string.len() <= max_len {
					let size = $string.len() * 10;
					let pad = (max_len * 10 - size) / 2;
					Text::new($string, Point::new(4 + pad as i32, $y), $styl).draw(disp)?;
				} else {
					let pad = " ".repeat(max_len);
					let full = format!("{pad}{}{pad}  ", $string);
					let idx_start = iters as usize % (full.len() - max_len);
					Text::new(
						&full[full.ceil_char_boundary(idx_start)..full.floor_char_boundary(idx_start + max_len)],
						Point::new(4, $y),
						$styl,
					)
					.draw(disp)?;
				}
			};
		}

		match self.id {
			"exit" => {
				Text::new("Confirm\n   shutdown?", Point::new(4, 64 - 20), FONT).draw(disp)?;
			},
			"exit_confirmed" => {
				Text::new("Unplug in\n  30 seconds", Point::new(4, 64 - 20), FONT).draw(disp)?;
			},
			"volume" => {
				Text::new(
					&format!("Volume\n{: >12}", format!("{}%", self.aux1)),
					Point::new(4, 64 - 20),
					FONT,
				)
				.draw(disp)?;
			},
			"select" => {
				let mut pages = self.aux2.chunks(6);
				let active_page = self.aux3 / 6;
				let active_idx = self.aux3 % 6;
				let page = pages.nth(active_page).unwrap();
				for i in 0..6 {
					let styl = if i == active_idx { FONT_RED } else { FONT };
					if i < page.len() {
						let name = &page[i];
						if name.len() <= 12 {
							Text::new(name, Point::new(4, 14 + i as i32 * 20), styl).draw(disp)?;
						} else {
							draw_scrolling!(name, 14 + i as i32 * 20, styl);
						}
					}
				}
			},
			_ => {},
		}
		Ok(true)
	}

	fn as_any(&self) -> &dyn std::any::Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		self
	}
}
