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
	pub fn label(
		#[inherits] theme: &Theme,
		#[required] text: &str,
		#[optional = 1] size: u32,
	) -> String {
		format!("{text}:{}", theme.accent * size)
	}

	#[sugary]
	pub fn button(
		#[inherits] log: &Log,
		#[inherits] theme: &Theme,
		#[required] text: &str,
		#[optional = false] disabled: bool,
		#[optional = WIDE] width: u32,
	) -> String {
		log.0.set(log.0.get() + 1);
		let caption = call!(label, text: text);
		format!("[{caption} {disabled} {width}]")
	}
}

use widgets::button;
use widgets::label as heading_text;

#[sugary]
fn form(
	#[inherits] log: &Log,
	#[inherits] theme: &Theme,
	#[required] title: &str,
	#[required] rows: &[&str],
) -> Vec<String> {
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
	#[inherits] theme: &Theme,
	#[inherits] scale: Scale,
	#[required] value: T,
	#[required] choose: impl Fn(u32) -> u32,
	#[optional = || 0] extra: impl FnOnce() -> u32,
) -> String {
	format!("{value}:{}", choose(theme.accent * scale.0) + extra())
}

#[sugary]
fn tree(#[inherits] theme: &Theme, #[required] depth: u32) -> u32 {
	if depth == 0 {
		theme.accent
	} else {
		call!(tree, depth: depth - 1) + call!(self::tree, depth: depth - 1)
	}
}

#[sugary]
fn lazy<'a>(#[inherits] theme: &'a Theme) -> impl Iterator<Item = String> + 'a {
	(1..=3u32).map(move |size| call!(heading_text, text: "row", size: size))
}

#[sugary]
fn screen(
	#[inherits] theme: &Theme,
	#[inherits] scale: Scale,
	#[required] input: Option<&str>,
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
fn toolbar(#[inherits] theme: &Theme) -> Vec<String> {
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
	pub fn chip(
		#[inherits] theme: &Theme,
		#[required] text: &str,
		#[optional = 1] weight: u32,
	) -> String {
		format!("{text}:{}", theme.accent * weight)
	}
}

use short::chip;

#[sugary]
fn bar(#[inherits] theme: &Theme) -> Vec<String> {
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
fn nothing_inherited(#[required] count: u32, #[optional] label: String) -> String {
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

#[sugary]
fn accent(#[inherits] theme: &Theme, #[inherits] scale: Scale) -> u32 {
	theme.accent * scale.0
}

#[sugary]
fn accents(#[inherits] theme: &Theme, #[inherits] scale: Scale) -> [u32; 4] {
	let other = Theme { accent: 100 };
	[
		call!(accent),
		call!(accent, scale: Scale(5)),
		call!(accent, theme: &other),
		accent!(scale: Scale(3), theme: &other),
	]
}

// A caller that inherits nothing can still call, by giving everything.
#[sugary]
fn from_nothing() -> u32 {
	call!(accent, theme: &Theme { accent: 4 }, scale: Scale(2))
}

#[test]
fn an_inherited_argument_can_be_given_at_the_call() {
	assert_eq!(accents(&Theme { accent: 7 }, Scale(2)), [14, 35, 200, 300]);
	assert_eq!(from_nothing(), 8);
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Key(u32);

#[sugary]
fn leaf(#[inherits(theme)] look: &Theme, #[required(key)] self_key: Key) -> u32 {
	self_key.0 + look.accent
}

#[sugary]
fn branch(#[inherits] theme: &Theme, #[required(key)] self_key: Key) -> [u32; 3] {
	let other = Theme { accent: 100 };
	[
		call!(leaf, key: Key(self_key.0 * 10 + 1)),
		leaf!(key: Key(self_key.0 * 10 + 2)),
		call!(leaf, key: self_key, theme: &other),
	]
}

// Callers write the name in the brackets; the body uses the parameter's own.
#[test]
fn a_parameter_can_be_called_something_else_by_callers() {
	let theme = Theme { accent: 7 };
	assert_eq!(branch(&theme, Key(3)), [38, 39, 103]);
	assert_eq!(leaf(&theme, Key(1)), 8);
}

#[sugary]
fn padded(
	#[required(key)] self_key: Key,
	#[optional(gap)] spacing: u32,
	#[optional(scale = 2)] factor: u32,
) -> u32 {
	(self_key.0 + spacing) * factor
}

#[sugary]
fn paddings() -> [u32; 3] {
	[
		padded!(key: Key(1)),
		padded!(key: Key(1), gap: 4),
		padded!(scale: 10, key: Key(1), gap: 4),
	]
}

#[test]
fn a_default_can_be_named_for_callers_too() {
	assert_eq!(paddings(), [2, 10, 50]);
	assert_eq!(padded(Key(1), 0, 1), 1);
}

// Eight parameters would trip clippy's too_many_arguments, which sugary allows.
#[sugary]
fn many(
	#[inherits] theme: &Theme,
	#[inherits] scale: Scale,
	#[required] a: u32,
	#[required] b: u32,
	#[optional] c: u32,
	#[optional] d: u32,
	#[optional] e: u32,
	#[optional] f: u32,
) -> u32 {
	theme.accent * scale.0 + a + b + c + d + e + f
}

#[sugary]
fn calls_many(#[inherits] theme: &Theme, #[inherits] scale: Scale) -> u32 {
	call!(many, a: 1, b: 2, f: 3)
}

#[test]
fn a_function_with_many_parameters_needs_no_lint_exemption() {
	assert_eq!(calls_many(&Theme { accent: 7 }, Scale(2)), 20);
}
