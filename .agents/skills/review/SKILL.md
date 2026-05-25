---
name: review
description: >
  Use for all reviews: code reviews, architecture reviews, documentation reviews, etc.
---

# review

Rule books are defined under `rules/`.

1. Load `rules/README.md` to understand how to parse them.
2. Collect a list of all the available rule books.
3. For each rule book, read its frontmatter to determine whether it applies to what you are reviewing.
4. Thoroughly review what you were asked to review, strictly enforcing all the rules of all the applicable rule books to the review task.
5. Write a full review report in Markdown, with only the violations of the rules. It should include the exact location of the violation, which rule was violated, a clear explanation of the violation, and a suggestion for how to fix the issue.

**Markdown hygiene:** Escape `<` and `>` that appear outside of inline code or code fences as `\<` and `\>`, to prevent renderers from swallowing them as HTML tags.
