# sugary

Nicer function parameters for Rust: named, default and inherited arguments
that compile to plain calls.

```rust
use sugary::{call, sugary};

pub struct Theme {
    pub accent: u32,
}

#[sugary]
fn label(#[inherit] theme: &Theme, text: &str, #[default(1)] size: u32) -> String {
    format!("{text} in colour {} at size {size}", theme.accent)
}

#[sugary]
fn heading(#[inherit] theme: &Theme, title: &str) -> String {
    // Named, in any order. `size` could be left out. `theme` is inherited.
    call!(label, size: 2, text: title)
}
```

A `#[sugary]` function stays an ordinary function. `label(&theme, "Hello", 2)`
still works, and `call!` expands to exactly that. Nothing is stored anywhere
at run time.

## The three kinds of argument

- **Named arguments** are written `name: value`, in any order. Every
  parameter that is not inherited is named.
- **Default arguments** may be left out. Mark the parameter
  `#[default(expression)]`, or `#[default]` to use the type's own default.
- **Inherited arguments** are never written at the call site. Mark the
  parameter `#[inherit]`, and its value is taken from the caller's own
  inherited parameters, matched by type.

## Rules

- `call!` works anywhere. Inside a `#[sugary]` function it passes on that
  function's inherited arguments. Anywhere else, such as `main` or a test,
  it has nothing to pass on, so named and default arguments work, and the
  function called must not need an inherited one.
- A caller must itself inherit everything its callees inherit. At the top of
  a call chain, where the inherited values are created, call the function
  plainly.
- Inherited parameters must have distinct types within one function, and
  must be `Copy`, which shared references are. A function can have up to
  eight.
- `call!(function, ...)` accepts the function however it is in scope: under
  its own name, renamed, or by path.
- `function!(...)` is a shorter form of `call!(function, ...)`. It only
  works when the function is in scope under its own name.

## Errors

Mistakes are caught when compiling:

- A required argument left out: `` `label` is missing its required argument `text` ``
- An argument the function does not have: `` `label` has no argument named `colour` ``
- An argument given twice: `` `label` was given the argument `size` more than once ``
- An inherited argument the caller lacks: `the calling function has no inherited `&Clock` to pass on`,
  or a plain type mismatch when the caller inherits exactly one thing. A
  caller that is not `#[sugary]` inherits nothing, so it gets the same error
  from any function that needs an inherited argument.

## How it works

`#[sugary]` adds one local macro to the top of the function body, which knows
the names of the function's inherited parameters and produces no code. Beside
the function it generates a macro of the same name and a hidden module
holding the defaults. A function, a module and a macro can share a name
because they live in separate namespaces, and one `use` imports all three.

`call!` asks the caller's local macro for the inherited parameters, sorts the
named arguments into the order of the signature, fills in defaults, and
expands to a plain call. Each inherited parameter is picked out of the
caller's by a small trait implemented for tuples, so the compiler does the
matching by type.

`call!` finds the caller's local macro through the path
`sugary::__caller::inherited`. `#[sugary]` also adds a local module named
`sugary` to the function body, which re-exports the crate and puts the local
macro at that path. Outside a sugary function the same path leads to the
crate itself, where the macro passes on nothing.

In an optimised build, a function written with `call!` and the same function
written with plain calls compile to identical machine code.

## Limitations

- A default for a borrowed parameter, such as `#[default("none")] text: &str`,
  does not compile yet.
- A default on a parameter whose type is a generic parameter does not work.
- Each sugary function generates a hidden module of the same name, which
  clashes with a real module of that name in the same place.
- Two inherited parameters of the same type in one function give an obscure
  compiler error.
- `call!` reaches the crate by the name `sugary`. It does not work where
  that name means something else, such as a dependency renamed in
  `Cargo.toml` or a module of your own called `sugary`, or inside a module
  marked `#[no_implicit_prelude]`.
- A plain `fn` nested inside a sugary function cannot inherit from it. A
  closure can.
- Calling a function that is not `#[sugary]` reports "cannot find macro",
  which does not suggest the fix.
- Editor completion and hover inside a `call!` have not been checked.
  rust-analyzer's diagnostics report no false errors.

## Layout

- The root crate is `sugary`, which is the one to depend on.
- `macros/` is `sugary-macros`, the attribute itself. Rust requires it to be
  a separate crate.
- `tests/kit/` is a second crate used only to test calls across a crate
  boundary.
