use time::OffsetDateTime;

use crate::{action::Action, Context};

/// Task to be executed at certain times.
/// Guaranteed to be checked at least once every minute.
pub trait Schedule {
	fn check_and_do(&self, ctx: &dyn Context, time: OffsetDateTime) {
		if self.check(time) {
			self.execute(ctx, time);
		}
	}

	fn check(&self, time: OffsetDateTime) -> bool;
	fn execute(&self, ctx: &dyn Context, time: OffsetDateTime);
}

#[derive(Debug, Clone, Copy)]
pub struct Reminder {
	hour: u8,
	minute: u8,
	action: Action,
}

impl Schedule for Reminder {
	fn check(&self, time: OffsetDateTime) -> bool {
		time.hour() == self.hour && time.minute() == self.minute
	}

	fn execute(&self, ctx: &dyn Context, _time: OffsetDateTime) {
		ctx.do_action(self.action);
	}
}

impl Reminder {
	const fn new(hour: u8, minute: u8, action: Action) -> Self {
		Reminder { hour, minute, action }
	}
}

static DUOLINGO: Reminder = Reminder::new(11, 30, Action::Screensaver("duolingo"));
static DUOLINGO_NIGHT: Reminder = Reminder::new(23, 30, Action::Screensaver("duolingo"));
static FOOD: Reminder = Reminder::new(13, 15, Action::Screensaver("plate"));

pub fn reminders() -> Vec<Box<dyn Schedule>> {
	vec![Box::new(DUOLINGO), Box::new(DUOLINGO_NIGHT), Box::new(FOOD)]
}
