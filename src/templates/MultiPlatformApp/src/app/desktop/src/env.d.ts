/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly PUBLIC_SUPABASE_URL: string;
  readonly PUBLIC_SUPABASE_ANON_KEY: string;
  readonly PUBLIC_STRIPE_PUBLISHABLE_KEY?: string;
  readonly PUBLIC_SITE_URL?: string;
  readonly PUBLIC_API_URL?: string;
  readonly PUBLIC_APP_TITLE?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
