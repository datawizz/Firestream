/**
 * Check if Supabase is configured with real credentials (not placeholders).
 * Works in both server and client contexts since Next.js inlines process.env at build time.
 */
export function hasValidSupabaseConfig(): boolean {
  const url = process.env.PUBLIC_SUPABASE_URL;
  const key = process.env.PUBLIC_SUPABASE_ANON_KEY;
  if (!url || !key) return false;
  if (url.includes('your-project') || key.includes('your-')) return false;
  return true;
}
