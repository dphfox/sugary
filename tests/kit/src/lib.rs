pub struct Theme {
	pub accent: u32,
}

pub mod buttons {
	use super::Theme;
	use sugary::sugary;

	#[sugary]
	pub fn chip(
		#[inherit] theme: &Theme,
		#[require] text: &str,
		#[default = 2] weight: u32,
	) -> String {
		format!("{text}:{}", theme.accent * weight)
	}
}

pub mod badges {
	use super::Theme;
	use sugary::{call, sugary};

	// Shares its name with `buttons::chip`, so the two hidden exported
	// macros must not collide.
	#[sugary]
	pub fn chip(#[inherit] theme: &Theme, #[require] count: u32) -> String {
		format!("badge:{}", theme.accent + count)
	}

	#[sugary]
	pub fn pair(#[inherit] theme: &Theme) -> Vec<String> {
		vec![call!(chip, count: 1), call!(chip, count: 2)]
	}
}
