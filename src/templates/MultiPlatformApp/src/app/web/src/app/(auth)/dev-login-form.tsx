'use client';

import { useState, type FormEvent } from 'react';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import { useAuth } from '@multi-platform-app/auth/hooks';

export function DevLoginForm() {
  const router = useRouter();
  const { signIn, isSigningIn } = useAuth();
  const [email, setEmail] = useState('admin@localhost');
  const [password, setPassword] = useState('admin');
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    const result = await signIn({ email, password });
    if (result.error) {
      setError(result.error.message);
    } else {
      router.replace('/');
    }
  }

  return (
    <div className="w-full max-w-md space-y-8">
      <div className="text-center">
        <h2 className="text-3xl font-bold tracking-tight">Sign in to your account</h2>
        <p className="mt-2 text-sm text-muted-foreground">
          Don&apos;t have an account?{' '}
          <Link href="/signup" className="font-medium text-primary hover:text-primary/80">
            Sign up
          </Link>
        </p>
      </div>

      <div className="p-3 rounded-lg bg-amber-50 dark:bg-amber-900/20 text-amber-800 dark:text-amber-200 text-sm">
        Dev mode — sign in with{' '}
        <code className="bg-amber-100 dark:bg-amber-900/40 px-1.5 py-0.5 rounded text-xs">admin@localhost</code>
        {' / '}
        <code className="bg-amber-100 dark:bg-amber-900/40 px-1.5 py-0.5 rounded text-xs">admin</code>
      </div>

      {error && (
        <div className="rounded-md p-4 bg-red-50 text-red-800 dark:bg-red-900/50 dark:text-red-200">
          <p className="text-sm">{error}</p>
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label htmlFor="email" className="block text-sm font-medium text-foreground mb-1">
            Email address
          </label>
          <input
            id="email"
            name="email"
            type="email"
            autoComplete="email"
            required
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className="w-full rounded-lg border border-input bg-card px-4 py-3 text-sm text-foreground placeholder-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
            placeholder="you@example.com"
          />
        </div>
        <div>
          <label htmlFor="password" className="block text-sm font-medium text-foreground mb-1">
            Password
          </label>
          <input
            id="password"
            name="password"
            type="password"
            autoComplete="current-password"
            required
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="w-full rounded-lg border border-input bg-card px-4 py-3 text-sm text-foreground placeholder-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
            placeholder="Enter your password"
          />
        </div>
        <button
          type="submit"
          disabled={isSigningIn}
          className="w-full rounded-lg bg-primary px-4 py-3 text-sm font-semibold text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isSigningIn ? 'Signing in...' : 'Sign In'}
        </button>
      </form>
    </div>
  );
}
