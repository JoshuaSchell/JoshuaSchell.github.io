# joshuaschell.github.io

Personal site, built by a small Rust static site generator. No Node, no client-side framework.

```sh
cargo run                            # build into dist/
python3 -m http.server -d dist 8000  # preview at localhost:8000
```

Pushing to `main` builds and deploys to GitHub Pages (`.github/workflows/deploy.yml`).

## Editing content

- `content/index.md`: the bio on the home page.
- `content/site.toml`: name, links, work, projects, and papers.
- `content/blog/<slug>.md`: posts, published at `/blog/<slug>/`. There are none yet; while there are none,
  the blog page, feed, and nav are left out, and they appear with the first post. Each starts with TOML front matter:

  ```markdown
  +++
  title = "Post title"
  description = "One sentence for the blog index, feeds, and search results."
  date = 2026-10-16
  draft = true  # optional, hides the post
  +++
  ```

  Footnotes use standard Markdown and render as sidenotes, in the right margin on wide screens and
  tap-to-expand on narrow ones. The feed keeps them as ordinary footnotes:

  ```markdown
  Let's take vim as an example[^vim].

  [^vim]: This is just an example, and applies to other editors too.
  ```

  Math is LaTeX between `$...$` or `$$...$$`, rendered to MathML at build time. Fenced code blocks are
  highlighted at build time too. Pages only load `math.css` or `syntax.css` when they need them.

- `static/`: copied as-is into `dist/` (stylesheet, fonts, favicon).

## How it's built

- `src/main.rs`: loads content, writes pages, `sitemap.xml`, and `feed.xml`, and adds content hashes to
  asset URLs so browsers refetch them after a change.
- `src/markdown.rs`: Markdown via `pulldown-cmark`, math via `pulldown-latex`, code via `syntect`.
- `themes/`: GitHub Light and Dark code colors (VS Code theme JSON, converted at build time). Any VS Code
  theme's JSON works; change `LIGHT_THEME` / `DARK_THEME` in `src/markdown.rs`.
- `src/templates.rs`: HTML templates written with `maud`, checked at compile time.
