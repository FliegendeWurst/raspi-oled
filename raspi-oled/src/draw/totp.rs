use std::{any::Any, cell::RefCell};

use andotp_import::Account;
use embedded_graphics::{
	mono_font::{
		ascii::{FONT_10X20, FONT_9X15},
		MonoTextStyleBuilder,
	},
	pixelcolor::Rgb565,
	prelude::*,
	text::Text,
	Drawable,
};
use raspi_lib::{Draw, Screensaver};
use totp_rs::TOTP;

use crate::context::{Rng, BLACK};

#[derive(Debug, Clone)]
pub struct Totp {
	codes: RefCell<Vec<String>>,
	secrets: Vec<(Account, TOTP)>,
	page: usize,
}

impl Totp {
	pub fn new(secrets: Vec<(Account, TOTP)>) -> Self {
		Self {
			codes: RefCell::new(vec![]),
			secrets,
			page: 0,
		}
	}

	pub fn next_page(&mut self) {
		self.page += 1;
		if self.secrets.len() < self.page * 6 {
			self.page = 0;
		}
	}
}

impl<D: DrawTarget<Color = Rgb565>> Screensaver<D> for Totp {
	fn id(&self) -> &'static str {
		"totp"
	}

	fn convert_draw(&self) -> Box<dyn Draw<D>> {
		Box::new(Totp {
			codes: RefCell::new(vec![]),
			secrets: self.secrets.clone(),
			page: 0,
		})
	}
}

impl<D: DrawTarget<Color = Rgb565>> Draw<D> for Totp {
	fn draw(&self, disp: &mut D, _rng: &mut Rng) -> Result<bool, <D as DrawTarget>::Error> {
		let codes: Vec<_> = self
			.secrets
			.iter()
			.skip(self.page * 6)
			.take(6)
			.map(|x| (&x.0.issuer, &x.0.label, x.1.generate_current().unwrap()))
			.collect();
		if codes.len() == self.codes.borrow().len()
			&& codes.iter().zip(self.codes.borrow().iter()).all(|(x, y)| &x.2 == y)
		{
			return Ok(false);
		}
		*self.codes.borrow_mut() = codes.iter().map(|x| x.2.clone()).collect();
		disp.clear(BLACK)?;
		let mut y = 16;

		let text_style_code = MonoTextStyleBuilder::new()
			.font(&FONT_10X20)
			.text_color(Rgb565::new(0xff, 0xff, 0xff))
			.build();
		let text_style_label = MonoTextStyleBuilder::new()
			.font(&FONT_9X15)
			.text_color(Rgb565::new(0xff, 0xff, 0xff))
			.build();

		for (issuer, label, code) in codes {
			Text::new(&code, Point::new(0, y), text_style_code).draw(disp)?;
			let label_text = if issuer != "" {
				issuer
			} else if let Some((issuer, _label)) = label.split_once(" - ") {
				issuer
			} else {
				label
			};
			Text::new(
				if label_text.len() > 7 {
					&label_text[0..7]
				} else {
					label_text
				},
				Point::new(60, y),
				text_style_label,
			)
			.draw(disp)?;
			y += 20 + 1;
		}
		Ok(true)
	}

	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}
