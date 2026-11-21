use proto::{Comp, Key, States, Theme, build, buildable};

#[buildable]
pub fn label(key: Key, #[service] theme: &Theme, text: &str, #[default(1)] size: u32) -> Comp {
	Comp::one(key, theme.accent * size, text)
}

#[buildable]
pub fn button(
	key: Key,
	#[service] state: &States,
	#[service] theme: &Theme,
	text: &str,
	#[default(false)] disabled: bool,
) -> Comp {
	let pressed = state.get(key / "pressed");
	let caption = build!(label, key: key / "caption", text: text);
	Comp::of([
		Comp::one(key, if disabled { 0 } else { theme.accent }, format!("{}", pressed % 10)),
		caption,
	])
}
