use time::OffsetDateTime;

use crate::{action::Action, Context};

/// Task to be executed at certain times.
/// Guaranteed to be checked at least once every minute.
pub trait Schedule {
	fn check_and_do(&self, ctx: &dyn Context, time: OffsetDateTime) {
		if self.check(ctx, time) {
			self.execute(ctx, time);
		}
	}

	fn check(&self, ctx: &dyn Context, time: OffsetDateTime) -> bool;
	fn execute(&self, ctx: &dyn Context, time: OffsetDateTime);
}

#[derive(Debug, Clone, Copy)]
pub struct Reminder {
	hour: u8,
	minute: u8,
	action: Action,
	should_beep: bool,
}

impl Schedule for Reminder {
	fn check(&self, ctx: &dyn Context, time: OffsetDateTime) -> bool {
		time.hour() == self.hour && time.minute() == self.minute && ctx.active_count() == 1
	}

	fn execute(&self, ctx: &dyn Context, _time: OffsetDateTime) {
		if self.should_beep {
			ctx.enable_pwm();
		}
		ctx.do_action(self.action);
	}
}

impl Reminder {
	const fn new(hour: u8, minute: u8, action: Action, should_beep: bool) -> Self {
		Reminder {
			hour,
			minute,
			action,
			should_beep,
		}
	}
}

static DUOLINGO: Reminder = Reminder::new(11, 40, Action::Screensaver("duolingo"), false);
static DUOLINGO_NIGHT: Reminder = Reminder::new(23, 40, Action::Screensaver("duolingo"), false);
static FOOD: Reminder = Reminder::new(13, 15, Action::Screensaver("plate"), false);

pub fn reminders() -> Vec<Box<dyn Schedule>> {
	vec![Box::new(DUOLINGO), Box::new(DUOLINGO_NIGHT), Box::new(FOOD)]
}
