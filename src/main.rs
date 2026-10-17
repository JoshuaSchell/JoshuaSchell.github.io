//! Static site generator: `content/` + `static/` -> `dist/`.

mod markdown;
mod templates;

use std::fs;
use std::path::Path;

use serde::Deserialize;

use markdown::{Markdown, Rendered};

const CONTENT: &str = "content";
const STATIC: &str = "static";
const OUT: &str = "dist";

#[derive(Deserialize)]
pub struct Site {
    pub name: String,
    pub description: String,
    pub url: String,
    pub job_title: String,
    pub employer: String,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default)]
    pub work: Vec<Work>,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub education: Vec<Education>,
    /// Whether any posts are published. Without them there's no blog, feed, or nav.
    #[serde(skip)]
    pub has_blog: bool,
}

#[derive(Deserialize)]
pub struct Link {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize)]
pub struct Work {
    pub company: String,
    pub url: Option<String>,
    pub dates: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct Project {
    pub name: String,
    pub url: Option<String>,
    pub description: String,
    #[serde(default)]
    pub tech: Vec<String>,
}

#[derive(Deserialize)]
pub struct Education {
    pub school: String,
    pub url: Option<String>,
    pub degree: String,
    pub dates: String,
}

#[derive(Deserialize)]
struct FrontMatter {
    title: String,
    description: String,
    date: toml::value::Datetime,
    #[serde(default)]
    draft: bool,
}

pub struct Post {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub date: Date,
    pub body: Rendered,
}

impl Post {
    pub fn path(&self) -> String {
        format!("/blog/{}/", self.slug)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

impl Date {
    pub fn iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// `Jan 10, 2026`
    pub fn short(&self) -> String {
        format!("{} {}, {}", &MONTHS[self.month as usize - 1][..3], self.day, self.year)
    }

    /// `January 10, 2026`
    pub fn long(&self) -> String {
        format!("{} {}, {}", MONTHS[self.month as usize - 1], self.day, self.year)
    }
}

/// URLs for stylesheets and scripts, with a content hash so browsers refetch them after a change.
pub struct Assets {
    pub style: String,
    pub syntax: String,
    pub math: String,
}

fn main() {
    if let Err(e) = build() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn build() -> Result<(), String> {
    let mut site: Site =
        toml::from_str(&read(&format!("{CONTENT}/site.toml"))?).map_err(|e| format!("site.toml: {e}"))?;
    site.url = site.url.trim_end_matches('/').to_owned();

    let md = Markdown::new();
    let bio = md.render(&read(&format!("{CONTENT}/index.md"))?)?.html;
    let posts = load_posts(&md)?;
    site.has_blog = !posts.is_empty();

    if Path::new(OUT).exists() {
        fs::remove_dir_all(OUT).map_err(|e| format!("clearing {OUT}: {e}"))?;
    }
    copy_dir(Path::new(STATIC), Path::new(OUT))?;
    write("syntax.css", &markdown::syntax_css()?)?;

    let assets = Assets {
        style: versioned("style.css")?,
        syntax: versioned("syntax.css")?,
        math: versioned("math.css")?,
    };

    write(
        "index.html",
        &templates::home(&site, &assets, &bio, &posts).into_string(),
    )?;
    write("404.html", &templates::not_found(&site, &assets).into_string())?;
    write("sitemap.xml", &templates::sitemap(&site, &posts))?;
    if site.has_blog {
        write(
            "blog/index.html",
            &templates::blog_index(&site, &assets, &posts).into_string(),
        )?;
        for post in &posts {
            write(
                &format!("blog/{}/index.html", post.slug),
                &templates::post(&site, &assets, post).into_string(),
            )?;
        }
        write("feed.xml", &templates::feed(&site, &posts))?;
    }

    println!("built {} post(s) into {OUT}/", posts.len());
    Ok(())
}

/// Published posts from `content/blog/*.md`, newest first.
fn load_posts(md: &Markdown) -> Result<Vec<Post>, String> {
    let dir = format!("{CONTENT}/blog");
    let mut posts = Vec::new();
    if !Path::new(&dir).exists() {
        return Ok(posts);
    }
    for entry in fs::read_dir(&dir).map_err(|e| format!("{dir}: {e}"))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        let name = path.display().to_string();
        let source = read(&name)?;
        let (front, body) = split_front_matter(&source).ok_or_else(|| format!("{name}: missing +++ front matter"))?;
        let front: FrontMatter = toml::from_str(front).map_err(|e| format!("{name}: {e}"))?;
        if front.draft {
            continue;
        }
        let date = front
            .date
            .date
            .ok_or_else(|| format!("{name}: date needs a calendar day"))?;
        posts.push(Post {
            slug: path.file_stem().unwrap().to_string_lossy().into_owned(),
            title: front.title,
            description: front.description,
            date: Date {
                year: date.year,
                month: date.month,
                day: date.day,
            },
            body: md.render(body).map_err(|e| format!("{name}: {e}"))?,
        });
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.slug.cmp(&b.slug)));
    Ok(posts)
}

/// Splits `+++\n<toml>\n+++\n<markdown>`.
fn split_front_matter(source: &str) -> Option<(&str, &str)> {
    let rest = source.strip_prefix("+++")?.trim_start_matches(['\r', '\n']);
    let end = rest.find("\n+++")?;
    Some((&rest[..end], rest[end + 4..].trim_start_matches(['\r', '\n'])))
}

fn read(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

fn write(rel: &str, contents: &str) -> Result<(), String> {
    let path = Path::new(OUT).join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(&path, contents).map_err(|e| format!("{}: {e}", path.display()))
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for entry in fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).map_err(|e| format!("{}: {e}", entry.path().display()))?;
        }
    }
    Ok(())
}

/// `/name?v=<hash of the built file>`
fn versioned(rel: &str) -> Result<String, String> {
    let path = Path::new(OUT).join(rel);
    let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    // FNV-1a: stable across builds and Rust versions, unlike std's DefaultHasher.
    let hash = bytes.iter().fold(0xcbf29ce484222325u64, |h, &b| {
        (h ^ b as u64).wrapping_mul(0x100000001b3)
    });
    Ok(format!("/{rel}?v={:08x}", hash as u32))
}
