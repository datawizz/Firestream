'use client';

import Link from 'next/link';
import { DemoView, Button } from '@multi-platform-app/shared/components';
import { useAuth } from '@multi-platform-app/auth/hooks';

export default function Home() {
  const { user } = useAuth();

  return (
    <DemoView
      platformLabel="Web"
      headerExtra={
        <Link href={user ? '/account' : '/login'}>
          <Button variant="outline">
            {user ? `Account (${user.email})` : 'Sign In'}
          </Button>
        </Link>
      }
      footerExtra={
        <div className="mb-8 p-6 border rounded-lg">
          <h2 className="text-2xl font-semibold mb-4">API Route Example</h2>
          <p className="text-sm text-muted-foreground">
            Try: <code className="bg-muted px-2 py-1 rounded">/api/hello</code>
          </p>
        </div>
      }
    />
  );
}
