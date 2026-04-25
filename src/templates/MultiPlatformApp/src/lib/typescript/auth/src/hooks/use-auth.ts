/**
 * useAuth hook
 *
 * React hook for accessing authentication state and operations
 *
 * @packageDocumentation
 */

import { useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useAuthStore } from '../stores/auth-store';
import { useAuthService } from '../context/auth-provider';
import type {
  SignInWithEmailOptions,
  SignInWithPasswordOptions,
  SignInWithOAuthOptions,
  SignUpWithEmailOptions,
  AuthUser,
} from '../types/auth';

/**
 * Authentication hook
 *
 * Provides authentication state and operations using React Query and Zustand.
 * Automatically syncs server state with local store and handles authentication flows.
 *
 * @example
 * ```tsx
 * function LoginForm() {
 *   const { signIn, signOut, user, isLoading, isAuthenticated } = useAuth();
 *
 *   const handleLogin = async () => {
 *     const result = await signIn({
 *       email: 'user@example.com',
 *       password: 'password123'
 *     });
 *     if (result.error) {
 *       console.error('Login failed:', result.error);
 *     }
 *   };
 *
 *   return (
 *     <div>
 *       {isAuthenticated ? (
 *         <div>
 *           <p>Welcome {user?.email}</p>
 *           <button onClick={() => signOut()}>Sign Out</button>
 *         </div>
 *       ) : (
 *         <button onClick={handleLogin} disabled={isLoading}>
 *           Sign In
 *         </button>
 *       )}
 *     </div>
 *   );
 * }
 * ```
 */
export function useAuth() {
  const authService = useAuthService();
  const queryClient = useQueryClient();

  // Get state from store
  const { user, isAuthenticated, setUser, clearUser, setInitialized, setSession } = useAuthStore();

  // Fetch current user with React Query
  const {
    data: currentUser,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['auth', 'user'],
    queryFn: async () => {
      const result = await authService.getUser();
      return result;
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
    retry: false,
  });

  // Sync user data to store when query updates
  useEffect(() => {
    if (currentUser !== undefined) {
      setUser(currentUser);
      setInitialized(true);
    }
  }, [currentUser, setUser, setInitialized]);

  // Fetch session
  const { data: session } = useQuery({
    queryKey: ['auth', 'session'],
    queryFn: async () => {
      const result = await authService.getSession();
      return result;
    },
    staleTime: 1000 * 60 * 5,
    retry: false,
  });

  // Sync session to store
  useEffect(() => {
    if (session !== undefined) {
      setSession(session);
    }
  }, [session, setSession]);

  // Sign in with password mutation
  const signInMutation = useMutation({
    mutationFn: async (options: SignInWithPasswordOptions) => {
      return authService.signInWithPassword(options);
    },
    onSuccess: async (result) => {
      if (result.data) {
        await queryClient.invalidateQueries({ queryKey: ['auth', 'user'] });
        await queryClient.invalidateQueries({ queryKey: ['auth', 'session'] });
      }
    },
  });

  // Sign in with email (magic link) mutation
  const signInWithEmailMutation = useMutation({
    mutationFn: async (options: SignInWithEmailOptions) => {
      return authService.signInWithEmail(options);
    },
  });

  // Sign in with OAuth mutation
  const signInWithOAuthMutation = useMutation({
    mutationFn: async (options: SignInWithOAuthOptions) => {
      return authService.signInWithOAuth(options);
    },
  });

  // Sign up mutation
  const signUpMutation = useMutation({
    mutationFn: async (options: SignUpWithEmailOptions) => {
      return authService.signUpWithEmail(options);
    },
    onSuccess: async (result) => {
      if (result.data) {
        await queryClient.invalidateQueries({ queryKey: ['auth', 'user'] });
        await queryClient.invalidateQueries({ queryKey: ['auth', 'session'] });
      }
    },
  });

  // Sign out mutation
  const signOutMutation = useMutation({
    mutationFn: async () => {
      return authService.signOut();
    },
    onSuccess: () => {
      clearUser();
      queryClient.clear(); // Clear all queries on sign out
    },
  });

  // Convenience methods
  const signIn = async (options: SignInWithPasswordOptions) => {
    return signInMutation.mutateAsync(options);
  };

  const signInWithEmail = async (options: SignInWithEmailOptions) => {
    return signInWithEmailMutation.mutateAsync(options);
  };

  const signInWithOAuth = async (options: SignInWithOAuthOptions) => {
    return signInWithOAuthMutation.mutateAsync(options);
  };

  const signUp = async (options: SignUpWithEmailOptions) => {
    return signUpMutation.mutateAsync(options);
  };

  const signOut = async () => {
    return signOutMutation.mutateAsync();
  };

  return {
    // State
    user: user as AuthUser | null,
    isLoading,
    isAuthenticated,
    error: error as Error | null,

    // Methods
    signIn,
    signInWithEmail,
    signInWithOAuth,
    signUp,
    signOut,
    refetch,

    // Mutation states
    isSigningIn: signInMutation.isPending,
    isSigningUp: signUpMutation.isPending,
    isSigningOut: signOutMutation.isPending,
  };
}
