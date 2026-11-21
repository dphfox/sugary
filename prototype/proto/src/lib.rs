// PROTOTYPE stand-ins for the real library's types.

use std::cell::Cell;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::Div;

pub use proto_macros::buildable;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Key(pub u64);

impl Key {
	pub const ROOT: Key = Key(0);
}

impl<T: Hash> Div<T> for Key {
	type Output = Key;

	#[inline(always)]
	fn div(self, name: T) -> Key {
		let mut hasher = DefaultHasher::new();
		self.0.hash(&mut hasher);
		name.hash(&mut hasher);
		Key(hasher.finish())
	}
}

#[derive(Default)]
pub struct States {
	pub asked: Cell<u64>,
}

impl States {
	#[inline(always)]
	pub fn get(&self, key: Key) -> u64 {
		self.asked.set(self.asked.get().wrapping_add(key.0));
		key.0
	}
}

pub struct Theme {
	pub accent: u32,
}

#[derive(Debug, PartialEq, Default)]
pub struct Comp {
	pub items: Vec<(u64, u32, String)>,
}

impl Comp {
	pub fn one(key: Key, colour: u32, text: impl Into<String>) -> Comp {
		Comp { items: vec![(key.0, colour, text.into())] }
	}

	pub fn of(parts: impl IntoIterator<Item = Comp>) -> Comp {
		Comp { items: parts.into_iter().flat_map(|part| part.items).collect() }
	}
}

/// Picks the service of type `T` out of the calling function's services.
#[diagnostic::on_unimplemented(
	message = "the calling build function has no `{T}` service to pass on",
	label = "this call needs a `{T}` service",
	note = "add a `#[service]` parameter of type `{T}` to the function this call is inside"
)]
pub trait Select<T, Index> {
	fn select(&self) -> T;
}

pub struct I0;
pub struct I1;
pub struct I2;
pub struct I3;

macro_rules! select {
	($index:ident $field:tt $pick:ident [$($all:ident),*]) => {
		impl<$($all: Copy),*> Select<$pick, $index> for ($($all,)*) {
			#[inline(always)]
			fn select(&self) -> $pick {
				self.$field
			}
		}
	};
}

select!(I0 0 A [A]);
select!(I0 0 A [A, B]);
select!(I1 1 B [A, B]);
select!(I0 0 A [A, B, C]);
select!(I1 1 B [A, B, C]);
select!(I2 2 C [A, B, C]);
select!(I0 0 A [A, B, C, D]);
select!(I1 1 B [A, B, C, D]);
select!(I2 2 C [A, B, C, D]);
select!(I3 3 D [A, B, C, D]);

/// PROTOTYPE v2: the one shared macro. The function is named at the call
/// site, so it resolves however it was imported.
#[macro_export]
macro_rules! build {
	($($f:ident)::+ $(, $($args:tt)*)?) => {
		__inside_a_build_function! { [$($f)::+] $($($args)*)? }
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __slot {
	((@given $value:expr)) => { $value };
	(($default:expr)) => { $default };
}
