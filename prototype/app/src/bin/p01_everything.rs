use proto::{Comp, Key, States, Theme, build, buildable};
#[buildable]
pub fn label(key: Key, #[service] theme: &Theme, text: &str, #[default(1)] size: u32) -> Comp { Comp::one(key, theme.accent * size, text) }
#[buildable]
pub fn counter(key: Key, #[service] state: &States, #[service] theme: &Theme, text: &str) -> Comp { Comp::one(key, theme.accent + state.get(key) as u32, text) }
#[allow(dead_code)] fn silence(_: &States) {}

mod w { use proto::{Comp, Key, Theme, buildable}; #[buildable] pub fn chip(key: Key, #[service] theme: &Theme, text: &str) -> Comp { Comp::one(key, theme.accent, text) } }
use kit::badges::chip as badge;
use w::chip as tag;
#[derive(Clone, Copy)] pub struct Scale(pub u32);
#[buildable] fn pick<T: std::fmt::Display>(key: Key, #[service] theme: &Theme, #[service] scale: Scale, value: T, choose: impl Fn(u32) -> u32, #[default(|| 0)] extra: impl FnOnce() -> u32) -> Comp { Comp::one(key, choose(theme.accent * scale.0) + extra(), value.to_string()) }
#[buildable] fn tree(key: Key, #[service] theme: &Theme, depth: u32) -> Comp { if depth == 0 { build!(label, key: key, text: "leaf") } else { Comp::of([build!(tree, key: key / "l", depth: depth - 1), build!(self::tree, key: key / "r", depth: depth - 1)]) } }
#[buildable] fn lazy<'a>(key: Key, #[service] theme: &'a Theme) -> impl Iterator<Item = Comp> + 'a { (0..3u32).map(move |i| build!(label, key: key / i, text: "row", size: i + 1)) }
#[buildable] fn screen(key: Key, #[service] theme: &Theme, #[service] scale: Scale, input: Option<&str>) -> Option<Comp> {
	let rows = lazy(key / "rows", theme);
	Some(Comp::of([
		build!(tag, key: key / 1, text: input?),
		build!(w::chip, key: key / 2, text: "by path"),
		build!(crate::w::chip, key: key / 3, text: "from the crate root"),
		build!(kit::buttons::chip, key: key / 4, text: "other crate", weight: 3),
		build!(badge, key: key / 5, count: 4),
		build!(pick, key: key / 6, value: 3.5, choose: |v| v + 1),
		build!(tree, key: key / 7, depth: 3),
	].into_iter().chain(rows)))
}
fn main() { let theme = Theme { accent: 7 }; let out = screen(Key::ROOT, &theme, Scale(2), Some("renamed")).unwrap(); println!("{} items; first few: {:?}", out.items.len(), out.items.iter().take(6).map(|i| (i.1, i.2.as_str())).collect::<Vec<_>>()); }
