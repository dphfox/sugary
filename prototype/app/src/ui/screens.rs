use proto::{Comp, Key, States, Theme, build, buildable};

// Three ways of reaching a build function: by its own name, renamed, by path.
use super::widgets;
use super::widgets::button;
use super::widgets::label as heading_text;

#[buildable]
pub fn form(
	key: Key,
	#[service] state: &States,
	#[service] theme: &Theme,
	title: &str,
	rows: &[(u32, String)],
) -> Comp {
	let heading = build!(heading_text, size: 2, text: title, key: key / "title");
	let rows = rows.iter().map(|(id, text)| build!(button, key: key / ("row", id), text: text));
	let cancel = build!(widgets::button, key: key / "cancel", text: "Cancel", disabled: true);
	Comp::of([heading].into_iter().chain(rows).chain([cancel]))
}

pub fn form_plain(
	key: Key,
	state: &States,
	theme: &Theme,
	title: &str,
	rows: &[(u32, String)],
) -> Comp {
	let heading = widgets::label(key / "title", theme, title, 2);
	let rows = rows.iter().map(|(id, text)| button(key / ("row", id), state, theme, text, false));
	let cancel = button(key / "cancel", state, theme, "Cancel", true);
	Comp::of([heading].into_iter().chain(rows).chain([cancel]))
}
