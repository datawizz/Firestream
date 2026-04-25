/**
 * useSession hook
 *
 * React hook for accessing the current authentication session
 *
 * @packageDocumentation
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useAuthStore } from '../stores/auth-store';
import { useAuthService } from '../context/auth-provider';
import type { Session } from '../types/auth';

/**
 * Session hook
 *
 * Provides access to the current authentication session and session management operations.
 *
 * @example
 * ```tsx
 * function SessionInfo() {
 *   const { session, isValid, refresh, isRefreshing } = useSession();
 *
 *   if (!session) {
 *     return <p>No active session</p>;
 *   }
 *
 *   return (
 *     <div>
 *       <p>Session expires: {new Date(session.expires_at * 1000).toLocaleString()}</p>
 *       <p>Valid: {isValid ? 'Yes' : 'No'}</p>
 *       <button onClick={() => refresh()} disabled={isRefreshing}>
 *         Refresh Session
 *       </button>
 *     </div>
 *   );
 * }
 * ```
 */
export function useSession() {
  const authService = useAuthService();
  const queryClient = useQueryClient();
  const { session: storedSession, setSession } = useAuthStore();

  // Fetch current session
  const {
    data: session,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['auth', 'session'],
    queryFn: async () => {
      const result = await authService.getSession();
      return result;
    },
    staleTime: 1000 * 60 * 5, // 5 minutes
    retry: false,
  });

  // Refresh session mutation
  const refreshMutation = useMutation({
    mutationFn: async () => {
      return authService.refreshSession();
    },
    onSuccess: async (result) => {
      if (result.data) {
        setSession(result.data);
        await queryClient.invalidateQueries({ queryKey: ['auth', 'session'] });
        await queryClient.invalidateQueries({ queryKey: ['auth', 'user'] });
      }
    },
  });

  /**
   * Check if the current session is valid
   */
  const isValid = (): boolean => {
    const currentSession = session ?? storedSession;
    if (!currentSession) return false;

    // Check if session has expired
    const expiresAt = currentSession.expires_at;
    if (!expiresAt) return true; // If no expiry, assume valid

    const now = Math.floor(Date.now() / 1000);
    return now < expiresAt;
  };

  /**
   * Get time until session expires (in seconds)
   */
  const getTimeUntilExpiry = (): number | null => {
    const currentSession = session ?? storedSession;
    if (!currentSession?.expires_at) return null;

    const now = Math.floor(Date.now() / 1000);
    return currentSession.expires_at - now;
  };

  /**
   * Refresh the current session
   */
  const refresh = async () => {
    return refreshMutation.mutateAsync();
  };

  return {
    // State
    session: (session ?? storedSession) as Session | null,
    isLoading,
    error: error as Error | null,

    // Methods
    refresh,
    isValid: isValid(),
    getTimeUntilExpiry,

    // Mutation states
    isRefreshing: refreshMutation.isPending,
  };
}
