import { useState, type ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider } from '@multi-platform-app/auth/context';
import { createDesktopAuthServices } from './lib/auth';

export function Providers({ children }: { children: ReactNode }) {
  const [queryClient] = useState(() => new QueryClient());
  const [{ authService, subscriptionService }] = useState(() =>
    createDesktopAuthServices()
  );

  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider
        authService={authService}
        subscriptionService={subscriptionService}
      >
        {children}
      </AuthProvider>
    </QueryClientProvider>
  );
}
