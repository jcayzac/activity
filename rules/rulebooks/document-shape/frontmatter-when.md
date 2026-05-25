# Use YAML frontmatter in `rules.md` to declare applicability

Rule id: `document-shape/frontmatter-when`

**Rationale:** The `when` block gives readers and tools a compact,
machine-readable statement of when a book should be consulted, without forcing
that information into the prose intro. Keeping the block at the top of
`rules.md` makes it easy to discover consistently across books, and using a
list keeps the shape stable when a book later gains more than one `when`
statement.
