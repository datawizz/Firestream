import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
import { createDesktopAuthServices } from './auth';

/**
 * Deep link handler callback type
 */
export type DeepLinkHandler = (url: string) => void | Promise<void>;

/**
 * Parse OAuth callback URL parameters
 */
export function parseOAuthCallback(url: string): {
  code?: string;
  error?: string;
  error_description?: string;
} | null {
  try {
    const urlObj = new URL(url);

    // Check if this is an auth callback
    if (urlObj.protocol !== 'multiplatformapp:' || !urlObj.pathname.includes('auth/callback')) {
      return null;
    }

    // Extract query parameters
    const params = urlObj.searchParams;
    const code = params.get('code');
    const error = params.get('error');
    const error_description = params.get('error_description');

    return {
      code: code || undefined,
      error: error || undefined,
      error_description: error_description || undefined,
    };
  } catch (err) {
    console.error('Failed to parse OAuth callback URL:', err);
    return null;
  }
}

/**
 * Handle OAuth callback deep link
 *
 * This exchanges the authorization code for a session and stores it
 * using the secure storage adapter.
 */
export async function handleOAuthCallback(url: string): Promise<boolean> {
  const params = parseOAuthCallback(url);

  if (!params) {
    return false;
  }

  // Handle errors
  if (params.error) {
    console.error('OAuth error:', params.error, params.error_description);
    return false;
  }

  // Exchange code for session
  if (params.code) {
    try {
      const { authService } = createDesktopAuthServices();

      // Exchange the code for a session
      // The Supabase client will handle this via the auth callback
      await authService.exchangeCodeForSession(params.code);

      console.log('OAuth callback handled successfully');
      return true;
    } catch (error) {
      console.error('Failed to exchange OAuth code for session:', error);
      return false;
    }
  }

  return false;
}

/**
 * Initialize deep link listener
 *
 * Call this once during app initialization to set up the deep link handler.
 * It will automatically handle OAuth callbacks and other deep links.
 *
 * @param customHandler - Optional custom handler for non-auth deep links
 */
export async function initDeepLinkListener(customHandler?: DeepLinkHandler): Promise<void> {
  try {
    // Listen for deep link events
    await onOpenUrl(async (urls) => {
      console.log('Deep link received:', urls);

      for (const url of urls) {
        // Try to handle as OAuth callback first
        const handled = await handleOAuthCallback(url);

        // If not handled and custom handler exists, call it
        if (!handled && customHandler) {
          await customHandler(url);
        }
      }
    });

    console.log('Deep link listener initialized');
  } catch (error) {
    console.error('Failed to initialize deep link listener:', error);
    throw error;
  }
}

/**
 * Handle checkout success/cancel callbacks
 */
export function parseCheckoutCallback(url: string): {
  status: 'success' | 'cancel';
} | null {
  try {
    const urlObj = new URL(url);
    const checkout = urlObj.searchParams.get('checkout');

    if (checkout === 'success') {
      return { status: 'success' };
    }

    if (checkout === 'cancel') {
      return { status: 'cancel' };
    }

    return null;
  } catch {
    return null;
  }
}

/**
 * Handle portal return callback
 */
export function parsePortalCallback(url: string): boolean {
  try {
    const urlObj = new URL(url);
    return urlObj.searchParams.get('portal') === 'return';
  } catch {
    return false;
  }
}
