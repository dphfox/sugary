//! The `#[sugary]` attribute. Use it through the `sugary` crate, which
//! documents how it works.
//!
//! The attribute leaves the function it is applied to as an ordinary
//! function, and generates what `sugary::call!` needs in order to expand a
//! call with named arguments into a plain call to it.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Meta, Pat, Visibility, parse_macro_input, parse_quote};

enum Kind {
	Inherited,
	Required,
	Default(Tokens),
}

#[proc_macro_attribute]
pub fn sugary(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut function = parse_macro_input!(item as ItemFn);
	let visibility = function.vis.clone();
	let name = function.sig.ident.clone();

	let mut parameters = Vec::new();
	let mut inherited = Vec::new();
	for argument in function.sig.inputs.iter_mut() {
		let FnArg::Typed(typed) = argument else {
			return error(&name, "#[sugary] cannot be used on a method");
		};
		let Pat::Ident(pattern) = &*typed.pat else {
			return error(&name, "#[sugary] needs every parameter to be a plain name");
		};
		let ident = pattern.ident.clone();
		let ty = (*typed.ty).clone();
		let mut kinds = Vec::new();
		let mut bracketed_default = false;
		// The name callers write, when it differs from the parameter's own.
		let mut label = None;
		let mut bad_label = false;
		let own_default = || quote!(::core::default::Default::default());
		typed.attrs.retain(|attribute| {
			let bracketed = matches!(attribute.meta, Meta::List(_));
			if attribute.path().is_ident("inherits") || attribute.path().is_ident("required") {
				if bracketed {
					match attribute.parse_args::<syn::Ident>() {
						Ok(given) => label = Some(given),
						Err(_) => bad_label = true,
					}
				}
				kinds.push(if attribute.path().is_ident("inherits") {
					Kind::Inherited
				} else {
					Kind::Required
				});
			} else if attribute.path().is_ident("optional") {
				// In brackets: the callers' name, alone or with a value.
				kinds.push(Kind::Default(match &attribute.meta {
					Meta::NameValue(pair) => {
						let value = &pair.value;
						quote!(#value)
					}
					Meta::Path(_) => own_default(),
					Meta::List(_) => {
						if let Ok(given) = attribute.parse_args::<syn::Ident>() {
							label = Some(given);
							own_default()
						} else if let Ok(pair) = attribute.parse_args::<syn::MetaNameValue>()
							&& let Some(given) = pair.path.get_ident()
						{
							label = Some(given.clone());
							let value = &pair.value;
							quote!(#value)
						} else {
							bracketed_default = true;
							quote!()
						}
					}
				}));
			} else {
				return true;
			}
			false
		});
		if bracketed_default {
			return error(
				&ident,
				&format!(
					"write the default for `{ident}` as #[optional = value], or as #[optional(name = value)] to name it for callers"
				),
			);
		}
		// The choice is the function's contract with its callers, so it is
		// always written down: adding a required parameter breaks them.
		let Some(kind) = kinds.pop().filter(|_| kinds.is_empty()) else {
			return error(
				&ident,
				&format!(
					"`{ident}` must be marked with exactly one of #[inherits], #[required] or #[optional]"
				),
			);
		};
		if bad_label {
			return error(
				&ident,
				&format!("the name callers use for `{ident}` must be a single word"),
			);
		}
		if matches!(kind, Kind::Inherited) {
			inherited.push(ident.clone());
		}
		parameters.push((label.unwrap_or(ident), ty, kind));
	}

	// A macro defined inside a function can see that function's parameters.
	// This one names the inherited ones, so that a `call!` written in the
	// body can pass them on. It produces no code of its own.
	function.block.stmts.insert(
		0,
		parse_quote! {
			#[allow(unused_macros)]
			macro_rules! __only_inside_a_sugary_function {
				([$($callee:tt)*] $($arguments:tt)*) => {
					$($callee)* ! { @inherit [$($callee)*] (#(#inherited,)*) $($arguments)* }
				};
			}
		},
	);

	let named: Vec<_> = parameters.iter().collect();
	let slots: Vec<_> = (0..named.len())
		.map(|index| format_ident!("__slot{index}"))
		.collect();
	let default_functions: Vec<_> = named
		.iter()
		.map(|(ident, _, _)| format_ident!("__sugary_default_{ident}"))
		.collect();

	// Every parameter has a slot, which starts out missing, holding its
	// default, or waiting to be inherited, and is overwritten when the
	// argument is given.
	let initial_slots = named
		.iter()
		.zip(&default_functions)
		.map(|((_, _, kind), default)| match kind {
			Kind::Default(_) => quote!(($($function)::+ :: #default())),
			Kind::Inherited => quote!((@inherit)),
			Kind::Required => quote!((@missing)),
		});

	let fill_slot = named.iter().enumerate().map(|(index, (ident, _, _))| {
		let before = &slots[..index];
		let after = &slots[index + 1..];
		let all = &slots;
		let repeated = format!("`{name}` was given the argument `{ident}` more than once");
		let already_given = slots.iter().enumerate().map(|(position, slot)| {
			if position == index { quote!((@given $($previous:tt)*)) } else { quote!($ #slot:tt) }
		});
		quote! {
			(@sort $context:tt [ #(#already_given)* ] #ident : $($rest:tt)*) => {
				compile_error!(#repeated)
			};
			(@sort ([$($function:ident)::+] $inherited:tt) [ #($ #all:tt)* ] #ident : $value:expr , $($rest:tt)*) => {
				$($function)::+ ! { @sort ([$($function)::+] $inherited) [ #($ #before)* (@given $value) #($ #after)* ] $($rest)* }
			};
			(@sort ([$($function:ident)::+] $inherited:tt) [ #($ #all:tt)* ] #ident : $value:expr) => {
				$($function)::+ ! { @sort ([$($function)::+] $inherited) [ #($ #before)* (@given $value) #($ #after)* ] }
			};
		}
	});

	let missing = named
		.iter()
		.enumerate()
		.filter_map(|(index, (ident, _, kind))| {
			if !matches!(kind, Kind::Required) {
				return None;
			}
			let pattern = slots.iter().enumerate().map(|(position, slot)| {
				if position == index {
					quote!((@missing))
				} else {
					quote!($ #slot:tt)
				}
			});
			let message = format!("`{name}` is missing its required argument `{ident}`");
			Some(quote! {
				(@sort $context:tt [ #(#pattern)* ]) => { compile_error!(#message) };
			})
		});

	// The plain call, with arguments back in the order of the signature.
	// An inherited parameter that was not given is picked out of the
	// caller's by its type.
	let call_arguments = parameters
		.iter()
		.zip(&slots)
		.map(|((_, _, kind), slot)| match kind {
			Kind::Inherited => quote!(::sugary::__sugary_inherited!($ #slot $inherited)),
			_ => quote!(::sugary::__sugary_slot!($ #slot)),
		});

	// Defaults live in functions beside the one they belong to, so that the
	// expression is resolved where it was written and not at the call site.
	let defaults = named
		.iter()
		.zip(&default_functions)
		.filter_map(|((_, ty, kind), default)| {
			let Kind::Default(expression) = kind else {
				return None;
			};
			Some(quote! {
				#[inline(always)]
				pub fn #default() -> #ty { #expression }
			})
		});

	// Exported macros share one namespace per crate, so the hidden name has
	// to differ between two functions that share a name.
	let unique = {
		use std::hash::{Hash, Hasher};
		let mut hasher = std::collections::hash_map::DefaultHasher::new();
		quote!(#function).to_string().hash(&mut hasher);
		let place = name.span().unwrap();
		(place.file(), place.line(), place.column()).hash(&mut hasher);
		hasher.finish()
	};
	let hidden = format_ident!("__sugary_{name}_{unique:x}");
	let (export, alias_visibility) = match visibility {
		Visibility::Inherited => (quote!(), quote!()),
		Visibility::Public(_) => (quote!(#[macro_export]), quote!(pub)),
		_ => (quote!(), quote!(pub(crate))),
	};
	let unknown = format!("`{name}` has no argument named `");

	quote! {
		#function

		#[doc(hidden)]
		#[allow(unused_imports, non_snake_case)]
		#visibility mod #name {
			use super::*;
			#(#defaults)*
		}

		#[doc(hidden)]
		#[allow(unused_macros)]
		#export
		macro_rules! #hidden {
			(@inherit [$($function:ident)::+] $inherited:tt $($arguments:tt)*) => {
				$($function)::+ ! { @sort ([$($function)::+] $inherited) [ #(#initial_slots)* ] $($arguments)* }
			};
			#(#fill_slot)*
			#(#missing)*
			(@sort ([$($function:ident)::+] $inherited:tt) [ #($ #slots:tt)* ]) => {
				$($function)::+ (#(#call_arguments),*)
			};
			(@sort $context:tt $slots:tt $unknown:ident : $($rest:tt)*) => {
				compile_error!(concat!(#unknown, stringify!($unknown), "`"))
			};
			(@sort $($rest:tt)*) => {
				compile_error!("arguments must be written as `name: value`")
			};
			// The short form, which only works when the function is in scope
			// under its own name.
			($($arguments:tt)*) => {
				::sugary::call!(#name, $($arguments)*)
			};
		}
		#[allow(unused_imports)]
		#alias_visibility use #hidden as #name;
	}
	.into()
}

fn error(name: &syn::Ident, message: &str) -> TokenStream {
	syn::Error::new(name.span(), message)
		.to_compile_error()
		.into()
}
