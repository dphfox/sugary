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
//! - **Named arguments** are written `name: value`, in any order. Every
//!   parameter that is not inherited is named.
//! - **Default arguments** may be left out. Mark the parameter
//!   `#[default(expression)]`, or `#[default]` to use the type's own default.
//! - **Inherited arguments** are never written at the call site. Mark the
//!   parameter `#[inherit]`, and its value is taken from the caller's own
//!   inherited parameters, matched by type.
//!
//! # Rules
//!
//! - `call!` works anywhere. Inside a `#[sugary]` function it passes on that
//!   function's inherited arguments. Anywhere else it has nothing to pass
//!   on, so named and default arguments work, and the function called must
//!   not need an inherited one.
//! - A caller must itself inherit everything its callees inherit. At the top
//!   of a call chain, where the inherited values are created, call the
//!   function plainly.
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
//!
//! `call!` finds the caller's local macro through the path
//! `sugary::__caller::inherited`. `#[sugary]` also adds a local module named
//! `sugary` to the function body, which re-exports this crate and puts the
//! local macro at that path. Outside a sugary function the same path leads
//! to this crate, where the macro passes on nothing. This is why the crate
//! must be in scope under the name `sugary` wherever `call!` is used.

pub use sugary_macros::sugary;

/// Calls a `#[sugary]` function with named arguments. Inside another
/// `#[sugary]` function, that function's inherited arguments are passed on.
/// See the [crate documentation](crate).
#[macro_export]
macro_rules! call {
	// `sugary` is deliberately not `$crate`: inside a `#[sugary]` function it
	// names a module local to that function, and anywhere else this crate.
	($($function:ident)::+ $(, $($arguments:tt)*)?) => {
		sugary::__caller::inherited! { [$($function)::+] $($($arguments)*)? }
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sugary_nothing_inherited {
	([$($callee:tt)*] $($arguments:tt)*) => {
		$($callee)* ! { @inherit [$($callee)*] () $($arguments)* }
	};
}

/// What a call finds when it is not inside a `#[sugary]` function.
#[doc(hidden)]
pub mod __caller {
	pub use crate::__sugary_nothing_inherited as inherited;
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
		note = "the function this call is inside must be `#[sugary]` and have an `#[inherit]` parameter of type `{Self}`",
		note = "alternatively, call the function plainly and pass every argument"
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
/// A call from outside a sugary function is not a mistake:
///
/// ```
/// # use sugary::{call, sugary};
/// #[sugary] fn label(size: u32, #[default(1)] weight: u32) -> u32 { size * weight }
/// fn caller() -> u32 { call!(label, size: 2) + label!(weight: 3, size: 2) }
/// assert_eq!(caller(), 8);
/// ```
///
/// Unless the function called needs an inherited argument:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// struct Theme;
/// #[sugary] fn label(#[inherit] theme: &Theme, size: u32) -> u32 { let _ = theme; size }
/// fn caller() -> u32 { call!(label, size: 2) }
/// ```
///
/// The same goes for a sugary function that inherits nothing:
///
/// ```compile_fail
/// # use sugary::{call, sugary};
/// struct Theme;
/// #[sugary] fn label(#[inherit] theme: &Theme, size: u32) -> u32 { let _ = theme; size }
/// #[sugary] fn caller() -> u32 { call!(label, size: 2) }
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
