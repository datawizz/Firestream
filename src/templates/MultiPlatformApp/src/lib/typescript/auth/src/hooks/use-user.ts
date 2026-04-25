/**
 * useUser hook
 *
 * React hook for accessing the current authenticated user
 *
 * @packageDocumentation
 */

import { useAuthStore } from '../stores/auth-store';
import type { AuthUser } from '../types/auth';

/**
 * User hook
 *
 * Provides access to the current authenticated user from the auth store.
 * This is a lightweight hook that doesn't fetch data - it only reads from the store.
 * For fetching and authentication operations, use `useAuth` instead.
 *
 * @example
 * ```tsx
 * function UserProfile() {
 *   const { user, isAuthenticated } = useUser();
 *
 *   if (!isAuthenticated) {
 *     return <p>Please sign in</p>;
 *   }
 *
 *   return (
 *     <div>
 *       <h1>Profile</h1>
 *       <p>Email: {user?.email}</p>
 *       <p>Full Name: {user?.profile?.full_name}</p>
 *     </div>
 *   );
 * }
 * ```
 */
export function useUser() {
  const { user, isAuthenticated, isInitialized } = useAuthStore();

  return {
    user: user as AuthUser | null,
    isAuthenticated,
    isInitialized,
    profile: user?.profile ?? null,
  };
}
