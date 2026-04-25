'use client';

import { useRouter } from 'next/navigation';
import { useAuth } from '@multi-platform-app/auth/hooks';
import type { ReactNode } from 'react';

export function DevAccountGuard({ children }: { children: ReactNode }) {
  const { user, isLoading } = useAuth();
  const router = useRouter();

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <p className="text-muted-foreground">Loading...</p>
      </div>
    );
  }

  if (!user) {
    router.replace('/login');
    return null;
  }

  return <>{children}</>;
}

export function DevAccountPage() {
  const { user, signOut, isSigningOut } = useAuth();
  const router = useRouter();

  async function handleSignOut() {
    await signOut();
    router.replace('/login');
  }

  if (!user) return null;

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          Account
        </h1>
        <p className="mt-2 text-sm text-gray-500 dark:text-gray-400">
          Manage your account settings and subscription
        </p>
      </div>

      <div className="p-3 rounded-lg bg-amber-50 dark:bg-amber-900/20 text-amber-800 dark:text-amber-200 text-sm">
        Dev mode — using mock authentication
      </div>

      <div className="overflow-hidden rounded-lg bg-white shadow dark:bg-gray-800">
        <div className="border-b border-gray-200 px-6 py-4 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Profile Information
          </h2>
        </div>
        <div className="px-6 py-4">
          <dl className="space-y-4">
            <div>
              <dt className="text-sm font-medium text-gray-500 dark:text-gray-400">Email</dt>
              <dd className="mt-1 text-sm text-gray-900 dark:text-white">{user.email}</dd>
            </div>
            <div>
              <dt className="text-sm font-medium text-gray-500 dark:text-gray-400">User ID</dt>
              <dd className="mt-1 text-sm font-mono text-gray-900 dark:text-white">{user.id}</dd>
            </div>
          </dl>
        </div>
        <div className="border-t border-gray-200 px-6 py-4 dark:border-gray-700">
          <button
            onClick={handleSignOut}
            disabled={isSigningOut}
            className="rounded-md bg-red-600 px-4 py-2 text-sm font-semibold text-white hover:bg-red-700 disabled:opacity-50"
          >
            {isSigningOut ? 'Signing out...' : 'Sign Out'}
          </button>
        </div>
      </div>
    </div>
  );
}
