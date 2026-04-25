import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '@multi-platform-app/auth/hooks';
import { Button } from '@multi-platform-app/shared/components';
import { GoogleIcon, GitHubIcon } from '@multi-platform-app/shared/components';
import { openUrl } from '@tauri-apps/plugin-opener';
import { isDevMode } from '../../lib/auth';

export function Login() {
  const navigate = useNavigate();
  const { signIn, signInWithOAuth, signInWithEmail, isLoading, isSigningIn } = useAuth();
  const [email, setEmail] = useState(isDevMode ? 'admin@localhost' : '');
  const [password, setPassword] = useState(isDevMode ? 'admin' : '');
  const [useMagicLink, setUseMagicLink] = useState(false);
  const [magicLinkSent, setMagicLinkSent] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handlePasswordSignIn = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      const result = await signIn({ email, password });
      if (result.error) {
        setError(result.error.message);
      } else {
        navigate('/', { replace: true });
      }
    } catch {
      setError('Sign in failed');
    }
  };

  const handleMagicLinkSignIn = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.trim()) return;
    setError(null);
    try {
      await signInWithEmail({
        email,
        options: { emailRedirectTo: 'multiplatformapp://auth/callback' },
      });
      setMagicLinkSent(true);
    } catch {
      setError('Failed to send magic link');
    }
  };

  const handleOAuthSignIn = async (provider: 'google' | 'github') => {
    setError(null);
    try {
      const result = await signInWithOAuth({
        provider,
        options: {
          redirectTo: 'multiplatformapp://auth/callback',
          skipBrowserRedirect: true,
        },
      });
      if (result.data?.url) {
        await openUrl(result.data.url);
      }
    } catch {
      setError('OAuth sign in failed');
    }
  };

  if (magicLinkSent) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="max-w-md w-full p-8 border rounded-lg bg-card">
          <h1 className="text-2xl font-bold mb-4">Check Your Email</h1>
          <p className="text-muted-foreground mb-6">
            We've sent a magic link to <strong>{email}</strong>. Click the link in the email to
            sign in.
          </p>
          <Button
            onClick={() => {
              setMagicLinkSent(false);
              setEmail('');
            }}
            variant="outline"
            className="w-full"
          >
            Send Another Link
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-background">
      <div className="max-w-md w-full p-8 border rounded-lg bg-card">
        <h1 className="text-3xl font-bold mb-2">Welcome</h1>
        <p className="text-muted-foreground mb-8">Sign in to your account to continue</p>

        {isDevMode && (
          <div className="mb-6 p-3 rounded-lg bg-amber-50 dark:bg-amber-900/20 text-amber-800 dark:text-amber-200 text-sm">
            Dev mode — sign in with{' '}
            <code className="bg-amber-100 dark:bg-amber-900/40 px-1.5 py-0.5 rounded text-xs">admin@localhost</code>
            {' / '}
            <code className="bg-amber-100 dark:bg-amber-900/40 px-1.5 py-0.5 rounded text-xs">admin</code>
          </div>
        )}

        {error && (
          <div className="mb-6 p-3 rounded-lg bg-red-50 dark:bg-red-900/20 text-red-800 dark:text-red-200 text-sm">
            {error}
          </div>
        )}

        {/* OAuth Providers */}
        <div className="space-y-3 mb-8">
          <Button
            onClick={() => handleOAuthSignIn('google')}
            disabled={isLoading}
            className="w-full"
            variant="outline"
          >
            <GoogleIcon className="w-5 h-5 mr-2" />
            Continue with Google
          </Button>

          <Button
            onClick={() => handleOAuthSignIn('github')}
            disabled={isLoading}
            className="w-full"
            variant="outline"
          >
            <GitHubIcon className="w-5 h-5 mr-2" />
            Continue with GitHub
          </Button>
        </div>

        {/* Divider */}
        <div className="relative mb-8">
          <div className="absolute inset-0 flex items-center">
            <div className="w-full border-t"></div>
          </div>
          <div className="relative flex justify-center text-sm">
            <span className="px-2 bg-card text-muted-foreground">Or continue with email</span>
          </div>
        </div>

        {/* Email/Password Form */}
        <form onSubmit={useMagicLink ? handleMagicLinkSignIn : handlePasswordSignIn}>
          <div className="mb-4">
            <label htmlFor="email" className="block text-sm font-medium mb-2">
              Email address
            </label>
            <input
              id="email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="you@example.com"
              className="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
              required
            />
          </div>

          {!useMagicLink && (
            <div className="mb-6">
              <label htmlFor="password" className="block text-sm font-medium mb-2">
                Password
              </label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Enter your password"
                className="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
                required
              />
            </div>
          )}

          <Button
            type="submit"
            disabled={isSigningIn || isLoading || !email.trim()}
            className="w-full"
          >
            {isSigningIn ? 'Signing in...' : useMagicLink ? 'Send Magic Link' : 'Sign In'}
          </Button>
        </form>

        <p className="mt-4 text-center text-sm text-muted-foreground">
          {useMagicLink ? (
            <button
              type="button"
              onClick={() => setUseMagicLink(false)}
              className="underline hover:text-foreground"
            >
              Sign in with password instead
            </button>
          ) : (
            <button
              type="button"
              onClick={() => setUseMagicLink(true)}
              className="underline hover:text-foreground"
            >
              Send a magic link instead
            </button>
          )}
        </p>

        <p className="mt-6 text-center text-sm text-muted-foreground">
          By signing in, you agree to our Terms of Service and Privacy Policy.
        </p>
      </div>
    </div>
  );
}
