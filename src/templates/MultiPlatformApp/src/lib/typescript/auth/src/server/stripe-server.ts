/**
 * Server-side Stripe utilities
 *
 * Utilities for Stripe operations in server environments
 *
 * @packageDocumentation
 */

import Stripe from 'stripe';

/**
 * Configuration for creating a Stripe client
 */
export interface StripeClientConfig {
  secretKey: string;
  apiVersion?: Stripe.LatestApiVersion;
  appInfo?: {
    name: string;
    version?: string;
    url?: string;
  };
}

/**
 * Create a Stripe admin client
 *
 * This client has full access to the Stripe API and should only be used in server environments.
 * Never expose the secret key to the client.
 *
 * @param config - Configuration for the Stripe client
 * @returns Stripe client instance
 *
 * @example
 * ```typescript
 * const stripe = createStripeClient({
 *   secretKey: process.env.STRIPE_SECRET_KEY,
 *   appInfo: {
 *     name: 'My App',
 *     version: '1.0.0',
 *   },
 * });
 * ```
 */
export function createStripeClient(config: StripeClientConfig): Stripe {
  return new Stripe(config.secretKey, {
    apiVersion: config.apiVersion,
    appInfo: config.appInfo
      ? {
          name: config.appInfo.name,
          version: config.appInfo.version,
          url: config.appInfo.url,
        }
      : undefined,
  });
}
