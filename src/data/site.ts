export const siteConfig = {
  name: "Josh Schell",
  title: "Josh Schell",
  description:
    "ML Engineer interested in continual learning, multimodal agents, mechanistic interpretability, and AI alignment.",
  url: "https://JoshuaSchell.github.io",
  ogImage: "/og-image.png",
  location: "Salt Lake City, UT",
  email: "contact@joshuaschell.com",
} as const;

export type SiteConfig = typeof siteConfig;
