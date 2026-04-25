'use client';

import { FormEvent, useState } from 'react';
import Link from 'next/link';
import type { ActionResponse } from './auth-actions';

const titleMap = {
  login: 'Sign in to your account',
  signup: 'Create your account',
} as const;

const subtitleMap = {
  login: "Don't have an account?",
  signup: 'Already have an account?',
} as const;

const linkMap = {
  login: { href: '/signup', text: 'Sign up' },
  signup: { href: '/login', text: 'Sign in' },
} as const;

interface AuthUIProps {
  mode: 'login' | 'signup';
  signInWithOAuth: (provider: 'github' | 'google') => Promise<ActionResponse>;
  signInWithEmail: (email: string) => Promise<ActionResponse>;
  signInWithPassword?: (email: string, password: string) => Promise<ActionResponse>;
}

export function AuthUI({ mode, signInWithOAuth, signInWithEmail, signInWithPassword }: AuthUIProps) {
  const [pending, setPending] = useState(false);
  const [useMagicLink, setUseMagicLink] = useState(false);
  const [message, setMessage] = useState<{ type: 'error' | 'success'; text: string } | null>(null);

  async function handleEmailPasswordSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPending(true);
    setMessage(null);

    const form = event.target as HTMLFormElement;
    const email = form['email'].value;
    const password = form['password'].value;

    if (!signInWithPassword) {
      setPending(false);
      return;
    }

    const response = await signInWithPassword(email, password);

    if (response?.error) {
      setMessage({
        type: 'error',
        text: 'Invalid email or password. Please try again.',
      });
    }

    setPending(false);
  }

  async function handleMagicLinkSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPending(true);
    setMessage(null);

    const form = event.target as HTMLFormElement;
    const email = form['email'].value;
    const response = await signInWithEmail(email);

    if (response?.error) {
      setMessage({
        type: 'error',
        text: 'An error occurred while authenticating. Please try again.',
      });
    } else {
      setMessage({
        type: 'success',
        text: `To continue, click the link in the email sent to: ${email}`,
      });
      form.reset();
    }

    setPending(false);
  }

  async function handleOAuthClick(provider: 'google' | 'github') {
    setPending(true);
    setMessage(null);
    const response = await signInWithOAuth(provider);

    if (response?.error) {
      setMessage({
        type: 'error',
        text: 'An error occurred while authenticating. Please try again.',
      });
      setPending(false);
    }
  }

  const showPasswordForm = mode === 'login' && signInWithPassword && !useMagicLink;

  return (
    <div className="w-full max-w-md space-y-8">
      <div className="text-center">
        <h2 className="text-3xl font-bold tracking-tight">{titleMap[mode]}</h2>
        <p className="mt-2 text-sm text-muted-foreground">
          {subtitleMap[mode]}{' '}
          <Link
            href={linkMap[mode].href}
            className="font-medium text-primary hover:text-primary/80"
          >
            {linkMap[mode].text}
          </Link>
        </p>
      </div>

      {message && (
        <div
          className={`rounded-md p-4 ${
            message.type === 'error'
              ? 'bg-red-50 text-red-800 dark:bg-red-900/50 dark:text-red-200'
              : 'bg-green-50 text-green-800 dark:bg-green-900/50 dark:text-green-200'
          }`}
        >
          <p className="text-sm">{message.text}</p>
        </div>
      )}

      <div className="space-y-4">
        <button
          onClick={() => handleOAuthClick('google')}
          disabled={pending}
          className="w-full flex items-center justify-center gap-3 rounded-lg border border-border bg-card px-4 py-3 text-sm font-semibold text-foreground shadow-sm hover:bg-accent disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <svg className="h-5 w-5" viewBox="0 0 24 24">
            <path
              fill="currentColor"
              d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
            />
            <path
              fill="currentColor"
              d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
            />
            <path
              fill="currentColor"
              d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
            />
            <path
              fill="currentColor"
              d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
            />
          </svg>
          Continue with Google
        </button>

        <button
          onClick={() => handleOAuthClick('github')}
          disabled={pending}
          className="w-full flex items-center justify-center gap-3 rounded-lg border border-border bg-card px-4 py-3 text-sm font-semibold text-foreground shadow-sm hover:bg-accent disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
            <path
              fillRule="evenodd"
              d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
              clipRule="evenodd"
            />
          </svg>
          Continue with GitHub
        </button>

        <div className="relative">
          <div className="absolute inset-0 flex items-center">
            <div className="w-full border-t border-border" />
          </div>
          <div className="relative flex justify-center text-sm">
            <span className="bg-background px-2 text-muted-foreground">
              Or continue with email
            </span>
          </div>
        </div>

        {showPasswordForm ? (
          <form onSubmit={handleEmailPasswordSubmit} className="space-y-4">
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
                className="w-full rounded-lg border border-input bg-card px-4 py-3 text-sm text-foreground placeholder-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                placeholder="Enter your password"
              />
            </div>
            <button
              type="submit"
              disabled={pending}
              className="w-full rounded-lg bg-primary px-4 py-3 text-sm font-semibold text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {pending ? 'Signing in...' : 'Sign In'}
            </button>
            <p className="text-center text-sm text-muted-foreground">
              <button
                type="button"
                onClick={() => setUseMagicLink(true)}
                className="underline hover:text-foreground"
              >
                Send a magic link instead
              </button>
            </p>
          </form>
        ) : (
          <form onSubmit={handleMagicLinkSubmit} className="space-y-4">
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
                autoFocus
                className="w-full rounded-lg border border-input bg-card px-4 py-3 text-sm text-foreground placeholder-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                placeholder="Enter your email"
              />
            </div>
            <button
              type="submit"
              disabled={pending}
              className="w-full rounded-lg bg-primary px-4 py-3 text-sm font-semibold text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {pending ? 'Sending...' : 'Send magic link'}
            </button>
            {mode === 'login' && signInWithPassword && (
              <p className="text-center text-sm text-muted-foreground">
                <button
                  type="button"
                  onClick={() => setUseMagicLink(false)}
                  className="underline hover:text-foreground"
                >
                  Sign in with password instead
                </button>
              </p>
            )}
          </form>
        )}
      </div>

      {mode === 'signup' && (
        <p className="text-center text-xs text-muted-foreground">
          By signing up, you agree to our{' '}
          <Link href="/terms" className="underline hover:text-foreground">
            Terms of Service
          </Link>{' '}
          and{' '}
          <Link href="/privacy" className="underline hover:text-foreground">
            Privacy Policy
          </Link>
          .
        </p>
      )}
    </div>
  );
}
