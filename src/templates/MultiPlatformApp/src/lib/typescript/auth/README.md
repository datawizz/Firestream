# @multi-platform-app/auth

A comprehensive authentication and subscription management package for multi-platform applications. Built with Supabase Auth and Stripe, this package provides a unified API that works seamlessly across web (Next.js, React) and desktop (Tauri) platforms.

## Overview

This package abstracts the complexity of authentication and subscription management behind a clean, platform-agnostic API. Whether you're building a web application with Next.js or a desktop application with Tauri, you get the same reliable authentication experience with minimal platform-specific code.

## Key Features

- **Cross-Platform**: Single API works on web and desktop with automatic platform detection
- **Authentication Methods**: Email/password, magic links, OAuth (GitHub, Google, and more)
- **Subscription Management**: Full Stripe integration for recurring billing and one-time payments
- **SSR Ready**: Built-in server-side rendering support for Next.js App Router
- **Type-Safe**: Complete TypeScript support with generated database types
- **State Management**: Zustand stores for reactive auth and subscription state
- **React Hooks**: Convenient hooks for common authentication patterns
- **Secure**: Cookie-based sessions (web) and IPC-based storage (Tauri)
- **Framework Agnostic**: Core adapters work with any JavaScript framework

## Installation

```bash
pnpm add @multi-platform-app/auth
```

### Peer Dependencies

```bash
pnpm add @tanstack/react-query react zustand
```

### Optional Dependencies

For Stripe integration:
```bash
pnpm add stripe
```

For Tauri desktop support:
```bash
pnpm add @tauri-apps/api
```

For email functionality:
```bash
pnpm add resend @react-email/components
```

## Quick Start

### 1. Environment Setup

Create a `.env.local` file:

```bash
# Supabase Configuration
NEXT_PUBLIC_SUPABASE_URL=https://your-project.supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key

# Stripe Configuration
STRIPE_SECRET_KEY=sk_test_...
NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY=pk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...

# Application URLs
NEXT_PUBLIC_SITE_URL=http://localhost:3000
```

### 2. Database Setup

Apply the included migrations to your Supabase project:

```bash
# From the auth package directory
cd src/lib/typescript/auth

# Apply migrations
supabase db push

# Generate TypeScript types
pnpm db:generate-types

# (Optional) Load test data
pnpm db:seed
```

### 3. Client-Side Usage (React)

```typescript
'use client';

import { useAuth, useSubscription } from '@multi-platform-app/auth/hooks';

export function AuthButton() {
  const { user, signOut, signInWithOAuth } = useAuth();
  const { subscription, isLoading } = useSubscription();

  if (user) {
    return (
      <div>
        <p>Welcome, {user.email}</p>
        {subscription && <p>Plan: {subscription.prices?.products?.name}</p>}
        <button onClick={signOut}>Sign Out</button>
      </div>
    );
  }

  return (
    <button onClick={() => signInWithOAuth({ provider: 'github' })}>
      Sign In with GitHub
    </button>
  );
}
```

### 4. Server-Side Usage (Next.js)

```typescript
import { cookies } from 'next/headers';
import { SupabaseWebAdapter } from '@multi-platform-app/auth/adapters/web';
import { createNextCookieHandler } from '@multi-platform-app/auth/adapters/web';

export default async function ProtectedPage() {
  const cookieStore = await cookies();
  const adapter = new SupabaseWebAdapter({
    supabase: {
      url: process.env.NEXT_PUBLIC_SUPABASE_URL!,
      anonKey: process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
      cookieHandler: createNextCookieHandler(cookieStore),
      isServer: true,
    },
  });

  const user = await adapter.getUser();

  if (!user) {
    redirect('/login');
  }

  return <div>Protected content for {user.email}</div>;
}
```

## Architecture

### Adapter Pattern

The package uses the adapter pattern to provide platform-specific implementations while maintaining a consistent API. This allows the same business logic to work across different platforms.

```
┌─────────────────────────────────────────┐
│         Application Layer               │
│  (Hooks, Components, Services)          │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│      Platform-Agnostic Layer            │
│  (Services, Stores, Interfaces)         │
└─────────────────┬───────────────────────┘
                  │
        ┌─────────┴─────────┐
        │                   │
┌───────▼────────┐  ┌──────▼─────────┐
│  Web Adapter   │  │  Tauri Adapter │
│  (SSR/CSR)     │  │  (IPC/Native)  │
└────────────────┘  └────────────────┘
```

### Package Exports

The package provides multiple entry points for different use cases:

- `@multi-platform-app/auth` - Main exports (types, services, hooks, stores)
- `@multi-platform-app/auth/types` - Type definitions only
- `@multi-platform-app/auth/services` - Service classes
- `@multi-platform-app/auth/hooks` - React hooks
- `@multi-platform-app/auth/stores` - Zustand stores
- `@multi-platform-app/auth/context` - React context providers
- `@multi-platform-app/auth/adapters/web` - Web platform adapter
- `@multi-platform-app/auth/adapters/tauri` - Tauri platform adapter
- `@multi-platform-app/auth/server` - Server-side utilities

## Core Concepts

### Authentication Flow

1. **Sign In**: User authenticates via email/password, magic link, or OAuth
2. **Session Management**: Secure session storage with automatic refresh
3. **State Sync**: Auth state synchronized across tabs/windows
4. **Sign Out**: Clean session termination with state cleanup

### Subscription Flow

1. **Product Selection**: User browses available products and prices
2. **Checkout**: Stripe Checkout session created with success/cancel URLs
3. **Webhook Processing**: Stripe webhooks sync subscription data to database
4. **Access Control**: Application checks subscription status for feature access
5. **Management**: Users can upgrade, downgrade, or cancel subscriptions

## Usage Examples

### Email Authentication

```typescript
import { useAuth } from '@multi-platform-app/auth/hooks';

function LoginForm() {
  const { signInWithEmail, signUpWithEmail } = useAuth();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

  const handleSignIn = async () => {
    const { error } = await signInWithEmail({ email, password });
    if (error) console.error(error);
  };

  const handleSignUp = async () => {
    const { error } = await signUpWithEmail({
      email,
      password,
      emailRedirectTo: `${window.location.origin}/auth/callback`
    });
    if (error) console.error(error);
  };

  return (
    <form>
      <input type="email" value={email} onChange={e => setEmail(e.target.value)} />
      <input type="password" value={password} onChange={e => setPassword(e.target.value)} />
      <button type="button" onClick={handleSignIn}>Sign In</button>
      <button type="button" onClick={handleSignUp}>Sign Up</button>
    </form>
  );
}
```

### OAuth Authentication

```typescript
import { useAuth } from '@multi-platform-app/auth/hooks';

function SocialLogin() {
  const { signInWithOAuth } = useAuth();

  return (
    <div>
      <button onClick={() => signInWithOAuth({ provider: 'github' })}>
        GitHub
      </button>
      <button onClick={() => signInWithOAuth({ provider: 'google' })}>
        Google
      </button>
    </div>
  );
}
```

### Magic Link Authentication

```typescript
import { useAuth } from '@multi-platform-app/auth/hooks';

function MagicLinkForm() {
  const { signInWithEmail } = useAuth();
  const [email, setEmail] = useState('');

  const handleMagicLink = async () => {
    const { error } = await signInWithEmail({
      email,
      options: {
        emailRedirectTo: `${window.location.origin}/auth/callback`,
      },
    });
    if (!error) {
      alert('Check your email for the magic link!');
    }
  };

  return (
    <form>
      <input type="email" value={email} onChange={e => setEmail(e.target.value)} />
      <button type="button" onClick={handleMagicLink}>Send Magic Link</button>
    </form>
  );
}
```

### Subscription Management

```typescript
import { useSubscription, useProducts } from '@multi-platform-app/auth/hooks';

function SubscriptionManager() {
  const { subscription, createCheckoutSession, createPortalSession } = useSubscription();
  const { data: products } = useProducts();

  const handleSubscribe = async (priceId: string) => {
    const { url } = await createCheckoutSession({
      priceId,
      successUrl: `${window.location.origin}/success`,
      cancelUrl: `${window.location.origin}/cancel`,
    });
    if (url) window.location.href = url;
  };

  const handleManage = async () => {
    const { url } = await createPortalSession({
      returnUrl: window.location.href,
    });
    if (url) window.location.href = url;
  };

  if (subscription) {
    return (
      <div>
        <h2>Current Plan: {subscription.prices?.products?.name}</h2>
        <p>Status: {subscription.status}</p>
        <button onClick={handleManage}>Manage Subscription</button>
      </div>
    );
  }

  return (
    <div>
      <h2>Available Plans</h2>
      {products?.map(product => (
        <div key={product.id}>
          <h3>{product.name}</h3>
          <p>{product.description}</p>
          {product.prices.map(price => (
            <button key={price.id} onClick={() => handleSubscribe(price.id)}>
              Subscribe - ${price.unit_amount / 100}/{price.interval}
            </button>
          ))}
        </div>
      ))}
    </div>
  );
}
```

### Server-Side User Access

```typescript
// app/api/protected/route.ts
import { createSupabaseServerClient } from '@multi-platform-app/auth/server';

export async function GET(request: Request) {
  const supabase = await createSupabaseServerClient();

  const { data: { user }, error } = await supabase.auth.getUser();

  if (error || !user) {
    return new Response('Unauthorized', { status: 401 });
  }

  return Response.json({ user });
}
```

### Stripe Webhook Handler

```typescript
// app/api/webhooks/stripe/route.ts
import { handleStripeWebhook } from '@multi-platform-app/auth/server';
import { headers } from 'next/headers';

export async function POST(request: Request) {
  const body = await request.text();
  const headersList = await headers();
  const signature = headersList.get('stripe-signature')!;

  try {
    await handleStripeWebhook({
      body,
      signature,
      webhookSecret: process.env.STRIPE_WEBHOOK_SECRET!,
    });
    return new Response('Webhook processed', { status: 200 });
  } catch (error) {
    console.error('Webhook error:', error);
    return new Response('Webhook failed', { status: 400 });
  }
}
```

### Middleware Authentication

```typescript
// middleware.ts
import { NextRequest, NextResponse } from 'next/server';
import {
  createMiddlewareSupabaseClient,
  createNextMiddlewareCookieHandler,
} from '@multi-platform-app/auth/adapters/web';

export async function middleware(request: NextRequest) {
  const response = NextResponse.next({ request });
  const cookieHandler = createNextMiddlewareCookieHandler(request, response);

  const client = createMiddlewareSupabaseClient({
    url: process.env.NEXT_PUBLIC_SUPABASE_URL!,
    anonKey: process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
    cookieHandler,
  });

  const { data: { user } } = await client.auth.getUser();

  // Redirect to login if not authenticated
  if (!user && request.nextUrl.pathname.startsWith('/dashboard')) {
    return NextResponse.redirect(new URL('/login', request.url));
  }

  return response;
}

export const config = {
  matcher: ['/dashboard/:path*', '/api/protected/:path*'],
};
```

## Platform-Specific Usage

### Web (Next.js App Router)

The web adapter uses `@supabase/ssr` for proper SSR support with cookie-based sessions.

**Client Component:**
```typescript
'use client';

import { createAuthAdapter } from '@multi-platform-app/auth/adapters';
import { createBrowserCookieHandler } from '@multi-platform-app/auth/adapters/web';

const adapter = createAuthAdapter({
  platform: 'web',
  web: {
    supabase: {
      url: process.env.NEXT_PUBLIC_SUPABASE_URL!,
      anonKey: process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
      cookieHandler: createBrowserCookieHandler(),
    },
  },
});
```

**Server Component:**
```typescript
import { cookies } from 'next/headers';
import { SupabaseWebAdapter } from '@multi-platform-app/auth/adapters/web';
import { createNextCookieHandler } from '@multi-platform-app/auth/adapters/web';

const cookieStore = await cookies();
const adapter = new SupabaseWebAdapter({
  supabase: {
    url: process.env.NEXT_PUBLIC_SUPABASE_URL!,
    anonKey: process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
    cookieHandler: createNextCookieHandler(cookieStore),
    isServer: true,
  },
});
```

### Desktop (Tauri)

The Tauri adapter uses IPC commands for secure session storage.

**Required Rust Commands:**
```rust
#[tauri::command]
async fn secure_storage_set(key: String, value: String) -> Result<(), String> {
    // Store encrypted data in OS keychain/credential manager
}

#[tauri::command]
async fn secure_storage_get(key: String) -> Result<Option<String>, String> {
    // Retrieve encrypted data
}

#[tauri::command]
async fn secure_storage_remove(key: String) -> Result<(), String> {
    // Delete encrypted data
}
```

**Frontend Usage:**
```typescript
import { SupabaseTauriAdapter } from '@multi-platform-app/auth/adapters/tauri';

const adapter = new SupabaseTauriAdapter({
  supabase: {
    url: import.meta.env.VITE_SUPABASE_URL,
    anonKey: import.meta.env.VITE_SUPABASE_ANON_KEY,
    // Use localStorage fallback in development
    useFallback: import.meta.env.DEV,
  },
});

await adapter.initialize();
```

## API Reference

### Hooks

#### `useAuth()`
Main authentication hook with all auth methods.

```typescript
const {
  user,              // Current user or null
  session,           // Current session or null
  isLoading,         // Loading state
  signInWithEmail,   // Email/password or magic link sign in
  signInWithPassword,// Email/password sign in
  signInWithOAuth,   // OAuth sign in (GitHub, Google, etc.)
  signUpWithEmail,   // Email/password sign up
  signOut,           // Sign out current user
  resetPassword,     // Send password reset email
  updateUser,        // Update user metadata
} = useAuth();
```

#### `useSession()`
Session management hook.

```typescript
const {
  session,           // Current session or null
  isLoading,         // Loading state
  refresh,           // Manually refresh session
} = useSession();
```

#### `useUser()`
User profile hook.

```typescript
const {
  user,              // Current user or null
  isLoading,         // Loading state
  updateProfile,     // Update user profile
} = useUser();
```

#### `useSubscription()`
Subscription management hook.

```typescript
const {
  subscription,              // Current subscription or null
  isLoading,                 // Loading state
  createCheckoutSession,     // Create Stripe checkout
  createPortalSession,       // Create Stripe portal
  cancelSubscription,        // Cancel subscription
} = useSubscription();
```

#### `useProducts()`
Product catalog hook.

```typescript
const {
  data: products,    // Available products with prices
  isLoading,         // Loading state
  error,             // Error if any
} = useProducts();
```

### Services

#### `AuthService`
Core authentication service (adapter-agnostic).

```typescript
import { AuthService } from '@multi-platform-app/auth/services';

const authService = new AuthService(adapter);

// Methods
await authService.signIn({ email, password });
await authService.signUp({ email, password });
await authService.signOut();
await authService.getSession();
await authService.getUser();
```

#### `SubscriptionService`
Subscription management service.

```typescript
import { SubscriptionService } from '@multi-platform-app/auth/services';

const subscriptionService = new SubscriptionService(adapter);

// Methods
await subscriptionService.getSubscription(userId);
await subscriptionService.createCheckoutSession({ priceId });
await subscriptionService.createPortalSession({ customerId });
```

### Stores

#### `useAuthStore`
Zustand store for auth state.

```typescript
import { useAuthStore } from '@multi-platform-app/auth/stores';

const user = useAuthStore(state => state.user);
const session = useAuthStore(state => state.session);
const setUser = useAuthStore(state => state.setUser);
```

#### `useSubscriptionStore`
Zustand store for subscription state.

```typescript
import { useSubscriptionStore } from '@multi-platform-app/auth/stores';

const subscription = useSubscriptionStore(state => state.subscription);
const setSubscription = useSubscriptionStore(state => state.setSubscription);
```

## Database Schema

The package includes migrations for the following tables:

- **users** - User profiles with billing information
- **customers** - Stripe customer ID mapping
- **products** - Stripe products synced via webhooks
- **prices** - Stripe prices synced via webhooks
- **subscriptions** - User subscriptions synced via webhooks

See `supabase/migrations/` for detailed schema definitions.

## Configuration

### Stripe Products

Use the included `stripe-fixtures.json` to create test products:

```bash
stripe fixtures stripe-fixtures.json
```

### Environment Variables

Required environment variables:

- `NEXT_PUBLIC_SUPABASE_URL` - Supabase project URL
- `NEXT_PUBLIC_SUPABASE_ANON_KEY` - Supabase anonymous key
- `SUPABASE_SERVICE_ROLE_KEY` - Supabase service role key (server-side only)
- `STRIPE_SECRET_KEY` - Stripe secret key (server-side only)
- `NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY` - Stripe publishable key
- `STRIPE_WEBHOOK_SECRET` - Stripe webhook signing secret
- `NEXT_PUBLIC_SITE_URL` - Application base URL

## Development

### Build

```bash
pnpm build
```

### Watch Mode

```bash
pnpm dev
```

### Type Check

```bash
pnpm typecheck
```

### Generate Database Types

```bash
pnpm db:generate-types
```

## Testing

The package includes comprehensive test coverage for adapters, services, and hooks.

```bash
pnpm test
```

## Troubleshooting

### Sessions Not Persisting

**Web**: Ensure cookies are enabled and the domain is configured correctly in Supabase settings.

**Tauri**: Ensure the required IPC commands are implemented in the Rust backend.

### OAuth Redirect Issues

Ensure the callback URL is configured in both:
1. Supabase Dashboard > Authentication > URL Configuration
2. OAuth provider settings (GitHub, Google, etc.)

### Stripe Webhook Failures

1. Verify webhook secret matches the one in Stripe Dashboard
2. Check that webhook endpoint is publicly accessible
3. Use Stripe CLI for local development: `stripe listen --forward-to localhost:3000/api/webhooks/stripe`

### Type Errors

Regenerate database types after schema changes:
```bash
pnpm db:generate-types
```

## Migration Guide

See [MIGRATION_FROM_STARTER.md](../../../../../../docs/MIGRATION_FROM_STARTER.md) for detailed migration instructions from the original Next.js Supabase Stripe Starter template.

## Related Documentation

- [Platform Adapters Guide](ADAPTERS.md)
- [Supabase Migrations](supabase/README.md)
- [Database Setup](supabase/README.md)

## License

Private package - not for public distribution.

## Support

For issues and questions, please contact the development team.
