import { redirect } from 'next/navigation';
import type { ReactNode } from 'react';
import { getSession } from '@/lib/auth';
import { hasValidSupabaseConfig } from '@/lib/supabase-config';
import { DevAccountGuard } from './dev-account';

export default async function AccountLayout({ children }: { children: ReactNode }) {
  if (!hasValidSupabaseConfig()) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
        <div className="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
          <DevAccountGuard>{children}</DevAccountGuard>
        </div>
      </div>
    );
  }

  const session = await getSession();

  // Redirect to login if not authenticated
  if (!session) {
    redirect('/login');
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">{children}</div>
    </div>
  );
}
