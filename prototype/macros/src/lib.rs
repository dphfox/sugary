// PROTOTYPE v2. `#[build]` leaves a function untouched and generates what
// `build!(path::to::function, name: value, ...)` needs to expand to a plain
// call. The key is an ordinary named parameter.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Meta, Pat, parse_macro_input, parse_quote};

enum Kind {
	Service,
	Required,
	Default(Tokens),
}

#[proc_macro_attribute]
pub fn buildable(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut func = parse_macro_input!(item as ItemFn);
	let vis = func.vis.clone();
	let name = func.sig.ident.clone();

	let mut params = Vec::new();
	for arg in func.sig.inputs.iter_mut() {
		let FnArg::Typed(typed) = arg else {
			return quote!(compile_error!("#[build] does not support methods");).into();
		};
		let Pat::Ident(pat) = &*typed.pat else {
			return quote!(compile_error!("#[build] parameters must be plain names");).into();
		};
		let ident = pat.ident.clone();
		let ty = (*typed.ty).clone();
		let mut kind = Kind::Required;
		typed.attrs.retain(|attr| {
			if attr.path().is_ident("service") {
				kind = Kind::Service;
				false
			} else if attr.path().is_ident("default") {
				kind = Kind::Default(match &attr.meta {
					Meta::List(list) => list.tokens.clone(),
					_ => quote!(::core::default::Default::default()),
				});
				false
			} else {
				true
			}
		});
		params.push((ident, ty, kind));
	}

	// Inside the function: a local macro that hands the function's own
	// services to whatever build function is called in its body.
	let services: Vec<_> = params
		.iter()
		.filter(|(_, _, kind)| matches!(kind, Kind::Service))
		.map(|(ident, _, _)| ident)
		.collect();
	func.block.stmts.insert(
		0,
		parse_quote! {
			#[allow(unused_macros)]
			macro_rules! __inside_a_build_function {
				([$($callee:tt)*] $($rest:tt)*) => {
					$($callee)* ! { @scope [$($callee)*] (#(#services,)*) $($rest)* }
				};
			}
		},
	);

	let data: Vec<_> = params
		.iter()
		.filter(|(_, _, kind)| !matches!(kind, Kind::Service))
		.collect();
	let slots: Vec<_> = (0..data.len()).map(|i| format_ident!("__slot{i}")).collect();
	let default_fns: Vec<_> = data
		.iter()
		.map(|(ident, _, _)| format_ident!("__default_{ident}"))
		.collect();

	let initial = data.iter().zip(&default_fns).map(|((_, _, kind), default_fn)| match kind {
		Kind::Default(_) => quote!(($($f)::+ :: #default_fn())),
		_ => quote!((@missing)),
	});

	let munch = data.iter().enumerate().map(|(index, (ident, _, _))| {
		let before = &slots[..index];
		let after = &slots[index + 1..];
		let all = &slots;
		let duplicate = format!("`{name}` was given the argument `{ident}` more than once");
		let filled = slots.iter().enumerate().map(|(i, slot)| {
			if i == index { quote!((@given $($old:tt)*)) } else { quote!($ #slot:tt) }
		});
		quote! {
			(@munch $ctx:tt [ #(#filled)* ] #ident : $($rest:tt)*) => {
				compile_error!(#duplicate)
			};
			(@munch ([$($f:ident)::+] $services:tt) [ #($ #all:tt)* ] #ident : $value:expr , $($rest:tt)*) => {
				$($f)::+ ! { @munch ([$($f)::+] $services) [ #($ #before)* (@given $value) #($ #after)* ] $($rest)* }
			};
			(@munch ([$($f:ident)::+] $services:tt) [ #($ #all:tt)* ] #ident : $value:expr) => {
				$($f)::+ ! { @munch ([$($f)::+] $services) [ #($ #before)* (@given $value) #($ #after)* ] }
			};
		}
	});

	let missing = data.iter().enumerate().filter_map(|(index, (ident, _, kind))| {
		if !matches!(kind, Kind::Required) {
			return None;
		}
		let pattern = slots.iter().enumerate().map(|(i, slot)| {
			if i == index { quote!((@missing)) } else { quote!($ #slot:tt) }
		});
		let message = format!("`{name}` is missing its required argument `{ident}`");
		Some(quote! {
			(@munch $ctx:tt [ #(#pattern)* ]) => { compile_error!(#message) };
		})
	});

	// Each slot ends as either `(@given value)` or `(default expression)`.
	// Normalise both to a plain expression before the final call.
	let mut slot_iter = slots.iter();
	let call_args = params.iter().map(|(_, _, kind)| match kind {
		Kind::Service => quote!(::proto::Select::select(&$services)),
		_ => {
			let slot = slot_iter.next().unwrap();
			quote!(::proto::__slot!($ #slot))
		}
	});

	let defaults = data.iter().zip(&default_fns).filter_map(|((_, ty, kind), default_fn)| {
		let Kind::Default(expr) = kind else { return None };
		Some(quote! {
			#[inline(always)]
			pub fn #default_fn() -> #ty { #expr }
		})
	});

	let unknown = format!("`{name}` has no argument named `");
	let unique = {
		use std::hash::{Hash, Hasher};
		let mut hasher = std::collections::hash_map::DefaultHasher::new();
		quote!(#func).to_string().hash(&mut hasher);
		let place = name.span().unwrap();
		(place.file(), place.line(), place.column()).hash(&mut hasher);
		hasher.finish()
	};
	let companion = format_ident!("__{name}_build_{unique:x}");
	let (export, alias_vis) = match vis {
		syn::Visibility::Inherited => (quote!(), quote!()),
		syn::Visibility::Public(_) => (quote!(#[macro_export]), quote!(pub)),
		_ => (quote!(), quote!(pub(crate))),
	};

	quote! {
		#func

		#[doc(hidden)]
		#[allow(unused_imports, non_snake_case)]
		#vis mod #name {
			use super::*;
			#(#defaults)*
		}

		#[doc(hidden)]
		#[allow(unused_macros)]
		#export
		macro_rules! #companion {
			(@scope [$($f:ident)::+] $services:tt $($args:tt)*) => {
				$($f)::+ ! { @munch ([$($f)::+] $services) [ #(#initial)* ] $($args)* }
			};
			#(#munch)*
			#(#missing)*
			(@munch ([$($f:ident)::+] $services:tt) [ #($ #slots:tt)* ]) => {
				$($f)::+ (#(#call_args),*)
			};
			(@munch $ctx:tt $slots:tt $bad:ident : $($rest:tt)*) => {
				compile_error!(concat!(#unknown, stringify!($bad), "`"))
			};
			(@munch $($rest:tt)*) => {
				compile_error!("arguments must be written as `name: value`")
			};
			// The short form: one rule that forwards to the shared macro.
			($($args:tt)*) => {
				::proto::build!(#name, $($args)*)
			};
		}
		#[allow(unused_imports)]
		#alias_vis use #companion as #name;
	}
	.into()
}
