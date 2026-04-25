/**
 * Server-side utilities for authentication and subscription management
 *
 * This module contains server-only code that should never be imported on the client.
 * It includes utilities for:
 * - Creating admin clients for Supabase and Stripe
 * - Handling webhook events from Stripe
 * - Creating checkout and customer portal sessions
 * - Syncing data between Stripe and Supabase
 * - Sending transactional emails
 *
 * @packageDocumentation
 */

// Admin client utilities
export * from './supabase-server';
export * from './stripe-server';

// Email utilities
export * from './email-server';

// Stripe integration utilities
export * from './checkout';
export * from './webhooks';
export * from './upsert';

// Email templates
export * from './templates/welcome';
