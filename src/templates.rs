//! Page templates. maud checks the markup at compile time and escapes interpolated text.

use maud::{DOCTYPE, Markup, PreEscaped, html};
use serde_json::json;

use crate::{Assets, Post, Site};

/// Sets the theme before first paint so dark mode never flashes.
const THEME_INIT: &str = "document.documentElement.classList.toggle('dark',(localStorage.getItem('theme')||(matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light'))==='dark')";
const THEME_TOGGLE: &str = "document.getElementById('theme-toggle').onclick=()=>localStorage.setItem('theme',document.documentElement.classList.toggle('dark')?'dark':'light')";

/// Sun and moon icons from Lucide (ISC license). CSS shows the moon in light mode and the sun in dark mode.
const THEME_ICONS: &str = concat!(
    r#"<svg class="icon-moon" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/></svg>"#,
    r#"<svg class="icon-sun" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4"/>"#,
    r#"<path d="M12 2v2M12 20v2m-7.07-17.07 1.41 1.41m11.32 11.32 1.41 1.41M2 12h2m16 0h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/></svg>"#,
);

struct Page<'a> {
    title: String,
    description: &'a str,
    /// Absolute path of the page, e.g. `/blog/` or `/blog/some-post/`.
    path: &'a str,
    stylesheets: Vec<&'a str>,
    json_ld: Vec<serde_json::Value>,
    article: Option<&'a Post>,
    noindex: bool,
}

impl<'a> Page<'a> {
    fn new(site: &Site, title: Option<&str>, description: &'a str, path: &'a str) -> Self {
        Self {
            title: title.map_or_else(|| site.name.clone(), |t| format!("{t} | {}", site.name)),
            description,
            path,
            stylesheets: Vec::new(),
            json_ld: Vec::new(),
            article: None,
            noindex: false,
        }
    }
}

fn layout(site: &Site, assets: &Assets, page: Page, content: Markup) -> Markup {
    let canonical = format!("{}{}", site.url, page.path);
    let og_image = format!("{}/og-image.png", site.url);
    let current = |prefix: &str| {
        let active = if prefix == "/" {
            page.path == "/"
        } else {
            page.path.starts_with(prefix)
        };
        active.then_some("page")
    };
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (page.title) }
                meta name="description" content=(page.description);
                link rel="canonical" href=(canonical);
                @if page.noindex { meta name="robots" content="noindex"; }
                link rel="icon" type="image/svg+xml" href="/favicon.svg";
                @if site.has_blog {
                    link rel="alternate" type="application/atom+xml" title=(site.name) href="/feed.xml";
                }
                link rel="preload" href="/fonts/geist.woff2" as="font" type="font/woff2" crossorigin;
                link rel="stylesheet" href=(assets.style);
                @for href in &page.stylesheets { link rel="stylesheet" href=(href); }

                meta property="og:type" content=(if page.article.is_some() { "article" } else { "website" });
                meta property="og:url" content=(canonical);
                meta property="og:title" content=(page.title);
                meta property="og:description" content=(page.description);
                meta property="og:image" content=(og_image);
                meta property="og:site_name" content=(site.name);
                @if let Some(post) = page.article {
                    meta property="article:published_time" content=(post.date.iso());
                }
                meta name="twitter:card" content="summary_large_image";

                @for data in &page.json_ld {
                    script type="application/ld+json" { (PreEscaped(json_for_script(data))) }
                }
                script { (PreEscaped(THEME_INIT)) }
            }
            body {
                // With a blog there's a nav bar for the toggle to sit in; otherwise it sits beside the
                // page title (see `title_row`).
                @if site.has_blog {
                    header.site-header.wrap {
                        nav {
                            a href="/" aria-current=[current("/")] { "home" }
                            a href="/blog/" aria-current=[current("/blog/")] { "blog" }
                        }
                        (theme_button())
                    }
                }
                main.wrap { (content) }
                script { (PreEscaped(THEME_TOGGLE)) }
            }
        }
    }
}

fn theme_button() -> Markup {
    html! {
        button #theme-toggle.theme-toggle type="button" aria-label="Toggle dark mode" { (PreEscaped(THEME_ICONS)) }
    }
}

/// The page's `h1`, with the theme toggle beside it when there's no header bar to hold it.
fn title_row(site: &Site, title: &str) -> Markup {
    html! {
        div.title-row {
            h1.page-title { (title) }
            @if !site.has_blog { (theme_button()) }
        }
    }
}

/// JSON inside a `<script>` must not contain `</`, or it would close the tag early.
fn json_for_script(value: &serde_json::Value) -> String {
    value.to_string().replace("</", "<\\/")
}

/// `title ········ date` row.
fn entry_head(title: Markup, meta: Option<&str>) -> Markup {
    html! {
        div.entry-head {
            span { (title) }
            @if let Some(meta) = meta { span.meta { (meta) } }
        }
    }
}

fn link_or_text(text: &str, url: Option<&str>) -> Markup {
    html! {
        @if let Some(url) = url { a href=(url) { (text) } } @else { (text) }
    }
}

pub fn home(site: &Site, assets: &Assets, bio: &str, posts: &[Post]) -> Markup {
    let mut page = Page::new(site, None, &site.description, "/");
    page.json_ld.push(json!({
        "@context": "https://schema.org",
        "@type": "Person",
        "name": site.name,
        "url": site.url,
        "jobTitle": site.job_title,
        "worksFor": { "@type": "Organization", "name": site.employer },
        "email": site.links.iter().find_map(|l| l.url.strip_prefix("mailto:")),
        "sameAs": site.links.iter().filter(|l| !l.url.starts_with("mailto:")).map(|l| &l.url).collect::<Vec<_>>(),
    }));
    page.json_ld.push(json!({
        "@context": "https://schema.org",
        "@type": "WebSite",
        "name": site.name,
        "url": site.url,
    }));

    let content = html! {
        (title_row(site, &site.name))
        div.intro { (PreEscaped(bio)) }

        section.section {
            h2 { "Work" }
            ul.entries {
                @for job in &site.work {
                    li.entry {
                        (entry_head(link_or_text(&job.company, job.url.as_deref()), Some(&job.dates)))
                        p { (job.description) }
                    }
                }
            }
        }

        section.section {
            h2 { "Education" }
            ul.entries {
                @for school in &site.education {
                    li.entry {
                        (entry_head(html! { (school.degree) }, Some(&school.dates)))
                        p { (link_or_text(&school.school, school.url.as_deref())) }
                    }
                }
            }
        }

        @if !site.projects.is_empty() {
            section.section {
                h2 { "Projects" }
                ul.entries {
                    @for project in &site.projects {
                        li.entry {
                            (entry_head(link_or_text(&project.name, project.url.as_deref()), None))
                            p { (project.description) }
                            @if !project.tech.is_empty() { span.meta.tags { (project.tech.join(", ")) } }
                        }
                    }
                }
            }
        }

        @if !posts.is_empty() {
            section.section {
                h2 { "Writing" }
                ul.entries.tight {
                    @for post in posts.iter().take(5) {
                        li { (entry_head(html! { a href=(post.path()) { (post.title) } }, Some(&post.date.short()))) }
                    }
                }
                a.more href="/blog/" { "All posts →" }
            }
        }

        section.section {
            h2 { "Elsewhere" }
            p.muted {
                @for (i, link) in site.links.iter().enumerate() {
                    @if i > 0 { (if i + 1 == site.links.len() { ", and " } else { ", " }) }
                    a href=(link.url) { (link.name) }
                }
                "."
            }
        }
    };
    layout(site, assets, page, content)
}

pub fn blog_index(site: &Site, assets: &Assets, posts: &[Post]) -> Markup {
    let page = Page::new(
        site,
        Some("Blog"),
        "Writing on machine learning, programming, and systems.",
        "/blog/",
    );
    let content = html! {
        (title_row(site, "Blog"))
        @if posts.is_empty() {
            p.muted { "Nothing here yet." }
        } @else {
            ul.entries {
                @for post in posts {
                    li.entry {
                        (entry_head(html! { a href=(post.path()) { (post.title) } }, Some(&post.date.short())))
                        p { (post.description) }
                    }
                }
            }
        }
    };
    layout(site, assets, page, content)
}

pub fn post(site: &Site, assets: &Assets, post: &Post) -> Markup {
    let path = post.path();
    let mut page = Page::new(site, Some(&post.title), &post.description, &path);
    page.article = Some(post);
    if post.body.has_code {
        page.stylesheets.push(&assets.syntax);
    }
    if post.body.has_math {
        page.stylesheets.push(&assets.math);
    }
    page.json_ld.push(json!({
        "@context": "https://schema.org",
        "@type": "BlogPosting",
        "headline": post.title,
        "description": post.description,
        "datePublished": post.date.iso(),
        "url": format!("{}{}", site.url, path),
        "author": { "@type": "Person", "name": site.name, "url": site.url },
    }));
    let content = html! {
        article {
            header.post-header {
                (title_row(site, &post.title))
                time.meta datetime=(post.date.iso()) { (post.date.long()) }
            }
            div.prose { (PreEscaped(&post.body.html)) }
        }
        a.more.back href="/blog/" { "← All posts" }
    };
    layout(site, assets, page, content)
}

pub fn not_found(site: &Site, assets: &Assets) -> Markup {
    let mut page = Page::new(site, Some("Not found"), "This page doesn't exist.", "/404.html");
    page.noindex = true;
    let content = html! {
        (title_row(site, "404"))
        p.muted { "This page doesn't exist. " a href="/" { "Go home" } "." }
    };
    layout(site, assets, page, content)
}

/// XML documents are built with maud too; empty elements use `{}` so they close properly.
pub fn sitemap(site: &Site, posts: &[Post]) -> String {
    let latest = posts.first().map(|p| p.date.iso());
    let doc = html! {
        urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" {
            url { loc { (site.url) "/" } @if let Some(d) = &latest { lastmod { (d) } } }
            @if site.has_blog {
                url { loc { (site.url) "/blog/" } @if let Some(d) = &latest { lastmod { (d) } } }
            }
            @for post in posts {
                url { loc { (site.url) (post.path()) } lastmod { (post.date.iso()) } }
            }
        }
    };
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", doc.into_string())
}

pub fn feed(site: &Site, posts: &[Post]) -> String {
    // Only built when there are posts, so the newest one dates the feed.
    let updated = posts.first().map(|p| p.date.iso()).unwrap_or_default();
    let doc = html! {
        feed xmlns="http://www.w3.org/2005/Atom" {
            title { (site.name) }
            subtitle { (site.description) }
            link href={ (site.url) "/" } {}
            link rel="self" href={ (site.url) "/feed.xml" } {}
            id { (site.url) "/" }
            updated { (updated) "T00:00:00Z" }
            author { name { (site.name) } }
            @for post in posts {
                entry {
                    title { (post.title) }
                    link href={ (site.url) (post.path()) } {}
                    id { (site.url) (post.path()) }
                    published { (post.date.iso()) "T00:00:00Z" }
                    updated { (post.date.iso()) "T00:00:00Z" }
                    summary { (post.description) }
                    content type="html" { (post.body.feed_html) }
                }
            }
        }
    };
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", doc.into_string())
}
