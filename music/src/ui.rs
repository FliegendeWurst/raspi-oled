use std::cell::RefCell;

use raspi_lib::{BLACK, Draw, DrawTarget, Drawable, FONT, Point, Rgb565, Text};

pub enum UiResult {
	Ignore,
	Close,
	Replace(&'static str),
}
use UiResult::*;

use crate::command::{execute, set_volume};

pub struct Ui {
	id: &'static str,
	drawn: RefCell<u32>,
	aux1: i32,
}

impl Ui {
	pub fn new(id: &'static str) -> Self {
		Ui {
			id,
			drawn: RefCell::new(0),
			aux1: 0,
		}
	}

	pub fn new_aux1(id: &'static str, aux1: i32) -> Self {
		Ui {
			id,
			drawn: RefCell::new(0),
			aux1,
		}
	}

	pub fn handle(&self, button: usize) -> UiResult {
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
		if *self.drawn.borrow() > 1 {
			return Ok(false);
		}
		disp.clear(BLACK)?;
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
