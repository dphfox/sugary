# sugary

Nicer function parameters for Rust: named, default and inherited arguments
that compile to plain calls.

```rust
use sugary::{call, sugary};

pub struct Theme {
    pub accent: u32,
}

#[sugary]
fn label(
    #[inherit] theme: &Theme,
    #[require] text: &str,
    #[default = 1] size: u32,
) -> String {
    format!("{text} in colour {} at size {size}", theme.accent)
}

#[sugary]
fn heading(
    #[inherit] theme: &Theme,
    #[require] title: &str,
) -> String {
    // Named, in any order. `size` could be left out. `theme` is inherited.
    call!(label, size: 2, text: title)
}
```

A `#[sugary]` function stays an ordinary function. `label(&theme, "Hello", 2)`
still works, and `call!` expands to exactly that. Nothing is stored anywhere
at run time.

## The three kinds of parameter

Every parameter of a sugary function carries exactly one marker, because the
choice is the function's contract with its callers. At the call, every
argument is written `name: value`, in any order.

- `#[require]`: the argument must be given. Adding a required parameter
  breaks every existing call.
- `#[default = expression]`, or `#[default]` for the type's own default: the
  argument may be left out.
- `#[inherit]`: the argument may be left out, and is then taken from the
  caller's own inherited parameters, matched by type. Giving it at the call
  overrides what would have been inherited.

## Rules

- `call!` only works inside a `#[sugary]` function, because that is where
  inherited arguments come from. At the top of a call chain, call the
  function plainly.
- A caller must itself inherit everything its callees inherit, except what
  it gives them at the call.
- Inherited parameters must have distinct types within one function, and
  must be `Copy`, which shared references are. A function can have up to
  eight.
- `call!(function, ...)` accepts the function however it is in scope: under
  its own name, renamed, or by path.
- `function!(...)` is a shorter form of `call!(function, ...)`. It only
  works when the function is in scope under its own name.

## Errors

Mistakes are caught when compiling:

- A parameter with no marker, or more than one: `` `size` must be marked with exactly one of #[inherit], #[require] or #[default] ``
- A required argument left out: `` `label` is missing its required argument `text` ``
- An argument the function does not have: `` `label` has no argument named `colour` ``
- An argument given twice: `` `label` was given the argument `size` more than once ``
- An inherited argument the caller lacks: `the calling function has no inherited `&Clock` to pass on`,
  or a plain type mismatch when the caller inherits exactly one thing.
- A call from outside a sugary function: `` cannot find macro `__only_inside_a_sugary_function` ``

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

In an optimised build, a function written with `call!` and the same function
written with plain calls compile to identical machine code.

## Limitations

- A default for a borrowed parameter, such as `#[default = "none"] text: &str`,
  does not compile yet.
- A default on a parameter whose type is a generic parameter does not work.
- Each sugary function generates a hidden module of the same name, which
  clashes with a real module of that name in the same place.
- Two inherited parameters of the same type in one function give an obscure
  compiler error.
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
