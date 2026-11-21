// PROTOTYPE: a separate crate offering build functions to others.
pub mod buttons {
	use proto::{Comp, Key, Theme, buildable};

	#[buildable]
	pub fn chip(key: Key, #[service] theme: &Theme, text: &str, #[default(2)] weight: u32) -> Comp {
		Comp::one(key, theme.accent * weight, text)
	}
}

pub mod badges {
	use proto::{Comp, Key, Theme, build, buildable};

	// Same name as in `buttons`, to test that the exported macros do not clash.
	#[buildable]
	pub fn chip(key: Key, #[service] theme: &Theme, count: u32) -> Comp {
		Comp::one(key, theme.accent + count, "badge")
	}

	#[buildable]
	pub fn pair(key: Key, #[service] theme: &Theme) -> Comp {
		Comp::of([build!(chip, key: key / "a", count: 1), build!(chip, key: key / "b", count: 2)])
	}
}
