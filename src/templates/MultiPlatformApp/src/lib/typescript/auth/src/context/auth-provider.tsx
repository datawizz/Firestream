/**
 * Authentication provider context
 *
 * Provides authentication services and initialization to the React component tree
 *
 * @packageDocumentation
 */

import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import type { AuthService } from '../services/auth-service';
import type { SubscriptionService } from '../services/subscription-service';

/**
 * Platform detection helper
 */
function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window;
}

/**
 * Authentication context value
 */
interface AuthContextValue {
  authService: AuthService | null;
  subscriptionService: SubscriptionService | null;
  isInitialized: boolean;
  platform: 'web' | 'tauri';
}

/**
 * Authentication context
 */
const AuthContext = createContext<AuthContextValue | undefined>(undefined);

/**
 * Authentication provider props
 */
interface AuthProviderProps {
  children: ReactNode;
  authService?: AuthService;
  subscriptionService?: SubscriptionService;
}

/**
 * Authentication provider component
 *
 * Initializes auth and subscription services and provides them to child components.
 * Automatically detects platform and initializes appropriate adapters.
 *
 * @example
 * ```tsx
 * import { AuthProvider } from '@multi-platform-app/auth';
 * import { createWebAuthService, createWebSubscriptionService } from '@multi-platform-app/auth/adapters/web';
 *
 * function App() {
 *   const authService = createWebAuthService(config);
 *   const subscriptionService = createWebSubscriptionService(config);
 *
 *   return (
 *     <AuthProvider
 *       authService={authService}
 *       subscriptionService={subscriptionService}
 *     >
 *       <YourApp />
 *     </AuthProvider>
 *   );
 * }
 * ```
 */
export function AuthProvider({
  children,
  authService,
  subscriptionService,
}: AuthProviderProps) {
  const [isInitialized, setIsInitialized] = useState(false);
  const platform = isTauri() ? 'tauri' : 'web';

  useEffect(() => {
    let mounted = true;

    async function initialize() {
      try {
        if (authService) {
          await authService.initialize();
        }
        if (subscriptionService) {
          await subscriptionService.initialize();
        }

        if (mounted) {
          setIsInitialized(true);
        }
      } catch (error) {
        console.error('Failed to initialize auth services:', error);
        if (mounted) {
          setIsInitialized(true); // Set initialized even on error to prevent blocking
        }
      }
    }

    initialize();

    return () => {
      mounted = false;
    };
  }, [authService, subscriptionService]);

  const value: AuthContextValue = {
    authService: authService ?? null,
    subscriptionService: subscriptionService ?? null,
    isInitialized,
    platform,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

/**
 * Hook to access auth context
 *
 * @throws Error if used outside of AuthProvider
 */
export function useAuthContext(): AuthContextValue {
  const context = useContext(AuthContext);

  if (context === undefined) {
    throw new Error('useAuthContext must be used within an AuthProvider');
  }

  return context;
}

/**
 * Hook to access auth service
 *
 * @throws Error if used outside of AuthProvider or if authService is not provided
 */
export function useAuthService(): AuthService {
  const { authService } = useAuthContext();

  if (!authService) {
    throw new Error('AuthService not provided to AuthProvider');
  }

  return authService;
}

/**
 * Hook to access subscription service
 *
 * @throws Error if used outside of AuthProvider or if subscriptionService is not provided
 */
export function useSubscriptionService(): SubscriptionService {
  const { subscriptionService } = useAuthContext();

  if (!subscriptionService) {
    throw new Error('SubscriptionService not provided to AuthProvider');
  }

  return subscriptionService;
}
