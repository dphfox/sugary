use proto::{Key, States, Theme, build, buildable};

#[buildable]
#[inline(never)]
pub fn leaf(key: Key, #[service] theme: &Theme, n: u64, #[default(3)] scale: u64) -> u64 {
	key.0 ^ (theme.accent as u64).wrapping_mul(n).wrapping_mul(scale)
}

#[buildable]
#[inline(never)]
pub fn row(key: Key, #[service] state: &States, #[service] theme: &Theme, index: u64, #[default(false)] dim: bool) -> u64 {
	let remembered = state.get(key / "pressed");
	let a = build!(leaf, key: key / "a", n: index);
	let b = build!(leaf, n: index + 1, scale: 5, key: key / "b");
	remembered ^ a ^ b ^ dim as u64
}

#[buildable]
#[inline(never)]
pub fn screen_sugared(key: Key, #[service] state: &States, #[service] theme: &Theme, rows: u64) -> u64 {
	let mut total = 0u64;
	for index in 0..rows {
		total ^= build!(row, key: key / ("row", index), index: index);
		total ^= build!(self::row, dim: true, index: index * 2, key: key / ("alt", index));
	}
	total ^ build!(leaf, key: key / "footer", n: rows)
}

#[inline(never)]
pub fn screen_plain(key: Key, state: &States, theme: &Theme, rows: u64) -> u64 {
	let mut total = 0u64;
	for index in 0..rows {
		total ^= row(key / ("row", index), state, theme, index, false);
		total ^= row(key / ("alt", index), state, theme, index * 2, true);
	}
	total ^ leaf(key / "footer", theme, rows, 3)
}

fn main() {
	let (states, theme) = (States::default(), Theme { accent: 7 });
	let rows = std::hint::black_box(500u64);
	println!("{}", screen_sugared(Key::ROOT, &states, &theme, rows) == screen_plain(Key::ROOT, &states, &theme, rows));
}
