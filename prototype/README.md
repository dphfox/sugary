# Prototype

Throwaway code. It answered one question: can an attribute leave a function
untouched and still give it named arguments, defaults, and arguments
inherited from the calling function, while compiling down to a plain call
with no hidden state? The answer is yes.

Names here (`buildable`, `build!`, `#[service]`, `proto`) are leftovers from
the project the idea came from and are not the intended names.

## Names decided for the real thing

| Prototype | Intended |
|---|---|
| `#[buildable]` on a function | `#[sugary]` |
| `build!(function, name: value)` | `call!(function, name: value)` |
| `#[service]` on a parameter | `#[inherit]` |
| `#[default(...)]` on a parameter | unchanged |
| `function!(name: value)` short form | kept; forwards to `call!` |
| `proto` and `proto_macros` crates | `sugary` and `sugary-macros` |

The three features are called named arguments, default arguments and
inherited arguments. The two functions in a call are the caller and the
callee.

## What is here

- `macros/` is the attribute. `src/lib.rs` is the second version, where the
  function is named at the call site. `src/lib_v1.rs.txt` is the first
  version, kept for reference, where each function got its own macro and the
  key was special.
- `proto/` holds stand-in types plus the three supporting pieces the
  attribute relies on: the `Select` trait, the shared `build!` macro, and
  the `__slot!` helper.
- `kit/` is a second crate offering annotated functions, to test use across
  a crate boundary.
- `app/` exercises it. `src/main.rs` compares the sugared form with plain
  calls. `src/bin/p01_everything.rs` covers renamed imports, calls by path,
  another crate, generics, closures, `?`, recursion and a lazy iterator.
  `src/bin/s01_short_form.rs` covers the per-function short form.
  `src/bin/perf.rs` is the pair of functions used for the machine code check.

Run with `cargo run -p app`, or `cargo run -p app --bin p01_everything`.

## How it works

1. The attribute removes its marker attributes from the parameters and
   inserts one local `macro_rules!` at the top of the function body. That
   helper knows the names of the function's own inherited parameters. It
   generates no run-time code.
2. Beside the function it generates a hidden module holding one function per
   default, and a `macro_rules!` with the same name as the function.
3. `build!(path, name: value, ...)` asks the caller's helper for the
   caller's inherited parameters, then hands over to the callee's macro.
4. The callee's macro keeps one slot per parameter, fills the slots from the
   named arguments one at a time, and emits a plain call to the function.
5. Each inherited parameter is written as `Select::select(&(a, b, ...))`.
   `Select` is implemented for tuples once per position, so the compiler
   picks the element whose type matches the parameter.

A function, a module and a macro can share a name because they live in
separate namespaces, and one `use` imports all three.

## Findings

- In an optimised build, a function written with the sugar and the same
  function written with plain calls were merged by the compiler into one
  function: identical machine code.
- 2,000 sugared calls with four named arguments added about 0.08 s to
  `cargo check`.
- rust-analyzer's diagnostics report no errors over this code. Completion
  and hover inside a call were not tested.
- Naming the function at the call site works under renamed imports, by
  path, from another crate, and for two functions that share a name.
- The per-function short form, `chip!(...)`, is one rule forwarding to the
  shared macro. It only works when the function is imported under its own
  name, because a macro cannot learn the name it was invoked under. Under
  `use w::chip as tag;`, `tag!(...)` fails with "cannot find macro `chip`".
- The attribute and the shared macro cannot have the same name, since both
  live in the macro namespace.
- A calling function must itself have every inherited parameter its callees
  need. Requirements propagate upward.
- Each annotated function generates a hidden module of the same name, which
  would clash with a real module of that name in the same place.
- Exported macros share one namespace per crate, so the hidden macro's name
  includes a hash of the function and its source location.

## Known gaps

- A default for a borrowed parameter, such as `#[default("none")] text: &str`,
  does not compile.
- A default on a parameter whose type is a generic parameter would not work.
- At most four inherited parameters per function.
- Two inherited parameters of the same type give an obscure compiler error.
- Calling `build!` on a function without the attribute reports "cannot find
  macro", which does not suggest the fix.
