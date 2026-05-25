# Rule Book Rules

This rule book contains the enforceable structure and authoring rules for the
material under `rules/`. Use it when creating or editing rule books. Companion
files under section directories carry rationale and maintenance provenance for
individual rules.

---

## Scope and Precedence

### `scope/guide`

Declare each book's scope and audience in `rules.md`

**Rule:** Start each book's `rules.md` with a title and a short intro that says
what the book governs, who should use it, and that companion files hold
rationale and source provenance for individual rules. Do not repeat
`CONTRIBUTING.md` workflow or other repository-level maintenance instructions
there.

### `scope/source-precedence`

Declare source precedence in `CONTRIBUTING.md`

**Rule:** If a rule book merges multiple sources, declare the precedence order
near the top of `CONTRIBUTING.md`. Use a sentence that makes the order explicit
and says that lower-precedence sources only fill gaps left by higher-precedence
ones.

**Good**

```md
When sources disagree, precedence is: [A], [B], [C]. Lower-precedence sources
are used only to fill gaps left by higher-precedence ones.
```

### `scope/final-conflict-resolution`

Resolve source conflicts into one rule

**Rule:** The final rule book must reconcile source conflicts into one usable
rule. Do not make the reader apply precedence themselves at review time. When
sources conflict, the higher-precedence source wins. When sources partially
align, write one merged rule that keeps the higher-precedence constraint and
folds in compatible lower-precedence guidance.

### `scope/origins-conflicts`

Explain conflict resolution inside companion `Origins`

**Rule:** If a rule required reconciliation, explain that naturally inside the
companion file's `Origins:` note. Do not add a separate `Conflict` field. Do
not repeat the rule text inside `Origins:`; explain only the source
relationship and the resolution.

**Bad**

```md
  **Origins:** `X` [A]; `Y` [B].

  **Conflict:** A is stricter, so this rule follows A.
```

**Good**

```md
  **Origins:** `X` [A]; `Y` [B]. These sources diverge here; this book follows A
  because it has higher precedence and keeps B only where it is compatible.
```

## Document Set

### `document-shape/shell`

Use the same book-level layout

**Rule:** Each rule book lives in its own directory under `rules/` and contains
exactly these canonical files:

1. `CONTRIBUTING.md`
2. `rules.md`
3. one companion subdirectory per section slug that has companion files
4. one companion markdown file per rule

**Good**

```text
rules/
  rust-code/
    CONTRIBUTING.md
    rules.md
    imports/
      avoid-wildcard-imports.md
    errors/
      real-public-error-types.md
```

### `document-shape/readme-role`

Use `rules/README.md` to explain the whole rules tree

**Rule:** `rules/README.md` must explain how to use the rule books for
enforcement, when to consult companion files, how to maintain sources and
precedence, how companion paths are derived, and which book directories exist
under `rules/`.

### `document-shape/intro-blockquote`

Keep the maintenance blockquote in `rules/README.md`

**Rule:** Include this exact blockquote near the top of `rules/README.md`:

```md
> The "Origins" note matters for future revisions: when one of the underlying
> guides changes, it shows which rules need to be rechecked together.
```

### `document-shape/contributing-role`

Record sources and precedence in `CONTRIBUTING.md`

**Rule:** Each book's `CONTRIBUTING.md` must list the established precedence and
the source documents in precedence order. If the book is original to the
repository, say that no external merged sources currently apply.

### `document-shape/main-content-delimiters`

Use `---` only for stable document boundaries

**Rule:** Put one `---` between the intro of `rules.md` and the first section
heading. Use `---` again only when introducing a local reference footer in a
companion file.

### `document-shape/reference-footer-comment`

Use an HTML-comment-titled footer for local references

**Rule:** If a companion file includes local source references, end it with:

```md
---

<!-- References -->

[ID]: https://example.com/source "Source Title"
```

Use one local reference definition per cited source.

### `document-shape/heading-format`

Use plain section titles and slugged rule headings in `rules.md`

**Rule:** Do not number sections or rules. Section headings are plain `##`
titles with no slug. Rule headings use the exact form:

```md
  ### `section-slug/rule-slug`

  Rule title
```

Then place the rule body below the title.

### `document-shape/field-order`

Split rule fields between `rules.md` and companion files

**Rule:** In `rules.md`, each rule should present its content in this order:

1. title line
2. `**Rule:**`
3. `**Bad**` and `**Good**` examples where they materially help

In the companion file, present content in this order:

1. `Rule id:`
2. `**Rationale:**`
3. `**Origins:**` when the rule is not original to the book; otherwise omit the
   field entirely

If a source has good and bad examples that materially teach the rule, keep them
in `rules.md`, not in the companion file.

### `document-shape/sidecar-files`

Keep one companion file per rule

**Rule:** Every rule heading in `rules.md` must have exactly one companion
markdown file at `<section-slug>/<rule-slug>.md`, and no companion file may
exist without a matching rule heading.

### `document-shape/sidecar-filenames`

Derive companion paths from the full rule id

**Rule:** Name each companion file by using the section slug as a directory name
and the rule slug as the filename with a `.md` suffix.

**Bad**

```text
avoid-wildcard-imports.md
imports--avoid-wildcard-imports
rule-17.md
```

**Good**

```text
imports/avoid-wildcard-imports.md
errors/real-public-error-types.md
meta/clear-stable-formulation.md
```

## Domain Sections

### `domain-sections/grouping`

Group rules by domain, not by source

**Rule:** Organize rules into domain sections that match how a reviewer or
writer looks for guidance: imports, naming, error handling, unsafe code, FFI,
builds, performance, data boundaries, and similar topics. Do not group rules by
source document.

### `domain-sections/title-style`

Use clear, direct, memorable section titles

**Rule:** A section title should be clear first, but it does not have to be
dryly taxonomic. Prefer titles that are concise, direct, and easy to remember.
A title may be verbal, rhetorical, or slightly catchy if that improves recall
without obscuring the domain it names.

**Bad**

```text
Miscellaneous Concerns
Various Things About Types
Stuff That Can Go Wrong
```

**Good**

```text
Put Meaning in Types, Not in Call Sites
Make Destructors Boring
Keep FFI Thin, Typed, and Defensive
```

### `domain-sections/cohesion`

Make sections broad enough to scan and narrow enough to cohere

**Rule:** Each section should represent one clear domain. Split a section when
it mixes unrelated concerns. Merge sections when a split would create tiny
fragments that readers would not look up separately.

### `domain-sections/refinement-proximity`

Keep refinements next to the rule they refine

**Rule:** If one rule is a refinement, carve-out, or more specific case of
another rule, place it immediately after the parent rule in the same section.
Keep the conceptual relationship obvious from proximity, even though the book
does not use numeric subrule notation.

### `domain-sections/meta-placement`

Reserve the last section for meta rules

**Rule:** Put cross-cutting interpretive rules at the end under the section
title `Meta rules`. Use the section slug `meta` for rules in that section.

## Stable Slugs

### `slugs/nominal-form`

Use nominal slugs

**Rule:** Section slugs and rule slugs should name the domain or concept being
indexed, not the action or wording of the sentence. Prefer noun phrases over
verb phrases.

**Bad**

```text
resolve-conflicts
use-standard-fields
preserve-intro-blockquote
make-sections-broad-cohesive
```

**Good**

```text
origins-conflicts
field-order
intro-blockquote
section-cohesion
```

### `slugs/section`

Derive section slugs from the section domain

**Rule:** A section slug must:

- use lowercase alphanumerics and hyphens only
- use at most two words
- be at most 20 characters long
- drop words that do not contribute meaning
- avoid opaque abbreviations
- stay unique across the document

Use the smallest clear domain name. Keep entrenched technical terms when they
are the clearest name of the domain, such as `api`, `ffi`, or `cargo`.

**Bad**

```text
buildperf
docs
things-about-errors
organize-modules
keep-imports-local
```

**Good**

```text
build-performance
api-documentation
error-handling
modules
imports
```

### `slugs/rule`

Derive rule slugs from the rule's stable concept

**Rule:** A rule slug must:

- use lowercase alphanumerics and hyphens only
- omit conjunctions, pronouns, articles, and other non-contributing words
- have no word-count limit
- be at most 45 characters long
- stay unique within its section

Base the slug on the rule's stable concept, not on every word of the visible
title. Name the distinguishing concept inside the section's domain, and prefer
concept words that are likely to survive future title edits.

**Bad**

```text
do-not-hide-side-effects-inside-expression-pipelines
the-public-api-errors-must-have-real-types
declare-source-precedence
prefer-single-writer-ownership-for-mutable-hot-state
```

**Good**

```text
side-effects-outside-pipelines
real-public-error-types
source-precedence
single-writer-hot-state
```

### `slugs/redundant-section-words`

Avoid repeating section words in the rule slug unless needed

**Rule:** Because the rule heading already contains `section-slug/rule-slug`,
do not repeat section terms inside the rule slug unless dropping them would make
the slug unclear or ambiguous.

**Bad**

```text
error-handling/error-handling-real-public-error-types
imports/imports-avoid-wildcard-imports
```

**Good**

```text
error-handling/real-public-error-types
imports/avoid-wildcard-imports
```

### `slugs/slug-stability`

Keep slugs stable across small editorial edits

**Rule:** Do not rename a section slug or rule slug just because the visible
title was polished. Rename a slug only when the underlying domain or rule
concept changed enough that the old slug became misleading.

## Maintenance

### `maintenance/material-source-citation`

Cite the sources that materially support each rule

**Rule:** Cite every source that materially contributes to the final rule in the
companion file's `Origins:` note, including lower-precedence sources that
reinforce an already-covered rule. Do not add citations that merely restate the
obvious or add no real support.

### `maintenance/origins-audit-trail`

Keep companion `Origins` audit-ready

**Rule:** `Origins:` should name the relevant source rule identifiers and source
documents when they exist, and briefly explain reconciliation when needed. It
should not restate the rule, add standalone maintenance chatter, or drift into
mini-essays.

### `maintenance/original-origins-omission`

Omit `Origins` for rules that are original to the book

**Rule:** If a rule is original to the book itself and does not derive from an
external source, omit the companion file's `Origins:` field entirely. Do not
write `**Origins:** This book.`

### `maintenance/new-source-reconciliation`

Update books by reconciling, not appending

**Rule:** When adding a new source, first assign its precedence in
`CONTRIBUTING.md`. Then:

1. update the source list there
2. decide which existing companion `Origins:` notes it strengthens
3. add new rules only where the higher-precedence sources leave a real gap
4. rewrite any affected rules so the final book still reads as one book

Do not append a source as a disconnected appendix unless that is the explicit
goal of the book.

## Meta rules

### `meta/clear-stable-formulation`

Choose the clearest stable formulation

**Rule:** If multiple document structures or phrasings satisfy the rules above,
choose the clearest one that also keeps the book stable under later edits.
