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
- `rules/<book>/`: one rule book per direct child directory under `rules/`
- `rules/rulebooks/`: the rule book that governs the structure and maintenance
  of rule books themselves

List the direct child directories under `rules/` to discover the available
books, then read each book's `rules.md` frontmatter to see when it applies.

## Using the books

1. Read the YAML frontmatter at the top of `<book>/rules.md` to see when the
   book applies. The `when` key is a YAML list of short applicability
   statements.
2. Read the rest of `<book>/rules.md` for the enforceable rule set.
3. The rationale of a rule, which is useful when reporting or understanding a
   violation, and its origin information, which is useful when maintaining the
   rule book, are both located in a companion file under the book directory.
   For a rule with the id `foo/bar-baz` in the `code-rust/` book, the
   companion file is `code-rust/foo/bar-baz.md`.
4. If you are updating a book's sources or precedence, start with the book's
   `CONTRIBUTING.md`, then update any affected companion `Origins:` notes.

## Structure of each book

Each book directory contains:

- `CONTRIBUTING.md`: source list and established precedence
- `rules.md`: YAML frontmatter with a `when` list, then the enforceable rules
- `<section-slug>/`: companion files for one section
- `<section-slug>/<rule-slug>.md`: rationale and `Origins:` for one rule

The companion path is derived directly from the full rule id. For example,
`imports/avoid-wildcard-imports` maps to
`imports/avoid-wildcard-imports.md`.
