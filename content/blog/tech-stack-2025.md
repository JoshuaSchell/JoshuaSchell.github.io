---
title: "My Recommended Tech Stack for 2025"
date: "Aug 10, 2025"
summary: "A pragmatic stack for shipping full-stack apps quickly without losing type safety."
---

People often ask me what to learn. Here's my current recommendations.

## Frontend

- **Framework:** Next.js or SvelteKit
- **Styling:** Tailwind CSS
- **Components:** shadcn/ui (unstyled, customizable)

## Backend

- **Runtime:** Node.js or Bun
- **API:** tRPC or Hono
- **Database:** PostgreSQL (via Neon or Supabase)
- **ORM:** Drizzle

## DevOps

- **Hosting:** Vercel, Railway, or Fly.io
- **CI/CD:** GitHub Actions

## Example Project Setup

```bash
pnpm create t3-app@latest my-app
cd my-app
pnpm dev
```

This gives you:
- Next.js
- TypeScript
- Tailwind
- tRPC
- Drizzle
- NextAuth.js

All configured and ready to go.

---

*Last updated: August 2025*
