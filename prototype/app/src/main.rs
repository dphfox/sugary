// PROTOTYPE driver.

mod ui;

use proto::{Key, States, Theme};

fn main() {
	let states = States::default();
	let theme = Theme { accent: 7 };
	let rows = vec![(1u32, "first".to_string()), (2, "second".to_string())];

	let sugared = ui::screens::form(Key::ROOT, &states, &theme, "Title", &rows);
	let plain = ui::screens::form_plain(Key::ROOT, &states, &theme, "Title", &rows);

	println!("{sugared:#?}");
	println!("identical to plain calls: {}", sugared == plain);
}
