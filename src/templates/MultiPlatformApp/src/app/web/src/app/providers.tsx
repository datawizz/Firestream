'use client';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { useState, type ReactNode } from 'react';
import { ThemeProvider } from 'next-themes';
import { AuthProvider } from '@multi-platform-app/auth/context';
import { AuthService, SubscriptionService } from '@multi-platform-app/auth/services';
import { SupabaseWebAdapter } from '@multi-platform-app/auth/adapters/web';
import { DevAuthAdapter, DevSubscriptionAdapter } from '@multi-platform-app/auth/adapters/dev';
import { hasValidSupabaseConfig } from '@/lib/supabase-config';

// Initialize auth services
function createAuthServices() {
  if (!hasValidSupabaseConfig()) {
    if (process.env.NODE_ENV === 'production') {
      console.error('Supabase credentials not configured in production!');
      return { authService: undefined, subscriptionService: undefined };
    }

    const authService = new AuthService(new DevAuthAdapter());
    const subscriptionService = new SubscriptionService(new DevSubscriptionAdapter() as any);
    return { authService, subscriptionService };
  }

  const supabaseUrl = process.env.PUBLIC_SUPABASE_URL!;
  const supabaseAnonKey = process.env.PUBLIC_SUPABASE_ANON_KEY!;

  // Create Supabase web adapter (browser client)
  const authAdapter = new SupabaseWebAdapter({
    supabase: {
      url: supabaseUrl,
      anonKey: supabaseAnonKey,
    },
    redirectUrls: {
      login: '/login',
      callback: '/auth/callback',
      error: '/auth/error',
    },
  });

  const authService = new AuthService(authAdapter);
  const subscriptionService = new SubscriptionService(authAdapter as any);

  return { authService, subscriptionService };
}

export function Providers({ children }: { children: ReactNode }) {
  const [queryClient] = useState(() => new QueryClient());
  const [services] = useState(() => createAuthServices());

  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <QueryClientProvider client={queryClient}>
        <AuthProvider
          authService={services.authService}
          subscriptionService={services.subscriptionService}
        >
          {children}
        </AuthProvider>
        <ReactQueryDevtools initialIsOpen={false} />
      </QueryClientProvider>
    </ThemeProvider>
  );
}
