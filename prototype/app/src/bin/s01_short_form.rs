use proto::{Comp, Key, Theme, build, buildable};

mod w {
	use proto::{Comp, Key, Theme, buildable};
	#[buildable]
	pub fn chip(key: Key, #[service] theme: &Theme, text: &str, #[default(1)] weight: u32) -> Comp {
		Comp::one(key, theme.accent * weight, text)
	}
}
use w::chip;

#[buildable]
fn bar(key: Key, #[service] theme: &Theme) -> Comp {
	Comp::of([
		chip!(key: key / 1, text: "short form"),
		chip!(weight: 3, text: "short form, reordered", key: key / 2),
		build!(chip, key: key / 3, text: "shared form"),
		build!(w::chip, key: key / 4, text: "shared form by path"),
	])
}

fn main() {
	let out = bar(Key::ROOT, &Theme { accent: 7 });
	println!("{:?}", out.items.iter().map(|i| (i.1, i.2.as_str())).collect::<Vec<_>>());
}
