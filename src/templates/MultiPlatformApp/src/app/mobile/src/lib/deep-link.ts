import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
import { createMobileAuthServices } from './auth';

export type DeepLinkHandler = (url: string) => void | Promise<void>;

export function parseOAuthCallback(url: string): {
  code?: string;
  error?: string;
  error_description?: string;
} | null {
  try {
    const urlObj = new URL(url);

    if (urlObj.protocol !== 'multiplatformapp:' || !urlObj.pathname.includes('auth/callback')) {
      return null;
    }

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

export async function handleOAuthCallback(url: string): Promise<boolean> {
  const params = parseOAuthCallback(url);

  if (!params) {
    return false;
  }

  if (params.error) {
    console.error('OAuth error:', params.error, params.error_description);
    return false;
  }

  if (params.code) {
    try {
      const { authService } = createMobileAuthServices();
      // The Supabase client (via the Tauri adapter) handles code exchange
      // internally when refreshSession is called after receiving the callback.
      await authService.refreshSession();
      console.log('OAuth callback handled successfully');
      return true;
    } catch (error) {
      console.error('Failed to handle OAuth callback:', error);
      return false;
    }
  }

  return false;
}

export async function initDeepLinkListener(customHandler?: DeepLinkHandler): Promise<void> {
  try {
    await onOpenUrl(async (urls) => {
      console.log('Deep link received:', urls);

      for (const url of urls) {
        const handled = await handleOAuthCallback(url);

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

export function parsePortalCallback(url: string): boolean {
  try {
    const urlObj = new URL(url);
    return urlObj.searchParams.get('portal') === 'return';
  } catch {
    return false;
  }
}
