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
//! fn label(#[inherit] theme: &Theme, text: &str, #[default(1)] size: u32) -> String {
//!     format!("{text} in colour {} at size {size}", theme.accent)
//! }
//!
//! #[sugary]
//! fn heading(#[inherit] theme: &Theme, title: &str) -> String {
//!     // Named, in any order. `size` could be left out. `theme` is inherited.
//!     call!(label, size: 2, text: title)
//! }
//!
//! let theme = Theme { accent: 7 };
//! // A sugary function is still a plain function.
//! assert_eq!(heading(&theme, "Hello"), label(&theme, "Hello", 2));
//! ```
//!
//! # The three kinds of argument
//!
//! - **Named arguments** are written `name: value`, in any order.
//! - **Default arguments** may be left out. Mark the parameter
//!   `#[default(expression)]`, or `#[default]` to use the type's own default.
//! - **Inherited arguments** may be left out too. Mark the parameter
//!   `#[inherit]`, and when it is not given its value is taken from the
//!   caller's own inherited parameters, matched by type. Giving it at the
//!   call, as `name: value`, overrides what would have been inherited.
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
		note = "add an `#[inherit]` parameter of type `{T}` to the function this call is inside"
	)]
	pub trait Select<T, Index> {
		fn select(&self) -> T;
	}

	/// Implemented by nothing. It exists so that a caller which inherits
	/// nothing gets an error that names the type it was asked for.
	#[diagnostic::on_unimplemented(
		message = "the calling function has no inherited `{Self}` to pass on",
		label = "this call needs an inherited `{Self}`",
		note = "add an `#[inherit]` parameter of type `{Self}` to the function this call is inside"
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
/// A required argument left out:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// #[sugary] fn label(text: &str, #[default(1)] size: u32) -> u32 { size }
/// #[sugary] fn caller() -> u32 { call!(label, size: 2) }
/// ```
///
/// An argument the function does not have:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// #[sugary] fn label(text: &str) -> usize { text.len() }
/// #[sugary] fn caller() -> usize { call!(label, text: "x", colour: 3) }
/// ```
///
/// An argument given twice:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// #[sugary] fn label(size: u32) -> u32 { size }
/// #[sugary] fn caller() -> u32 { call!(label, size: 2, size: 3) }
/// ```
///
/// A call from outside a sugary function:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// #[sugary] fn label(size: u32) -> u32 { size }
/// fn caller() -> u32 { call!(label, size: 2) }
/// ```
///
/// An inherited argument the caller does not have:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// struct Theme;
/// struct Clock;
/// #[sugary] fn timed(#[inherit] clock: &Clock) -> u32 { let _ = clock; 0 }
/// #[sugary] fn caller(#[inherit] theme: &Theme) -> u32 { let _ = theme; call!(timed) }
/// ```
///
/// An inherited argument, when the caller inherits nothing at all:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// struct Clock;
/// #[sugary] fn timed(#[inherit] clock: &Clock) -> u32 { let _ = clock; 0 }
/// #[sugary] fn caller() -> u32 { call!(timed) }
/// ```
///
/// The short form under a renamed import:
///
/// ```compile_fail
/// # use sugary::sugary;
/// mod widgets {
///     # use sugary::sugary;
///     #[sugary] pub fn chip(size: u32) -> u32 { size }
/// }
/// use widgets::chip as tag;
/// #[sugary] fn caller() -> u32 { tag!(size: 2) }
/// ```
#[cfg(doctest)]
struct CompileErrors;
