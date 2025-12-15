const BLOG_MANIFEST = './content/blog/manifest.json';
const PAPERS_MANIFEST = './content/papers/manifest.json';
const PROJECTS_DATA = './data/projects.json';
const WORK_DATA = './data/work.json';
const ROUTABLE_PAGES = new Set(['home', 'blog', 'projects', 'papers']);

const state = {
  work: [],
  projects: [],
  blog: [],
  blogIndex: new Map(),
  papers: []
};

const FRONT_MATTER_REGEX = /^---\s*[\r\n]+([\s\S]+?)\s*---\s*/;
const EMPTY_TARGETS = ['work-list', 'home-blog-container', 'home-projects', 'all-posts', 'projects-list', 'papers-list'];

function configureMarkdown() {
  if (!window.marked) return;
  marked.setOptions({
    highlight(code, lang) {
      if (lang && window.hljs?.getLanguage(lang)) {
        return window.hljs.highlight(code, { language: lang }).value;
      }
      return window.hljs ? window.hljs.highlightAuto(code).value : code;
    },
    gfm: true,
    breaks: true
  });
}

function parseFrontMatter(markdown) {
  if (!markdown.startsWith('---')) {
    return { meta: {}, content: markdown.trim() };
  }

  const match = markdown.match(FRONT_MATTER_REGEX);
  if (!match) {
    return { meta: {}, content: markdown.trim() };
  }

  const rawMeta = match[1].split(/\r?\n/);
  const meta = {};

  rawMeta.forEach((line) => {
    if (!line.trim()) return;
    const [key, ...rest] = line.split(':');
    if (!key || rest.length === 0) return;
    const value = rest.join(':').trim().replace(/^"|"$/g, '');
    meta[key.trim()] = value;
  });

  const content = markdown.slice(match[0].length).trim();
  return { meta, content };
}

async function fetchJSON(path) {
  const res = await fetch(path, { cache: 'no-store' });
  if (!res.ok) {
    throw new Error(`Failed to load ${path}: ${res.status}`);
  }
  return res.json();
}

async function loadMarkdownCollection(manifestPath) {
  const manifest = await fetchJSON(manifestPath);
  const entries = [];

  for (const item of manifest) {
    const filePath = item.file.startsWith('./') ? item.file : './' + item.file;
    const response = await fetch(filePath, { cache: 'no-store' });
    if (!response.ok) {
      throw new Error(`Failed to load ${filePath}: ${response.status}`);
    }
    const raw = await response.text();
    const { meta, content } = parseFrontMatter(raw);
    entries.push({ slug: item.slug, content, ...meta });
  }

  return entries;
}

function renderWork() {
  const container = document.getElementById('work-list');
  if (!container) return;

  if (!state.work || state.work.length === 0) {
    container.innerHTML = '<p class="empty-state">No work experience listed.</p>';
    return;
  }

  container.innerHTML = state.work.map((job) => `
    <div class="work-item">
      <div class="work-header">
        <div class="work-info">
          <span class="work-company">
            ${job.companyUrl ? `<a href="${job.companyUrl}" target="_blank" rel="noreferrer">${job.company}</a>` : job.company}
          </span>
          <span class="work-title">${job.title}</span>
        </div>
        <span class="work-date">${job.date}</span>
      </div>
      <p class="work-desc">${job.description}</p>
    </div>
  `).join('');
}

function renderProjectPreview() {
  const container = document.getElementById('home-projects');
  if (!container) return;

  if (!state.projects || state.projects.length === 0) {
    container.innerHTML = '<p class="empty-state">No projects listed.</p>';
    return;
  }

  const preview = state.projects.slice(0, 3);
  container.innerHTML = preview.map((project) => `
    <a class="project-preview" href="${project.link}" target="_blank" rel="noreferrer">
      <div class="project-preview-header">
        <span class="project-preview-name">${project.name}</span>
        <span class="project-preview-role">${project.role}</span>
      </div>
      <p class="project-preview-desc">${project.description}</p>
    </a>
  `).join('') + `
    <a class="project-preview-link" data-page="projects">all projects <span class="arrow">→</span></a>
  `;

  container.querySelectorAll('[data-page]').forEach((el) => {
    el.addEventListener('click', (event) => {
      event.preventDefault();
      showPage('projects');
      history.pushState(null, '', '#projects');
    });
  });
}

function renderProjectCards() {
  const container = document.getElementById('projects-list');
  if (!container) return;

  if (!state.projects || state.projects.length === 0) {
    container.innerHTML = '<p class="empty-state">No projects listed.</p>';
    return;
  }

  container.innerHTML = state.projects.map((project) => `
    <a class="project-card" href="${project.link}" target="_blank" rel="noreferrer">
      <div class="project-card-header">
        <div>
          <span class="project-name">${project.name}</span>
          <span class="project-role">${project.role}</span>
        </div>
        <span class="project-timeframe">${project.timeframe}</span>
      </div>
      <p class="project-desc">${project.description}</p>
      <h4 class="project-section-title">achievements</h4>
      <ul class="project-achievements">
        ${project.achievements.map((achievement) => `<li>${achievement}</li>`).join('')}
      </ul>
      <h4 class="project-section-title">technologies</h4>
      <div class="project-tags">
        ${project.technologies.map((tech) => `<span class="project-tag">${tech}</span>`).join('')}
      </div>
      <span class="project-cta">open <span class="arrow">→</span></span>
    </a>
  `).join('');
}

function renderPapers() {
  const container = document.getElementById('papers-list');
  if (!container) return;

  if (!state.papers || state.papers.length === 0) {
    container.innerHTML = '<p class="empty-state">coming soon</p>';
    return;
  }

  container.innerHTML = state.papers.map((paper) => `
    <a class="paper-card" href="${paper.link || '#'}" target="_blank" rel="noreferrer">
      <div>
        <span class="paper-title">${paper.title}</span>
        <div class="paper-meta">${paper.venue || ''} · ${paper.date || ''}</div>
        ${paper.summary ? `<p class="paper-desc">${paper.summary}</p>` : ''}
      </div>
      <span class="paper-cta">read <span class="arrow">→</span></span>
    </a>
  `).join('');
}

function renderPostList(containerId, entries) {
  const container = document.getElementById(containerId);
  if (!container) return;

  if (!entries || entries.length === 0) {
    container.innerHTML = '<p class="empty-state">No posts available.</p>';
    return;
  }

  container.innerHTML = entries.map((post) => `
    <a class="post" data-slug="${post.slug}">
      <span class="post-title">${post.title || 'Untitled'}</span>
      <span class="post-date">${post.date || ''}</span>
    </a>
  `).join('');

  container.querySelectorAll('[data-slug]').forEach((el) => {
    el.addEventListener('click', (event) => {
      event.preventDefault();
      showArticle(el.dataset.slug);
    });
  });
}

function renderBlogPreview() {
  const container = document.getElementById('home-blog-container');
  if (!container) return;

  const preview = state.blog.slice(0, 3);

  if (!preview || preview.length === 0) {
    container.innerHTML = '<p class="empty-state">No posts available.</p>';
    return;
  }

  const postsHtml = `<div class="post-list">${preview.map((post) => `
    <a class="post" data-slug="${post.slug}">
      <span class="post-title">${post.title || 'Untitled'}</span>
      <span class="post-date">${post.date || ''}</span>
    </a>
  `).join('')}</div>`;

  container.innerHTML = postsHtml + `
    <a class="section-link" data-page="blog">all posts <span class="arrow">→</span></a>
  `;

  container.querySelectorAll('[data-slug]').forEach((el) => {
    el.addEventListener('click', (event) => {
      event.preventDefault();
      showArticle(el.dataset.slug);
    });
  });

  container.querySelectorAll('[data-page]').forEach((el) => {
    el.addEventListener('click', (event) => {
      event.preventDefault();
      showPage('blog');
      history.pushState(null, '', '#blog');
    });
  });
}

function renderBlogLists() {
  renderBlogPreview();
  renderPostList('all-posts', state.blog);
}

function showArticle(slug, pushHistory = true) {
  const post = state.blogIndex.get(slug);
  if (!post) return;

  const titleEl = document.getElementById('article-title');
  const metaEl = document.getElementById('article-meta');
  const contentEl = document.getElementById('article-content');

  if (titleEl) titleEl.textContent = post.title || '';
  if (metaEl) metaEl.textContent = post.date || '';
  if (contentEl && window.marked) {
    contentEl.innerHTML = marked.parse(post.content || '');
    contentEl.querySelectorAll('pre code').forEach((block) => {
      window.hljs?.highlightElement(block);
    });
  }

  showPage('blog-article');

  if (pushHistory) {
    history.pushState({ slug }, '', `#blog/${slug}`);
  }
}

function showPage(pageId) {
  const target = document.getElementById(pageId);
  const resolved = target ? pageId : 'home';
  document.querySelectorAll('.page').forEach((page) => page.classList.remove('active'));
  document.getElementById(resolved)?.classList.add('active');

  const navTarget = resolved === 'blog-article' ? 'blog' : resolved;
  document.querySelectorAll('.nav-links a').forEach((link) => {
    link.classList.toggle('active', link.dataset.page === navTarget);
  });

  window.scrollTo({ top: 0 });
}

function bindNavigation() {
  document.querySelectorAll('[data-page]').forEach((link) => {
    link.addEventListener('click', (event) => {
      event.preventDefault();
      const pageId = link.dataset.page;
      showPage(pageId);
      history.pushState(null, '', `#${pageId}`);
    });
  });
}

function handleRoute() {
  const hash = window.location.hash.slice(1);
  if (!hash) {
    showPage('home');
    return;
  }

  if (hash.startsWith('blog/')) {
    const slug = hash.split('/')[1];
    if (slug) {
      showArticle(slug, false);
      return;
    }
    showPage('blog');
    return;
  }

  if (ROUTABLE_PAGES.has(hash)) {
    showPage(hash);
    return;
  }

  showPage('home');
}

function showDataError(message) {
  EMPTY_TARGETS.forEach((id) => {
    const el = document.getElementById(id);
    if (el && !el.children.length) {
      el.innerHTML = `<p class="empty-state">${message}</p>`;
    }
  });
}

async function loadData() {
  if (window.location.protocol === 'file:') {
    showDataError("Content can't load from file://. Run a local server.");
    return;
  }

  try {
    const [work, projects, blog, papers] = await Promise.all([
      fetchJSON(WORK_DATA),
      fetchJSON(PROJECTS_DATA),
      loadMarkdownCollection(BLOG_MANIFEST),
      loadMarkdownCollection(PAPERS_MANIFEST)
    ]);

    state.work = work || [];
    state.projects = projects || [];
    state.blog = blog || [];
    state.blogIndex = new Map((blog || []).map((entry) => [entry.slug, entry]));
    state.papers = papers || [];

    renderWork();
    renderProjectPreview();
    renderProjectCards();
    renderPapers();
    renderBlogLists();
  } catch (error) {
    console.error('Failed to initialize site', error);
    showDataError('Unable to load content. Check console for details.');
  }
}

function bootstrap() {
  configureMarkdown();
  bindNavigation();
  window.addEventListener('popstate', handleRoute);
  handleRoute();
  loadData();
}

bootstrap();
