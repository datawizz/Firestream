import { createAuthAdapter } from '@multi-platform-app/auth/adapters';
import { DevAuthAdapter } from '@multi-platform-app/auth/adapters/dev';
import { DevSubscriptionAdapter } from '@multi-platform-app/auth/adapters/dev';
import { AuthService } from '@multi-platform-app/auth/services';
import { SubscriptionService } from '@multi-platform-app/auth/services';
import type { IAuthAdapter } from '@multi-platform-app/auth';

function hasValidSupabaseConfig(): boolean {
  const url = import.meta.env.PUBLIC_SUPABASE_URL;
  const key = import.meta.env.PUBLIC_SUPABASE_ANON_KEY;
  if (!url || !key) return false;
  // Detect placeholder values from .env.public
  if (url.includes('your-project') || key.includes('your-')) return false;
  return true;
}

/**
 * Initialize the auth adapter for Tauri desktop app
 *
 * Falls back to DevAuthAdapter when Supabase credentials are not configured.
 */
function initAuthAdapter(): { adapter: IAuthAdapter; isDevMode: boolean } {
  const supabaseUrl = import.meta.env.PUBLIC_SUPABASE_URL;
  const supabaseAnonKey = import.meta.env.PUBLIC_SUPABASE_ANON_KEY;

  if (!hasValidSupabaseConfig()) {
    return { adapter: new DevAuthAdapter(), isDevMode: true };
  }

  return {
    adapter: createAuthAdapter({
      platform: 'tauri',
      config: {
        supabase: {
          url: supabaseUrl,
          anonKey: supabaseAnonKey,
        },
        deepLink: {
          scheme: 'multiplatformapp',
          host: 'auth',
        },
      },
    }),
    isDevMode: false,
  };
}

export const isDevMode = !hasValidSupabaseConfig();

/**
 * Singleton services
 */
let services: {
  authService: AuthService;
  subscriptionService: SubscriptionService;
} | null = null;

/**
 * Create or return auth services for the desktop app
 */
export function createDesktopAuthServices() {
  if (!services) {
    const { adapter, isDevMode: devMode } = initAuthAdapter();
    const authService = new AuthService(adapter);
    const subscriptionService = devMode
      ? new SubscriptionService(new DevSubscriptionAdapter() as any)
      : undefined;

    services = {
      authService,
      subscriptionService: subscriptionService!,
    };
  }
  return services;
}
