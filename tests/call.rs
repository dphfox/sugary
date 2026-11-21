use sugary::{call, sugary};
use sugary_test_kit::Theme;

pub struct Log(std::cell::Cell<u32>);

#[derive(Clone, Copy)]
pub struct Scale(pub u32);

mod widgets {
	use super::{Log, Theme};
	use sugary::{call, sugary};

	const WIDE: u32 = 9;

	#[sugary]
	pub fn label(#[inherit] theme: &Theme, text: &str, #[default(1)] size: u32) -> String {
		format!("{text}:{}", theme.accent * size)
	}

	#[sugary]
	pub fn button(
		#[inherit] log: &Log,
		#[inherit] theme: &Theme,
		text: &str,
		#[default(false)] disabled: bool,
		#[default(WIDE)] width: u32,
	) -> String {
		log.0.set(log.0.get() + 1);
		let caption = call!(label, text: text);
		format!("[{caption} {disabled} {width}]")
	}
}

use widgets::button;
use widgets::label as heading_text;

#[sugary]
fn form(#[inherit] log: &Log, #[inherit] theme: &Theme, title: &str, rows: &[&str]) -> Vec<String> {
	let heading = call!(heading_text, size: 2, text: title);
	let rows = rows.iter().map(|row| call!(button, text: row));
	let cancel = call!(widgets::button, disabled: true, text: "Cancel", width: 3);
	[heading].into_iter().chain(rows).chain([cancel]).collect()
}

fn form_plain(log: &Log, theme: &Theme, title: &str, rows: &[&str]) -> Vec<String> {
	let heading = widgets::label(theme, title, 2);
	let rows = rows
		.iter()
		.map(|row| widgets::button(log, theme, row, false, 9));
	let cancel = widgets::button(log, theme, "Cancel", true, 3);
	[heading].into_iter().chain(rows).chain([cancel]).collect()
}

// Named arguments in any order, defaults, inheritance by type through two
// levels, and three ways of naming the function: own name, renamed, by path.
#[test]
fn a_sugared_call_is_the_plain_call() {
	let (log, theme) = (Log(Default::default()), Theme { accent: 7 });

	let sugared = form(&log, &theme, "Title", &["first", "second"]);
	let plain = form_plain(&log, &theme, "Title", &["first", "second"]);

	assert_eq!(sugared, plain);
	assert_eq!(sugared[0], "Title:14");
	assert_eq!(sugared[3], "[Cancel:7 true 3]");
	assert_eq!(log.0.get(), 6);
}

#[sugary]
fn pick<T: std::fmt::Display>(
	#[inherit] theme: &Theme,
	#[inherit] scale: Scale,
	value: T,
	choose: impl Fn(u32) -> u32,
	#[default(|| 0)] extra: impl FnOnce() -> u32,
) -> String {
	format!("{value}:{}", choose(theme.accent * scale.0) + extra())
}

#[sugary]
fn tree(#[inherit] theme: &Theme, depth: u32) -> u32 {
	if depth == 0 {
		theme.accent
	} else {
		call!(tree, depth: depth - 1) + call!(self::tree, depth: depth - 1)
	}
}

#[sugary]
fn lazy<'a>(#[inherit] theme: &'a Theme) -> impl Iterator<Item = String> + 'a {
	(1..=3u32).map(move |size| call!(heading_text, text: "row", size: size))
}

#[sugary]
fn screen(
	#[inherit] theme: &Theme,
	#[inherit] scale: Scale,
	input: Option<&str>,
) -> Option<Vec<String>> {
	let rows = lazy(theme);
	let mut out = vec![
		// `?` works inside an argument, because the expansion is a plain call.
		call!(heading_text, text: input?),
		// A generic function, a closure whose parameter type is inferred,
		// and a by-value inherited argument.
		call!(pick, value: 3.5, choose: |v| v + 1),
		call!(pick, value: "s", choose: |v| v, extra: || 100),
		call!(tree, depth: 3).to_string(),
	];
	out.extend(rows);
	Some(out)
}

#[test]
fn ordinary_rust_keeps_working_inside_and_around_a_call() {
	let theme = Theme { accent: 7 };

	assert_eq!(
		screen(&theme, Scale(2), Some("in")).unwrap(),
		["in:7", "3.5:15", "s:114", "56", "row:7", "row:14", "row:21"]
	);
	assert_eq!(screen(&theme, Scale(2), None), None);
}

// The values a lazy iterator captures are the ones in scope where it was
// written, whatever happens before it is consumed.
#[test]
fn a_lazy_iterator_consumed_later_keeps_its_inherited_arguments() {
	let (theme, other) = (Theme { accent: 7 }, Theme { accent: 100 });

	let pending = lazy(&theme);
	let _unrelated = widgets::label(&other, "x", 1);

	assert_eq!(pending.collect::<Vec<_>>(), ["row:7", "row:14", "row:21"]);
}

use sugary_test_kit::badges::chip as badge;

#[sugary]
fn toolbar(#[inherit] theme: &Theme) -> Vec<String> {
	vec![
		call!(sugary_test_kit::buttons::chip, text: "Save"),
		call!(sugary_test_kit::buttons::chip, weight: 3, text: "Open"),
		call!(badge, count: 4),
	]
}

// Two functions that share a name, both from another crate.
#[test]
fn functions_from_another_crate_can_be_called_by_path_or_renamed() {
	let theme = Theme { accent: 7 };

	assert_eq!(toolbar(&theme), ["Save:14", "Open:21", "badge:11"]);
	assert_eq!(
		sugary_test_kit::badges::pair(&theme),
		["badge:8", "badge:9"]
	);
}

mod short {
	use super::Theme;
	use sugary::sugary;

	#[sugary]
	pub fn chip(#[inherit] theme: &Theme, text: &str, #[default(1)] weight: u32) -> String {
		format!("{text}:{}", theme.accent * weight)
	}
}

use short::chip;

#[sugary]
fn bar(#[inherit] theme: &Theme) -> Vec<String> {
	vec![
		chip!(text: "a"),
		chip!(weight: 3, text: "b"),
		call!(chip, text: "c"),
	]
}

#[test]
fn the_short_form_is_the_same_call() {
	assert_eq!(bar(&Theme { accent: 7 }), ["a:7", "b:21", "c:7"]);
}

#[sugary]
fn nothing_inherited(count: u32, #[default] label: String) -> String {
	format!("{count}{label}")
}

#[sugary]
fn no_inheritance_needed() -> String {
	call!(nothing_inherited, count: 3)
}

#[test]
fn a_bare_default_uses_the_type_s_own() {
	assert_eq!(no_inheritance_needed(), "3");
}

// None of the callers below is sugary, so each call inherits nothing.
fn plain_caller(count: u32) -> Vec<String> {
	let later = move || call!(nothing_inherited, label: "!".to_string(), count: count);
	vec![
		call!(nothing_inherited, count: count),
		nothing_inherited!(count: count + 1),
		nothing_inherited!(label: "?".to_string(), count: count),
		later(),
	]
}

static AT_MODULE_LEVEL: fn() -> String = || call!(self::nothing_inherited, count: 9);

#[test]
fn a_call_from_a_plain_function_inherits_nothing() {
	assert_eq!(plain_caller(3), ["3", "4", "3?", "3!"]);
	assert_eq!(AT_MODULE_LEVEL(), "9");
	assert_eq!(sugary::call!(nothing_inherited, count: 1), "1");
}

#[test]
fn a_plain_function_can_call_into_another_crate() {
	use sugary_test_kit::layout::spacer;

	assert_eq!(call!(sugary_test_kit::layout::spacer), "1:0");
	assert_eq!(call!(spacer, fill: 'a', width: 2), "2:97");
	assert_eq!(spacer!(width: 3), "3:0");
}

#[sugary]
fn nested(#[inherit] theme: &Theme, #[inherit] scale: Scale) -> Vec<String> {
	let other = Theme { accent: 1000 };
	let closure = move |text: &str| {
		let inner = |size| call!(heading_text, text: text, size: size);
		let _ = &other;
		inner(scale.0)
	};
	let pending = ["a", "b"].into_iter().map(closure);
	let mut out = vec![sugary::call!(heading_text, text: "path")];
	out.extend(pending);
	out.push(call!(nothing_inherited, count: 5));
	out
}

// A local variable of the inherited type is not what gets passed on, and a
// call that needs nothing still works next to ones that inherit.
#[test]
fn calls_inside_a_sugary_function_still_inherit() {
	let theme = Theme { accent: 7 };

	assert_eq!(nested(&theme, Scale(2)), ["path:7", "a:14", "b:14", "5"]);
}
