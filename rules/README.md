# Rules

This directory contains the repository's rule books.

Use each book's `rules.md` for writing, reviewing, and enforcing rules.
Companion files under section directories carry the rationale and maintenance
provenance for individual rules. Use `CONTRIBUTING.md` when updating a rule
book's sources or precedence.

> The "Origins" note matters for future revisions: when one of the underlying
> guides changes, it shows which rules need to be rechecked together.

## Layout

- `rules/README.md`: how to use and maintain the rule books
- `rules/rule-books/`: rules for rule books and the structure under `rules/`
- `rules/rust-code/`: low-level Rust coding rules
- `rules/application-architecture/`: high-level application architecture rules

## Using the books

1. Read `<book>/rules.md` for the enforceable rule set.
2. The rationale of a rule, which is useful when reporting or understanding a
   violation, and its origin information, which is useful when maintaining the
   rule book, are both located in a companion file under the book directory.
   For a rule with the id `foo/bar-baz` in the `rust-code/` book, the
   companion file is `rust-code/foo/bar-baz.md`.
3. If you are updating a book's sources or precedence, start with the book's
   `CONTRIBUTING.md`, then update any affected companion `Origins:` notes.

## Structure of each book

Each book directory contains:

- `CONTRIBUTING.md`: source list and established precedence
- `rules.md`: enforceable rules only
- `<section-slug>/`: companion files for one section
- `<section-slug>/<rule-slug>.md`: rationale and `Origins:` for one rule

The companion path is derived directly from the full rule id. For example,
`imports/avoid-wildcard-imports` maps to
`imports/avoid-wildcard-imports.md`.
