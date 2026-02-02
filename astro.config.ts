import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";
import sitemap from "@astrojs/sitemap";
import tailwind from "@astrojs/tailwind";
import expressiveCode from "astro-expressive-code";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";

export default defineConfig({
  site: "https://JoshuaSchell.github.io",
  integrations: [
    tailwind({ applyBaseStyles: false }),
    expressiveCode({
      themes: ["gruvbox-dark-medium", "github-light"],
      themeCssSelector: (theme) => {
        if (theme.name === "gruvbox-dark-medium") return ".dark";
        return ":root:not(.dark)";
      },
      styleOverrides: {
        borderRadius: "0",
        codeFontFamily: "Geist Mono, ui-monospace, monospace",
        borderColor: ({ theme }) =>
          theme.type === "dark" ? "#504945" : "#d0d7de",
        frames: {
          frameBoxShadowCssValue: "none",
        },
      },
    }),
    mdx(),
    sitemap(),
  ],
  markdown: {
    remarkPlugins: [remarkMath],
    rehypePlugins: [rehypeKatex],
  },
  compressHTML: true,
  prefetch: true, // Prefetch on hover only (not all links)
});
