# Web Application (Next.js 15)

Next.js 15 web application with App Router, demonstrating platform adapter pattern.

## Features

- Next.js 15 with Turbopack
- React 19
- App Router
- Server-side rendering
- API routes
- Shared library integration
- Platform adapter pattern (Web storage)

## Development

```bash
# Install dependencies
pnpm install

# Start development server
pnpm dev

# Build for production
pnpm build

# Start production server
pnpm start
```

## Structure

- `src/app/` - App Router pages and layouts
- `src/app/api/` - API routes
- `src/app/providers.tsx` - Client-side providers (React Query)

## Environment Variables

Copy `.env.local.example` to `.env.local` and configure as needed.
