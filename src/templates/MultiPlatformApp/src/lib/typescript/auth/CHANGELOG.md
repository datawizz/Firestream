# Changelog

All notable changes to the `@multi-platform-app/auth` package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-12-22

### Added

#### Core Architecture
- Platform adapter pattern for multi-platform authentication support
- Support for web (Next.js, React) and desktop (Tauri) platforms
- Automatic platform detection with `detectPlatform()` utility
- Factory function `createAuthAdapter()` for easy adapter creation

#### Authentication Features
- Email/password authentication
- Magic link authentication (passwordless)
- OAuth authentication (GitHub, Google, and more)
- Session management with automatic refresh
- Password reset functionality
- User profile management
- Auth state change subscriptions

#### Subscription Management
- Stripe integration for subscription billing
- Product and pricing catalog management
- Checkout session creation
- Customer portal integration
- Webhook handling for Stripe events
- Subscription status tracking
- Support for one-time and recurring billing

#### Web Platform Support
- SSR-compatible Supabase client using `@supabase/ssr`
- Cookie-based session storage
- Next.js App Router integration
- Next.js middleware support
- Server component utilities
- Client component utilities
- Framework-agnostic cookie handlers

#### Tauri Platform Support
- IPC-based secure session storage
- Supabase client with custom storage callbacks
- Development fallback to localStorage
- Desktop-specific OAuth flows

#### React Integration
- `useAuth()` hook for authentication operations
- `useSession()` hook for session management
- `useUser()` hook for user profile access
- `useSubscription()` hook for subscription management
- `useProducts()` hook for product catalog
- Zustand stores for reactive state management
- React Query integration for data fetching and caching
- Auth context provider for dependency injection

#### Services Layer
- `AuthService` for authentication operations
- `SubscriptionService` for subscription management
- `CustomerService` for customer operations
- Platform-agnostic service interfaces
- Factory function `createAuthServices()` for service creation

#### Server-Side Utilities
- Admin Supabase client creation
- Admin Stripe client utilities
- Stripe webhook handler
- Checkout session creation
- Customer portal session creation
- Email sending with Resend
- React Email template support
- Data synchronization between Stripe and Supabase

#### Database
- Comprehensive database schema with migrations
- User profiles table with billing information
- Customers table for Stripe customer ID mapping
- Products and prices tables synced from Stripe
- Subscriptions table with status tracking
- Row Level Security (RLS) policies
- Database triggers for automatic profile creation
- Realtime subscriptions support
- Seed data for development

#### Type Safety
- Complete TypeScript types for all APIs
- Generated database types from Supabase schema
- Auth types (User, Session, etc.)
- Subscription types (Product, Price, Subscription)
- Configuration types for all adapters
- Type-safe cookie handlers
- Type-safe webhook handlers

#### Developer Experience
- Comprehensive JSDoc documentation
- Multiple package entry points for tree-shaking
- Vite build configuration with TypeScript declaration files
- Development mode with watch support
- Stripe fixtures for local testing
- Example implementations
- Migration guide from Next.js Supabase Stripe Starter

#### Configuration
- Environment variable support
- Platform-specific configuration
- Cookie configuration options
- Redirect URL configuration
- Stripe product fixtures
- Development and production modes

### Package Exports

- `@multi-platform-app/auth` - Main exports (all modules)
- `@multi-platform-app/auth/types` - Type definitions only
- `@multi-platform-app/auth/services` - Service classes
- `@multi-platform-app/auth/hooks` - React hooks
- `@multi-platform-app/auth/stores` - Zustand stores
- `@multi-platform-app/auth/context` - React context providers
- `@multi-platform-app/auth/adapters` - Adapter factory and types
- `@multi-platform-app/auth/adapters/web` - Web platform adapter
- `@multi-platform-app/auth/adapters/tauri` - Tauri platform adapter
- `@multi-platform-app/auth/server` - Server-side utilities

### Documentation

- Comprehensive README with usage examples
- Platform adapters architecture guide (ADAPTERS.md)
- Migration guide from Next.js Supabase Stripe Starter
- Database setup guide
- Stripe integration guide
- API reference documentation
- Troubleshooting guide

### Dependencies

#### Core
- `@supabase/supabase-js@^2.45.0` - Supabase client
- `@supabase/ssr@^0.5.0` - SSR support for Supabase

#### Peer Dependencies
- `@tanstack/react-query@^5.0.0` - Data fetching and caching
- `react@^18.0.0 || ^19.0.0` - React framework
- `zustand@^5.0.0` - State management

#### Optional Dependencies
- `@tauri-apps/api@^2.0.0` - Tauri desktop integration
- `stripe@^14.0.0` - Stripe payment processing
- `resend@^3.0.0` - Email service
- `@react-email/components@^0.0.25` - Email template components
- `@react-email/tailwind@^0.1.0` - Email template styling

### Breaking Changes

None (initial release)

### Security

- Cookie-based session storage with httpOnly support
- IPC-based secure storage for Tauri
- Row Level Security (RLS) for database access
- Service role key isolation for server-side operations
- Webhook signature verification
- CSRF protection via Supabase session management

### Performance

- React Query caching for reduced API calls
- Lazy loading of platform-specific adapters
- Tree-shakable package exports
- Optimized bundle size with proper code splitting

### Known Limitations

- Tauri IPC commands must be implemented in Rust backend
- OAuth flows require proper redirect URL configuration
- Stripe webhooks require publicly accessible endpoint
- Email functionality requires Resend API key

### Migration Notes

This package is a refactored and enhanced version of the Next.js Supabase Stripe Starter template. See [MIGRATION_FROM_STARTER.md](../../../../../docs/MIGRATION_FROM_STARTER.md) for detailed migration instructions.

---

[0.1.0]: https://github.com/yourorg/yourrepo/releases/tag/auth-v0.1.0
