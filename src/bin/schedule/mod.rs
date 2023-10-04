use time::OffsetDateTime;

use crate::action::Action;

/// Task to be executed at certain times.
/// Guaranteed to be checked at least once every minute.
trait Schedule {
	fn check_and_do(&mut self, time: OffsetDateTime) {
		if self.check(time) {
			self.execute(time);
		}
	}

	fn check(&self, time: OffsetDateTime) -> bool;
	fn execute(&mut self, time: OffsetDateTime);
}

struct Reminder {
	hour: i32,
	minute: i32,
	action: Action,
}

impl Reminder {
	const fn new(hour: i32, minute: i32, action: Action) -> Self {
		Reminder { hour, minute, action }
	}
}

static DUOLINGO: Reminder = Reminder::new(11, 30, Action::Screensaver("duolingo"));
static DUOLINGO_NIGHT: Reminder = Reminder::new(23, 30, Action::Screensaver("duolingo"));
