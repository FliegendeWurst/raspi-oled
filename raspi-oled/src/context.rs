use std::{cell::RefCell, rc::Rc};

use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};
use rand_xoshiro::Xoroshiro128StarStar;
use raspi_lib::{Draw, Screensaver, TimeDisplay};
use rusqlite::Connection;
use time::OffsetDateTime;
use time_tz::{timezones::db::europe::BERLIN, OffsetDateTimeExt};

use crate::{
	action::Action,
	disable_pwm,
	draw::{self, Measurements, Totp},
	enable_pwm,
	schedule::{self, github_notifications::GithubNotifications, Schedule},
	screensaver::{self, BearReminder},
};

pub static BLACK: Rgb565 = Rgb565::new(0, 0, 0);

pub type Rng = Xoroshiro128StarStar;

pub trait Context<D: DrawTarget<Color = Rgb565>> {
	fn do_draw(&self, drawable: Box<dyn Draw<D>>);

	fn do_action(&self, action: Action);

	fn active_count(&self) -> usize;

	fn database(&self) -> Rc<RefCell<Connection>>;

	fn enable_pwm(&self);
}

pub trait DrawWithContext<D: DrawTarget<Color = Rgb565>> : Draw<D> {
	fn draw_with_ctx(&self, _ctx: &ContextDefault<D>, disp: &mut D, rng: &mut Rng) -> Result<bool, D::Error> {
		self.draw(disp, rng)
	}
}

pub struct ContextDefault<D: DrawTarget<Color = Rgb565>> {
	screensavers: Vec<Box<dyn Screensaver<D>>>,
	scheduled: Vec<Box<dyn Schedule<D>>>,
	pub active: RefCell<Vec<Box<dyn Draw<D>>>>,
	database: Rc<RefCell<Connection>>,
}

impl<D: DrawTarget<Color = Rgb565>> ContextDefault<D> {
	pub fn new() -> Self {
		let mut screensavers = screensaver::screensavers();
		screensavers.push(Box::new(draw::Measurements::default()));
		screensavers.push(Box::new(draw::Measurements::temps()));
		screensavers.push(Box::new(draw::Measurements::events()));
		let database = Connection::open("sensors.db").expect("failed to open database");
		let mut scheduled = schedule::reminders();
		scheduled.push(Box::new(GithubNotifications {
			pat: std::env::var("GITHUB_PAT").expect("no env var GITHUB_PAT set"),
			last_modified: RefCell::new(None),
			last_call: RefCell::new(OffsetDateTime::now_utc().to_timezone(BERLIN) - time::Duration::seconds(50)),
		}));
		scheduled.push(Box::new(BearReminder::default()));
		ContextDefault {
			database: Rc::new(RefCell::new(database)),
			screensavers,
			scheduled,
			active: RefCell::new(vec![Box::new(TimeDisplay::new())]),
		}
	}

	pub fn add(&mut self, totp: Totp) {
		self.screensavers.push(Box::new(totp));
	}

	pub fn loop_iter(&mut self, disp: &mut D, rng: &mut Rng) -> bool {
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
		if !a.expired() {
			let measure: Option<&Measurements> = a.as_any().downcast_ref();
			if let Some(measure) = measure {
				return measure.draw_with_ctx(self, disp, rng).unwrap_or(true);
			} else {
				return a.draw(disp, rng).unwrap_or(true);
			}
		}
		drop(active);
		self.active.borrow_mut().pop();
		disable_pwm().unwrap();
		self.loop_iter(disp, rng)
	}

	pub fn pop_action_and_clear(&mut self, disp: &mut D) -> Result<(), D::Error> {
		let active = self.active.get_mut();
		if active.len() > 1 {
			active.pop();
			disp.clear(BLACK)?;
		}
		Ok(())
	}
}

impl<D: DrawTarget<Color = Rgb565>> Context<D> for ContextDefault<D> {
	fn do_draw(&self, drawable: Box<dyn Draw<D>>) {
		self.active.borrow_mut().push(drawable);
	}

	fn do_action(&self, action: Action) {
		match action {
			Action::Screensaver(id) => {
				for s in &self.screensavers {
					if s.id() == id {
						self.active.borrow_mut().push(s.convert_draw());
						return;
					}
				}
				println!("warning: screensaver not found");
			},
		}
	}

	fn active_count(&self) -> usize {
		self.active.borrow().len()
	}

	fn database(&self) -> Rc<RefCell<Connection>> {
		self.database.clone()
	}

	fn enable_pwm(&self) {
		enable_pwm().unwrap();
	}
}
