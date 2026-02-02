import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const blog = defineCollection({
  loader: glob({ pattern: '**/*.mdx', base: './src/content/blog' }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    publishedDate: z.coerce.date(),
    draft: z.boolean().default(false),
  }),
});

const projects = defineCollection({
  loader: glob({ pattern: '**/*.json', base: './src/content/projects' }),
  schema: z.object({
    name: z.string(),
    description: z.string(),
    longDescription: z.string().optional(),
    technologies: z.array(z.string()),
    achievements: z.array(z.string()).optional(),
    link: z.string().url().optional(),
    demo: z.string().url().optional(),
    image: z.string().optional(),
    featured: z.boolean().default(false),
    order: z.number().default(0),
  }),
});

const papers = defineCollection({
  loader: glob({ pattern: '**/*.json', base: './src/content/papers' }),
  schema: z.object({
    title: z.string(),
    authors: z.array(z.string()),
    abstract: z.string(),
    venue: z.string().optional(),
    year: z.number(),
    doi: z.string().optional(),
    arxiv: z.string().url().optional(),
    pdf: z.string().url().optional(),
  }),
});

export const collections = { blog, projects, papers };
