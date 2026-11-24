//! Nicer function parameters: named, default and inherited arguments that
//! compile to plain calls.
//!
//! Mark a function with [`macro@sugary`] and it stays an ordinary function.
//! What it gains is a nicer way to be called, through [`call!`]:
//!
//! ```
//! use sugary::{call, sugary};
//!
//! pub struct Theme {
//!     pub accent: u32,
//! }
//!
//! #[sugary]
//! fn label(
//!     #[inherits] theme: &Theme,
//!     #[required] text: &str,
//!     #[optional = 1] size: u32,
//! ) -> String {
//!     format!("{text} in colour {} at size {size}", theme.accent)
//! }
//!
//! #[sugary]
//! fn heading(
//!     #[inherits] theme: &Theme,
//!     #[required] title: &str,
//! ) -> String {
//!     // Named, in any order. `size` could be left out. `theme` is inherited.
//!     call!(label, size: 2, text: title)
//! }
//!
//! let theme = Theme { accent: 7 };
//! // A sugary function is still a plain function.
//! assert_eq!(heading(&theme, "Hello"), label(&theme, "Hello", 2));
//! ```
//!
//! # The three kinds of parameter
//!
//! Every parameter of a sugary function carries exactly one marker, because
//! the choice is the function's contract with its callers. At the call,
//! every argument is written `name: value`, in any order.
//!
//! - `#[required]`: the argument must be given. Adding a required parameter
//!   breaks every existing call.
//! - `#[optional = expression]`, or `#[optional]` for the type's own default:
//!   the argument may be left out.
//! - `#[inherits]`: the argument may be left out, and is then taken from the
//!   caller's own inherited parameters, matched by type. Giving it at the
//!   call overrides what would have been inherited.
//!
//! A parameter can be given a different name for callers to write, in
//! brackets: `#[required(key)] self_key: Key` is called as `key: ...`, and is
//! `self_key` inside the function. The same works for `#[inherits(name)]`,
//! for `#[optional(name)]`, and with a value as `#[optional(name = value)]`.
//!
//! # Rules
//!
//! - `call!` only works inside a `#[sugary]` function, because that is where
//!   inherited arguments come from. At the top of a call chain, call the
//!   function plainly.
//! - A caller must itself inherit everything its callees inherit, except
//!   what it gives them at the call.
//! - Inherited parameters must have distinct types within one function, and
//!   must be `Copy`, which shared references are.
//! - `call!(function, ...)` accepts the function however it is in scope:
//!   under its own name, renamed, or by path.
//! - `function!(...)` is a shorter form of `call!(function, ...)`. It only
//!   works when the function is in scope under its own name.
//!
//! # How it works
//!
//! `#[sugary]` adds one local macro to the top of the function body, which
//! knows the names of the function's inherited parameters and produces no
//! code. Beside the function it generates a macro of the same name and a
//! hidden module holding the defaults. `call!` asks the caller's local macro
//! for the inherited parameters, sorts the named arguments into the order of
//! the signature, fills in defaults, and expands to a plain call. Nothing is
//! stored anywhere at run time.

pub use sugary_macros::sugary;

/// Calls a `#[sugary]` function with named arguments, from inside another
/// `#[sugary]` function. See the [crate documentation](crate).
#[macro_export]
macro_rules! call {
	($($function:ident)::+ $(, $($arguments:tt)*)?) => {
		__only_inside_a_sugary_function! { [$($function)::+] $($($arguments)*)? }
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sugary_slot {
	((@given $value:expr)) => {
		$value
	};
	(($default:expr)) => {
		$default
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sugary_inherited {
	((@given $value:expr) $inherited:tt) => {
		$value
	};
	((@inherit) $inherited:tt) => {
		$crate::__private::Select::select(&$inherited)
	};
}

#[doc(hidden)]
pub mod __private {
	/// Picks the inherited argument of type `T` out of the caller's.
	#[diagnostic::on_unimplemented(
		message = "the calling function has no inherited `{T}` to pass on",
		label = "this call needs an inherited `{T}`",
		note = "add an `#[inherits]` parameter of type `{T}` to the function this call is inside"
	)]
	pub trait Select<T, Index> {
		fn select(&self) -> T;
	}

	/// Implemented by nothing. It exists so that a caller which inherits
	/// nothing gets an error that names the type it was asked for.
	#[diagnostic::on_unimplemented(
		message = "the calling function has no inherited `{Self}` to pass on",
		label = "this call needs an inherited `{Self}`",
		note = "add an `#[inherits]` parameter of type `{Self}` to the function this call is inside"
	)]
	pub trait Inherited {}

	impl<T: Inherited> Select<T, I0> for () {
		fn select(&self) -> T {
			unreachable!()
		}
	}

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

	pub struct I0;
	pub struct I1;
	pub struct I2;
	pub struct I3;
	pub struct I4;
	pub struct I5;
	pub struct I6;
	pub struct I7;

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
	select!(I0 0 A [A, B, C, D, E]);
	select!(I1 1 B [A, B, C, D, E]);
	select!(I2 2 C [A, B, C, D, E]);
	select!(I3 3 D [A, B, C, D, E]);
	select!(I4 4 E [A, B, C, D, E]);
	select!(I0 0 A [A, B, C, D, E, F]);
	select!(I1 1 B [A, B, C, D, E, F]);
	select!(I2 2 C [A, B, C, D, E, F]);
	select!(I3 3 D [A, B, C, D, E, F]);
	select!(I4 4 E [A, B, C, D, E, F]);
	select!(I5 5 F [A, B, C, D, E, F]);
	select!(I0 0 A [A, B, C, D, E, F, G]);
	select!(I1 1 B [A, B, C, D, E, F, G]);
	select!(I2 2 C [A, B, C, D, E, F, G]);
	select!(I3 3 D [A, B, C, D, E, F, G]);
	select!(I4 4 E [A, B, C, D, E, F, G]);
	select!(I5 5 F [A, B, C, D, E, F, G]);
	select!(I6 6 G [A, B, C, D, E, F, G]);
	select!(I0 0 A [A, B, C, D, E, F, G, H]);
	select!(I1 1 B [A, B, C, D, E, F, G, H]);
	select!(I2 2 C [A, B, C, D, E, F, G, H]);
	select!(I3 3 D [A, B, C, D, E, F, G, H]);
	select!(I4 4 E [A, B, C, D, E, F, G, H]);
	select!(I5 5 F [A, B, C, D, E, F, G, H]);
	select!(I6 6 G [A, B, C, D, E, F, G, H]);
	select!(I7 7 H [A, B, C, D, E, F, G, H]);
}

/// What goes wrong, and that it is caught when compiling.
///
/// A parameter with no marker:
///
/// ```compile_fail
/// # use sugary::sugary;
/// #[sugary] fn label(size: u32) -> u32 { size }
/// ```
///
/// A parameter with two markers:
///
/// ```compile_fail
/// # use sugary::sugary;
/// #[sugary] fn label(#[required] #[optional] size: u32) -> u32 { size }
/// ```
///
/// A default written in brackets:
///
/// ```compile_fail
/// # use sugary::sugary;
/// #[sugary] fn label(#[optional(1)] size: u32) -> u32 { size }
/// ```
///
/// A required argument left out:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// #[sugary] fn label(#[required] text: &str, #[optional = 1] size: u32) -> u32 { size }
/// #[sugary] fn caller() -> u32 { call!(label, size: 2) }
/// ```
///
/// An argument the function does not have:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// #[sugary] fn label(#[required] text: &str) -> usize { text.len() }
/// #[sugary] fn caller() -> usize { call!(label, text: "x", colour: 3) }
/// ```
///
/// An argument given twice:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// #[sugary] fn label(#[required] size: u32) -> u32 { size }
/// #[sugary] fn caller() -> u32 { call!(label, size: 2, size: 3) }
/// ```
///
/// A call from outside a sugary function:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// #[sugary] fn label(#[required] size: u32) -> u32 { size }
/// fn caller() -> u32 { call!(label, size: 2) }
/// ```
///
/// An inherited argument the caller does not have:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// struct Theme;
/// struct Clock;
/// #[sugary] fn timed(#[inherits] clock: &Clock) -> u32 { let _ = clock; 0 }
/// #[sugary] fn caller(#[inherits] theme: &Theme) -> u32 { let _ = theme; call!(timed) }
/// ```
///
/// An inherited argument, when the caller inherits nothing at all:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// struct Clock;
/// #[sugary] fn timed(#[inherits] clock: &Clock) -> u32 { let _ = clock; 0 }
/// #[sugary] fn caller() -> u32 { call!(timed) }
/// ```
///
/// The short form under a renamed import:
///
/// ```compile_fail
/// # use sugary::sugary;
/// mod widgets {
///     # use sugary::sugary;
///     #[sugary] pub fn chip(#[required] size: u32) -> u32 { size }
/// }
/// use widgets::chip as tag;
/// #[sugary] fn caller() -> u32 { tag!(size: 2) }
/// ```
#[cfg(doctest)]
struct CompileErrors;
