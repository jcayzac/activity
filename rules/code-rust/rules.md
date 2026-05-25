---
when:
  - Writing, modifying or reviewing Rust code of any kind.
---
# Rust Code Rules

This rule book contains the enforceable Rust coding rules for this repository. Use it for writing and reviewing code. Companion files under section directories in this book carry rationale and source provenance for individual rules.

---

## Organize Modules for Navigation

### `modules/use-mod-rs-directory-root`

Use `mod.rs` for the root of a directory module

**Rule:** If a module has submodules, put its root in `foo/mod.rs`. Do not keep
a split module as `foo.rs` plus `foo/`.

**Bad**

```text
src/
  stuff.rs
  stuff/
    stuff_files.rs
```

**Good**

```text
src/
  stuff/
    mod.rs
    stuff_files.rs
```

### `modules/directory-roots-preludes-toc`

Make directory roots and preludes tables of contents

**Rule:** `mod.rs`, `lib.rs`, and prelude modules should mostly contain `mod`
declarations and re-exports. Put real definitions in topically named child
files.

**Bad**

```rust
// src/parser/mod.rs
pub mod lexer;
pub mod syntax;

pub fn parse(input: &str) -> Ast {
    // substantial logic hidden in the module root
}
```

**Good**

```rust
// src/parser/mod.rs
mod parse;
pub mod lexer;
pub mod syntax;

pub use parse::parse;
```

### `modules/public-api-filesystem-mapping`

Keep the public API mappable to the filesystem

**Rule:** Keep module paths and file layout aligned. Avoid inline modules and
`#[path]` unless generated code or platform glue leaves no better option.

**Bad**

```rust
#[cfg(windows)]
#[path = "foo_windows.rs"]
mod foo;

#[cfg(unix)]
#[path = "foo_unix.rs"]
mod foo;
```

**Good**

```rust
#[cfg(windows)]
mod foo_windows;
#[cfg(windows)]
use foo_windows as foo;

#[cfg(unix)]
mod foo_unix;
#[cfg(unix)]
use foo_unix as foo;
```

## Keep Visibility and File Layout Simple

### `visibility-layout/simple-visibility-levels`

Use only simple visibility levels

**Rule:** Prefer private, `pub(crate)`, or `pub`. Avoid finer-grained restricted
visibility unless there is a strong, local reason.

### `visibility-layout/top-down-file-order`

Order files top-down for skimming

**Rule:** Order items so that a reader sees the main abstraction first, then how
to use it, then supporting details:

1. the central type or function
2. inherent impls
3. trait impls
4. public helpers and supporting items
5. private helpers

Within that structure, prefer public before private, types before impls,
inherent impls before trait impls, and callers before callees.

**Bad**

```rust
fn parse_token(...) -> Token { ... }
fn parse_expr(...) -> Expr { ... }

pub struct Parser { ... }

impl Display for Parser { ... }
impl Parser { ... }
```

**Good**

```rust
pub struct Parser { ... }

impl Parser { ... }

impl Display for Parser { ... }

fn parse_expr(...) -> Expr { ... }
fn parse_token(...) -> Token { ... }
```

## Keep Imports Local, Explicit, and Merge-Friendly

### `imports/separate-imports-reexports`

Separate private imports from public re-exports

**Rule:** Put private `use` items first. Put `pub use` re-exports after them,
separated by a blank line.

**Bad**

```rust
use rand::Rng as _;
pub use regex::Regex;
use serde::Deserialize as _;
```

**Good**

```rust
use rand::Rng as _;
use serde::Deserialize as _;

pub use regex::Regex;
```

### `imports/minimal-obvious-imports`

Import only what keeps the code obvious

**Rule:** Keep private imports minimal. Prefer one imported item per line. Use
anonymous trait imports for extension traits. Avoid large compound imports
unless they clearly improve the public surface.

**Bad**

```rust
use std::collections::{HashMap, hash_map::Entry};
use serde::Deserialize;
```

**Good**

```rust
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use serde::Deserialize as _;
```

### `imports/sort-imports-per-group`

Sort imports inside each import group

**Rule:** Keep imports near the top of the file. Within each contiguous import
group, sort imports version-wise. Keep `self` and `super` first when present.

### `imports/avoid-wildcard-imports`

Avoid wildcard imports in production code

**Rule:** Do not use wildcard imports in production code except where the glob
is itself the public API, such as a prelude. Prefer explicit imports in normal
modules.

**Bad**

```rust
use my_crate::prelude::*;
use std::collections::*;
```

**Good**

```rust
use my_crate::prelude::{Parse, Render};
use std::collections::HashMap;
```

## Use Predictable Rust Naming

### `naming/standard-rust-casing`

Follow standard Rust casing

**Rule:** Use:

- `snake_case` for modules, functions, methods, fields, locals, and macros
- `UpperCamelCase` for types, traits, and enum variants
- `SCREAMING_SNAKE_CASE` for constants and immutable statics

Treat acronyms as words: `Uuid`, `HttpClient`, `Stdin`, not `UUID`,
`HTTPClient`, or `StdIn`.

### `naming/standard-conversion-getter-iterator-naming`

Name conversions, getters, iterators, and constructors the standard way

**Rule:** 
- Use `as_` for cheap borrowed views.
- Use `to_` for work or owned conversion.
- Use `into_` for consuming conversion.
- Use `into_inner` to extract a wrapped inner value.
- Do not use `get_` for ordinary getters; use `name()` and `name_mut()`.
- Use `iter`, `iter_mut`, and `into_iter` for collection iteration.
- Name iterator types `Iter`, `IterMut`, and `IntoIter`.
- Put constructors on the type as inherent methods; use `new` for the primary
  constructor unless the domain clearly calls for `open`, `connect`, `bind`, and
  similar verbs.

**Bad**

```rust
impl Buffer {
    pub fn get_bytes(&self) -> &[u8] { ... }
    pub fn slice_mut(&mut self) -> &mut [u8] { ... }
}

impl Collection {
    pub fn values(&self) -> Iter<'_, Item> { ... }
}
```

**Good**

```rust
impl Buffer {
    pub fn as_bytes(&self) -> &[u8] { ... }
    pub fn as_mut_slice(&mut self) -> &mut [u8] { ... }
}

impl Collection {
    pub fn iter(&self) -> Iter<'_, Item> { ... }
}
```

## Put Meaning in Types, Not in Call Sites

### `type-design/prefer-methods-clear-receiver`

Prefer methods when the receiver is clear

**Rule:** If an operation is naturally "something you do with a `T`", make it a
method on `T`, not a free function that takes `&T`.

**Bad**

```rust
pub fn frob(foo: &Foo, widget: Widget) { ... }
```

**Good**

```rust
impl Foo {
    pub fn frob(&self, widget: Widget) { ... }
}
```

### `type-design/return-values-out-parameters`

Return values instead of out-parameters

**Rule:** Return values directly. Do not use out-parameters except when the API
is explicitly about mutating caller-owned storage or reusing a buffer.

**Bad**

```rust
fn split_at_midpoint(input: &[u8], left: &mut Vec<u8>, right: &mut Vec<u8>) { ... }
```

**Good**

```rust
fn split_at_midpoint(input: &[u8]) -> (Vec<u8>, Vec<u8>) { ... }
```

### `type-design/domain-types-over-bool-option-integers`

Do not encode meaning in `bool`, `Option`, or raw integers when a real type would do

**Rule:** Use newtypes, enums, or small structs to encode units, modes, flags,
and validated values. Use `bitflags` for combinable flags. Use builders when
construction has many options.

**Bad**

```rust
let widget = Widget::new(true, false);
```

**Good**

```rust
let widget = Widget::new(Size::Small, Shape::Round);
```

### `type-design/private-representations-passive-data`

Keep representations private unless the type is intentionally passive data

**Rule:** Default to private fields. Expose state through methods, smart
constructors, or builders. Seal traits that are not intended for downstream
implementation.

### `type-design/composition-over-inheritance-reuse`

Prefer composition over inheritance-style reuse

**Rule:** Prefer composing smaller structs and capabilities over using `Deref`,
macro expansion, or other indirection to emulate inheritance-style code reuse.

**Bad**

```rust
struct Foo { a: String }
struct Bar { inner: Foo, b: String }

impl std::ops::Deref for Bar {
    type Target = Foo;
    fn deref(&self) -> &Foo { &self.inner }
}
```

**Good**

```rust
struct FooPart { a: String }
struct BarPart { b: String }

struct Bar {
    foo: FooPart,
    bar: BarPart,
}
```

## Implement the Right Traits, and Implement Them Correctly

### `traits/implement-standard-interoperability-traits`

Eagerly implement standard interoperability traits when they are semantically correct

**Rule:** Public types should implement the common traits that obviously apply:
`Clone`, `Copy`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Debug`,
`Display`, and `Default`. Collection-like types should implement `FromIterator`
and `Extend`. Standard conversions should use `From`, `TryFrom`, `AsRef`, and
`AsMut`. Data-structure types that are reasonably serializable should support
Serde; if that support is optional, gate it behind a feature named `serde`.

### `traits/reader-writer-generics-by-value`

Reader/writer generics take ownership by value

**Rule:** Generic reader/writer APIs should take `R: Read` and `W: Write` by
value. Document that callers may pass `&mut` when they need to retain ownership.

### `traits/reexport-dependency-types-public-api`

Re-export dependency types that appear in your public API

**Rule:** If a public API exposes a type from a dependency, re-export that type
from your crate.

**Bad**

```rust
pub fn parse(input: &str) -> dep_lib::ParsedName { ... }
```

**Good**

```rust
pub use dep_lib::ParsedName;

pub fn parse(input: &str) -> ParsedName { ... }
```

### `traits/derive-standard-comparison-traits`

Derive standard comparison traits when structural semantics are correct

**Rule:** Prefer deriving `PartialEq`, `Eq`, `PartialOrd`, and `Ord` when
structural equality and lexicographic ordering are the intended behavior. If you
implement them manually, preserve the standard invariants and document why
derive is insufficient.

### `traits/avoid-manual-send-sync-deref`

Avoid manual `Send`, `Sync`, and `Deref` unless the type truly deserves them

**Rule:** Let the compiler derive `Send` and `Sync` when possible. Only
smart-pointer-like types should implement `Deref` or `DerefMut`. Avoid manual
`Send` and `Sync` impls unless there is no simpler design.

## Structure Functions as Readable Narratives

### `function-structure/visual-function-paragraphs`

Break functions into visual paragraphs

**Rule:** Use blank lines to separate groups of statements that serve different
purposes. Start each paragraph with the variable or condition that announces
what that paragraph is doing.

**Bad**

```rust
fn report_warning_count(&self, ...) {
    let gctx = runner.bcx.gctx;
    runner.compilation.lint_warning_count += count.lints;
    let mut message = descriptive_pkg_name(&unit.pkg.name(), &unit.target, &unit.mode);
    message.push_str(" generated ");
    gctx.shell().warn(message)
}
```

**Good**

```rust
fn report_warning_count(&self, ...) {
    let gctx = runner.bcx.gctx;

    runner.compilation.lint_warning_count += count.lints;

    let mut message = descriptive_pkg_name(&unit.pkg.name(), &unit.target, &unit.mode);
    message.push_str(" generated ");

    gctx.shell().warn(message)
}
```

### `function-structure/visible-business-control-flow`

Keep business logic in visible control-flow blocks

**Rule:** Use `if`/`else`, `match`, and explicit blocks to make mutually
exclusive business cases easy to compare. Use early returns mostly for guards,
validation, and bookkeeping.

**Bad**

```rust
if let Some(foo) = foo {
    if case {
        return Ok(a);
    }

    Ok(b)
} else {
    Err(err)
}
```

**Good**

```rust
let foo = foo.ok_or(err)?;

if case {
    Ok(a)
} else {
    Ok(b)
}
```

### `function-structure/side-effects-outside-pipelines`

Do not hide side effects inside expression pipelines

**Rule:** Keep combinators pure with respect to business logic. If a step
mutates state or performs an effect, use statements or a `for` loop instead of
burying that behavior inside `map`, `for_each`, or related combinators.

**Bad**

```rust
let mut seen = false;
let other = list
    .map(|item| {
        seen = true;
        transform(item)
    })
    .collect::<Vec<_>>();
```

**Good**

```rust
let mut seen = false;
let mut other = Vec::new();

for item in list {
    seen = true;
    other.push(transform(item));
}
```

### `function-structure/iterator-transforms-pure-data`

Prefer iterator transforms for pure data transformations

**Rule:** When code is primarily transforming, filtering, or aggregating data
without side effects, prefer iterator transforms and combinators over manual
loop scaffolding.

**Bad**

```rust
let mut out = Vec::new();
for item in items {
    if keep(&item) {
        out.push(transform(item));
    }
}
```

**Good**

```rust
let out: Vec<_> = items
    .into_iter()
    .filter(keep)
    .map(transform)
    .collect();
```

### `function-structure/avoid-clone-borrow-checker`

Do not clone just to satisfy the borrow checker

**Rule:** Treat `.clone()` used only to silence a borrow-checker error as an
anti-pattern. Prefer restructuring ownership or borrowing, shortening the
borrow's scope, or using `mem::take`, `mem::replace`, or `Option::take` where
they express the actual move.

**Bad**

```rust
let mut x = String::from("hi");
let y = &mut x.clone();
println!("{x}");
y.push('!');
```

**Good**

```rust
let mut x = String::from("hi");
{
    let y = &mut x;
    y.push('!');
}
println!("{x}");
```

## Prefer `Result` to Panic, and Be Specific About Failure

### `errors/result-not-panic`

Normal failure is `Result`, not panic

**Rule:** Use `Result` for ordinary fallible behavior. Library code must not
panic for invalid input, parse errors, I/O failures, or externally supplied
data. Reserve `panic!`, `unwrap`, `expect`, and `assert!` for violated internal
invariants, impossible states, tests, or explicitly documented contracts.

### `errors/real-public-error-types`

Public library errors must have real types

**Rule:** Never use `()` as an error type. Public library APIs should return
crate-specific error types that implement
`std::error::Error + Send + Sync + 'static` and `Display`. Do not expose
type-erased wrappers such as `anyhow::Error` in library APIs.

**Bad**

```rust
pub fn parse(input: &str) -> anyhow::Result<Value> { ... }
```

**Good**

```rust
#[derive(Debug)]
pub struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid input")
    }
}

impl std::error::Error for ParseError {}

pub fn parse(input: &str) -> Result<Value, ParseError> { ... }
```

### `errors/validate-at-boundary-type-system`

Validate at the boundary, preferably in the type system

**Rule:** Prefer types that make bad states unrepresentable. When runtime
validation is still needed, do it at the boundary and make the failure mode
explicit. Provide `_unchecked` escape hatches only when the performance case is
real and the contract is documented.

### `errors/explicit-integer-overflow-behavior`

Make integer overflow behavior explicit

**Rule:** If integer overflow is possible and part of the behavior, use
`checked_*`, `overflowing_*`, `wrapping_*`, `saturating_*`, or the `Wrapping<T>`
/ `Saturating<T>` wrapper types. Do not rely on profile-dependent overflow
behavior.

## Make Destructors Boring

### `destructors/infallible-nonblocking-destructors`

Destructors must not fail, panic, or block

**Rule:** `Drop` implementations must be infallible, non-panicking, and
non-blocking. If teardown can fail or block, expose an explicit method such as
`close`, `finish`, or `flush` and keep `Drop` best-effort only.

### `destructors/raii-guards-scoped-access`

Prefer RAII guards for scoped access to resources or invariants

**Rule:** When access to a resource or temporary invariant must be paired with
reliable teardown, expose that access through a guard object whose `Drop`
restores or releases it at scope exit.

**Bad**

```rust
resource.lock();
do_work(&resource);
resource.unlock();
```

**Good**

```rust
let guard = resource.lock();
do_work(&guard);
```

## Document Public APIs for Use, Failure, and Safety

### `api-documentation/document-public-items-purpose`

Document every public item and show why it exists

**Rule:** Every public crate, module, trait, struct, enum, function, method,
macro, and type alias should have rustdoc. Examples should show why someone
would want the API, not just the mechanics of calling it.

### `api-documentation/copy-safe-examples`

Make examples copy-safe

**Rule:** In docs, prefer examples that use `?` and return `Result`. Do not use
`unwrap` in examples unless the point of the example is the panic path itself.

### `api-documentation/document-errors-panics-safety`

Document `Errors`, `Panics`, and `Safety`

**Rule:** Public fallible APIs need an `# Errors` section. Public APIs that can
panic need an `# Panics` section. Unsafe functions and unsafe contracts need an
explicit `# Safety` section listing caller obligations.

### `api-documentation/link-related-items-rustdoc`

Link related items in rustdoc prose

**Rule:** In doc comments, link to related types, modules, and methods instead
of leaving names as unlinked prose.

## Let `rustfmt` Own Mechanical Formatting

### `formatting/default-rust-formatting`

Use default Rust formatting

**Rule:** Format code with `rustfmt`. In handwritten code, still write toward
the default Rust style: 4-space indentation, 100-column lines, block
indentation, trailing commas in multiline lists, one blank line at most between
adjacent items or statement groups, and one attribute per line.

**Bad**

```rust
a_function_call(foo,
                bar)
```

**Good**

```rust
a_function_call(
    foo,
    bar,
)
```

### `formatting/scannable-signatures-type-layouts`

Keep signatures and type layouts easy to scan

**Rule:** When a signature, struct, enum variant, tuple, or type does not fit on
one line, break at the outermost syntactic boundary and put one logical element
per line. Prefer named structs over wide tuple structs once field meaning
matters.

**Bad**

```rust
fn build(config: Config, input: Input, options: Options, cache: &mut Cache, state: &mut State) -> Result<Output, Error> { ... }
```

**Good**

```rust
fn build(
    config: Config,
    input: Input,
    options: Options,
    cache: &mut Cache,
    state: &mut State,
) -> Result<Output, Error> {
    ...
}
```

## Keep Unsafe Code Rare, Small, and Auditable

### `unsafe/prefer-safe-rust-forbid-unneeded-unsafe`

Prefer safe Rust, and forbid `unsafe` when you do not need it

**Rule:** If a crate does not need `unsafe`, forbid it at the crate root. If
`unsafe` is necessary, make the block small, local, and justified.

### `unsafe/unsafe-internals-safe-apis-explicit-contracts`

Encapsulate unsafe internals behind safe APIs or explicit unsafe contracts

**Rule:** Safe APIs that use `unsafe` internally must fully preserve Rust's
safety guarantees. If the caller must uphold an invariant, make the API `unsafe`
and document every obligation.

## Treat Memory Management as an API Contract

### `memory/avoid-convenience-memory-resource-leaks`

Do not leak memory or resources as a convenience

**Rule:** Do not use `mem::forget`, `Box::leak`, deprecated
`mem::uninitialized`, or ad hoc raw-pointer ownership tricks unless the
ownership transfer is deliberate, documented, and unavoidable.

### `memory/exact-raw-ownership-round-trips`

Make raw ownership round-trips exact

**Rule:** If you turn an owned value into a raw pointer with `into_raw`, make
the matching `from_raw` responsibility explicit. Only call `from_raw` on
pointers produced by the matching `into_raw`.

### `memory/no-drop-security-cleanup`

Do not rely on `Drop` for security-sensitive cleanup

**Rule:** Sensitive cleanup such as wiping secrets must not rely solely on
`Drop`. Avoid reference-count cycles for types with `Drop` or scarce resources.

## Keep FFI Thin, Typed, and Defensive

### `ffi-boundaries/ffi-raw-bindings-safe-wrappers`

Split FFI into raw bindings and safe wrappers

**Rule:** Structure FFI as a low-level `extern` binding layer plus a safe Rust
wrapper layer. If the raw layer is public, put it in a dedicated `*-sys` crate.

### `ffi-boundaries/object-based-ffi-wrappers`

Prefer object-based wrapper APIs at FFI boundaries

**Rule:** When exporting Rust functionality over FFI, prefer opaque object-based
APIs that route operations through wrapper types rather than exposing multiple
related raw structs and handles directly.

### `ffi-boundaries/generated-over-handwritten-ffi-bindings`

Prefer generated FFI bindings over handwritten ones where practical

**Rule:** Prefer generated low-level FFI bindings over handwritten bindings when
reliable generators exist for the boundary, and review the generated layer
rather than treating generation as a substitute for review.

### `ffi-boundaries/c-compatible-ffi-types`

Use only C-compatible types at the boundary

**Rule:** Use `repr(C)` or `repr(transparent)` deliberately. Use portable `c_*`
aliases for platform-dependent C types. Do not accept non-C-compatible Rust
types directly across the FFI boundary unless they are intentionally opaque.

### `ffi-boundaries/validate-foreign-values`

Treat foreign values as untrusted until checked

**Rule:** Check foreign pointers before dereferencing them. Prefer raw pointers
for foreign-owned pointer-like inputs. Do not accept incoming foreign Rust enums
directly; accept integers and validate them. Mark FFI function pointer types
with the correct `extern` ABI and `unsafe`.

### `ffi-boundaries/no-panics-across-ffi`

Do not let panics cross the FFI boundary

**Rule:** Rust code callable from foreign code must not unwind through foreign
frames. Prevent panics or catch unwinds at the boundary.

## Use Stable Tooling, Reproducible Builds, and Measured Performance

### `build-performance/stable-toolchain`

Use a stable toolchain for normal development

**Rule:** Use a stable toolchain by default. If a nightly tool is needed, invoke
it locally and explicitly; do not make nightly the project's normal toolchain.

### `build-performance/explicit-checked-in-build-inputs`

Keep build inputs explicit and checked in

**Rule:** Track `Cargo.lock` in version control. Do not rely on ad hoc `RUSTC`,
`RUSTC_WRAPPER`, `RUSTFLAGS`, or profile overrides to define normal project
behavior.

### `build-performance/intentional-dependency-graph`

Keep the dependency graph intentional

**Rule:** Review and manage the dependency graph explicitly. Avoid unnecessary
dependencies, and periodically inspect the graph for bloat, duplicated major
versions, and surprising transitive pulls.

### `build-performance/formatter-linter-dependency-audits`

Run the formatter, the linter, and dependency audits regularly

**Rule:** Run `rustfmt` and `clippy` regularly. Review automatic fixes before
accepting them. Check dependencies for staleness and known vulnerabilities.
Track direct dependency validation.

### `build-performance/small-focused-crates-modules`

Prefer small crates and modules with focused responsibilities

**Rule:** Prefer smaller crates and modules that do one thing well. Split code
when doing so produces clearer ownership, clearer APIs, or clearer build
boundaries, but do not split purely for its own sake.

### `build-performance/more-than-unit-tests`

Write more than unit tests

**Rule:** Use unit tests for internal logic, integration tests for public API
behavior, and doc tests for documented usage. Add examples, benchmarks, or fuzz
tests when the crate's behavior or risk profile warrants them.

### `build-performance/ci-quality-gates`

Run CI on the quality gates that define "done"

**Rule:** CI should run the checks that define merge readiness for the project,
including formatting, linting, tests, and documentation, plus any project-
specific security, compatibility, or performance gates.

### `build-performance/test-supported-dependency-ranges`

Test against supported dependency ranges when compatibility matters

**Rule:** If a library promises compatibility across a dependency version range,
test against more than one point in that range, especially the minimum supported
version and the current version.

### `build-performance/benchmark-track-performance`

Benchmark and track performance when performance matters

**Rule:** If performance is a stated requirement, add benchmarks with realistic
workloads and track them over time.

### `build-performance/profile-before-optimizing-hot-path`

Profile before you optimize, and optimize the hot path first

**Rule:** Do not optimize code just because it looks slow. Profile first, then
optimize the hot code. Prefer algorithm, data-structure, allocation, and call-
frequency improvements before low-level tweaks.

### `build-performance/document-measured-optimizations`

Document non-obvious optimizations with the measurement that justified them

**Rule:** If an optimization makes the code structure non-obvious, add a short
comment explaining the measured behavior or workload shape that justified it.

### `build-performance/measured-hasher-changes`

Do not change hashers without a measured reason

**Rule:** Keep the default hashing strategy unless profiling shows hashing is a
hot cost and the security trade-off is acceptable for the workload.

### `build-performance/measured-inline-attributes`

Use inline attributes sparingly and only after measurement

**Rule:** Do not add `#[inline]` or `#[inline(always)]` preemptively. Use inline
attributes only for measured hot paths where they improve overall results.

### `build-performance/avoid-unnecessary-logging-io`

Avoid unnecessary logging and I/O work in non-hot paths

**Rule:** Keep logging and I/O code from doing expensive work unnecessarily,
especially when that work is discarded in common configurations. Avoid needless
allocation and formatting in such paths.

### `build-performance/predictable-memory-access`

Keep memory access patterns predictable in hot code

**Rule:** In latency-sensitive or throughput-critical code, prefer data access
patterns that are sequential, local, and easy for the hardware prefetcher to
predict. Avoid pointer-chasing and scattered access when a flatter or more
contiguous layout would do.

### `build-performance/avoid-false-sharing`

Avoid false sharing in hot mutable state

**Rule:** Keep independently updated hot fields from sharing a cache line across
threads. Separate such state structurally or with alignment/padding when
measurement shows cache-line contention.

### `build-performance/single-writer-hot-state`

Prefer single-writer ownership for mutable hot state

**Rule:** For mutable state on hot concurrent paths, prefer designs with one
clear writer and explicit handoff or message passing over designs with many
contending writers.

### `build-performance/batch-expensive-boundaries`

Batch work when the boundary cost dominates

**Rule:** When crossing an expensive boundary such as a lock, channel, syscall,
network send, storage write, or FFI call, prefer natural batching over many tiny
operations when latency requirements still allow it.

## Design Data Boundaries and State for Evolution

### `data-boundaries/explicit-evolvable-data-formats`

Make wire and stored data formats explicit, schema-driven, and evolvable

**Rule:** Treat any wire format, message format, on-disk format, or durable
cache format as an explicit contract. Define its schema deliberately, version it
when needed, and do not treat a Rust in-memory representation as the format by
accident.

### `data-boundaries/compatible-data-contract-evolution`

Require backward- and forward-compatible evolution for durable or cross-service data contracts

**Rule:** If data crosses service boundaries or is stored durably, evolve the
schema compatibly unless a deliberate migration plan says otherwise. Document
which compatibility guarantees the format provides.

### `data-boundaries/layout-match-access-patterns-workload`

Choose storage and in-memory layout to match access patterns and workload

**Rule:** Choose storage structures and in-memory layouts based on the access
pattern they serve. Do not force one representation to handle incompatible read,
write, scan, and update workloads when separate representations are clearer.

### `data-boundaries/authoritative-vs-derived-state`

Keep authoritative state distinct from derived state

**Rule:** Make the source of truth explicit. Treat caches, indexes,
materializations, denormalized views, and other projections as derived state,
and design them to be recomputable or repairable where practical.

### `data-boundaries/explicit-boundary-guarantees`

Make consistency, ordering, and durability guarantees explicit at boundaries

**Rule:** For networked, persistent, or multi-process components, document the
ordering, consistency, idempotency, and durability guarantees each boundary
provides. Do not rely on implicit expectations.

### `data-boundaries/monotonic-time-durations-timeouts`

Use monotonic time for durations and timeouts

**Rule:** Measure elapsed time, deadlines, retries, and timeouts with monotonic
time sources. Use wall-clock time only for timestamps meant to describe real-
world time to humans or external systems.

## Keep `Cargo.toml` Complete and Predictable

### `cargo-manifest/simple-stable-manifest-formatting`

Keep manifest formatting simple and stable

**Rule:** Put `[package]` first. Put `name` and `version` first inside it. Use
bare keys when possible, one space around `=`, and multiline arrays with
trailing commas when they do not fit on one line. Version-sort keys within
sections unless a stronger convention applies.

### `cargo-manifest/package-metadata-release-notes`

Fill in standard package metadata and keep release notes

**Rule:** Provide `description`, `license`, `repository`, `keywords`, and
`categories`, plus non-redundant `homepage` or `documentation` when needed. Use
a valid SPDX expression for `license`. Keep release notes for significant
changes and tag published releases.

### `cargo-manifest/deliberate-features`

Be deliberate about features

**Rule:** Keep Cargo features narrow, orthogonal, and documented. Do not add a
feature unless it creates a meaningful build, platform, or integration boundary.
Avoid features that merely paper over API indecision or create an unbounded
configuration matrix.

## Prefer Functions and Types Over Macros, but Make Macros Feel Native

### `macro-style/rust-like-macros`

If you need a macro, make it read like Rust

**Rule:** Prefer ordinary functions, types, and traits over macros. When a macro
is justified, make its input syntax resemble the Rust syntax it expands to. Item
macros should support attributes, visibility, and function-scope use.

**Bad**

```rust
bitflags! {
    flags S: u32 {
        const A = 0b0001,
        const B = 0b0010,
    }
}
```

**Good**

```rust
bitflags! {
    pub struct S: u32 {
        const A = 0b0001;
        const B = 0b0010;
    }
}
```

## Meta rules

### `meta/principle-least-surprise`

Follow the principle of least surprise

**Rule:** If code can be rewritten in multiple ways to follow the other rules in
this rule book, and all satisfy the compiler, choose the one that is clearer,
lighter cognitively, and easier to evolve.
