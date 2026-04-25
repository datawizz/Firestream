import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  transpilePackages: [
    '@multi-platform-app/shared',
    '@multi-platform-app/auth',

  ],
  experimental: {
    optimizePackageImports: [
      '@multi-platform-app/shared',
      '@multi-platform-app/auth',

    ],
  },
  env: {
    PUBLIC_SUPABASE_URL: process.env.PUBLIC_SUPABASE_URL,
    PUBLIC_SUPABASE_ANON_KEY: process.env.PUBLIC_SUPABASE_ANON_KEY,
    PUBLIC_STRIPE_PUBLISHABLE_KEY: process.env.PUBLIC_STRIPE_PUBLISHABLE_KEY,
    PUBLIC_SITE_URL: process.env.PUBLIC_SITE_URL,
  },
};

export default nextConfig;
