use rand_xoshiro::rand_core::SeedableRng;
pub use rand_xoshiro::Xoroshiro128StarStar as Rng;

mod context;
pub use context::Draw;

mod screensaver;
pub use screensaver::Screensaver;

mod time_display;
pub use time_display::TimeDisplay;

pub fn new_rng() -> Rng {
	let seed = getrandom::u64().unwrap();
	Rng::seed_from_u64(seed)
}
