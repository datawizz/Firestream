/**
 * Server-side webhook utilities
 *
 * Functions for handling Stripe webhook events
 *
 * @packageDocumentation
 */

import Stripe from 'stripe';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '../types/database';
import { upsertProduct, upsertPrice, upsertSubscription } from './upsert';

/**
 * Set of Stripe events we care about
 */
const RELEVANT_EVENTS = new Set([
  'product.created',
  'product.updated',
  'price.created',
  'price.updated',
  'checkout.session.completed',
  'customer.subscription.created',
  'customer.subscription.updated',
  'customer.subscription.deleted',
]);

/**
 * Configuration for webhook handler
 */
export interface WebhookHandlerConfig {
  stripe: Stripe;
  supabase: SupabaseClient<Database>;
  webhookSecret: string;
  onSuccess?: (event: Stripe.Event) => void | Promise<void>;
  onError?: (error: Error, event?: Stripe.Event) => void | Promise<void>;
}

/**
 * Result of webhook processing
 */
export interface WebhookResult {
  received: boolean;
  error?: string;
}

/**
 * Create a webhook handler function
 *
 * This factory function creates a handler that can process Stripe webhook events.
 * It verifies the webhook signature, processes relevant events, and syncs data to Supabase.
 *
 * @param config - Webhook handler configuration
 * @returns Handler function that processes webhook events
 *
 * @example
 * ```typescript
 * const handleWebhook = createWebhookHandler({
 *   stripe,
 *   supabase,
 *   webhookSecret: process.env.STRIPE_WEBHOOK_SECRET,
 * });
 *
 * // In your API route:
 * const rawBody = await request.text();
 * const signature = request.headers.get('stripe-signature');
 * const result = await handleWebhook(rawBody, signature);
 * ```
 */
export function createWebhookHandler(config: WebhookHandlerConfig) {
  const { stripe, supabase, webhookSecret, onSuccess, onError } = config;

  return async (rawBody: string, signature: string | null): Promise<WebhookResult> => {
    let event: Stripe.Event;

    // Verify webhook signature
    try {
      if (!signature || !webhookSecret) {
        throw new Error('Missing webhook signature or secret');
      }

      event = stripe.webhooks.constructEvent(rawBody, signature, webhookSecret);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      const errorMessage = `Webhook signature verification failed: ${message}`;

      if (onError) {
        await onError(new Error(errorMessage));
      }

      return {
        received: false,
        error: errorMessage,
      };
    }

    // Process relevant events
    if (RELEVANT_EVENTS.has(event.type)) {
      try {
        await processWebhookEvent(stripe, supabase, event);

        if (onSuccess) {
          await onSuccess(event);
        }

        return { received: true };
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Unknown error';
        const errorMessage = `Webhook handler failed: ${message}`;

        console.error(`Error processing webhook event ${event.type}:`, error);

        if (onError) {
          await onError(new Error(errorMessage), event);
        }

        return {
          received: false,
          error: errorMessage,
        };
      }
    }

    // Event not relevant, but still acknowledge receipt
    return { received: true };
  };
}

/**
 * Process a webhook event
 *
 * @param stripe - Stripe client
 * @param supabase - Supabase admin client
 * @param event - Stripe event
 */
async function processWebhookEvent(
  stripe: Stripe,
  supabase: SupabaseClient<Database>,
  event: Stripe.Event
): Promise<void> {
  switch (event.type) {
    case 'product.created':
    case 'product.updated':
      await upsertProduct(supabase, event.data.object as Stripe.Product);
      break;

    case 'price.created':
    case 'price.updated':
      await upsertPrice(supabase, event.data.object as Stripe.Price);
      break;

    case 'customer.subscription.created':
    case 'customer.subscription.updated':
    case 'customer.subscription.deleted':
      const subscription = event.data.object as Stripe.Subscription;
      await upsertSubscription(supabase, stripe, {
        subscriptionId: subscription.id,
        customerId: subscription.customer as string,
        isCreateAction: false,
      });
      break;

    case 'checkout.session.completed':
      const checkoutSession = event.data.object as Stripe.Checkout.Session;

      if (checkoutSession.mode === 'subscription') {
        const subscriptionId = checkoutSession.subscription;
        await upsertSubscription(supabase, stripe, {
          subscriptionId: subscriptionId as string,
          customerId: checkoutSession.customer as string,
          isCreateAction: true,
        });
      }
      break;

    default:
      console.warn(`Unhandled event type: ${event.type}`);
  }
}

/**
 * Verify a webhook signature without processing the event
 *
 * Useful for testing webhook configuration or pre-validation.
 *
 * @param stripe - Stripe client
 * @param rawBody - Raw request body
 * @param signature - Stripe signature header
 * @param webhookSecret - Webhook secret
 * @returns Verified Stripe event
 * @throws Error if signature verification fails
 *
 * @example
 * ```typescript
 * const event = verifyWebhookSignature(
 *   stripe,
 *   rawBody,
 *   signature,
 *   webhookSecret
 * );
 * console.log('Verified event:', event.type);
 * ```
 */
export function verifyWebhookSignature(
  stripe: Stripe,
  rawBody: string,
  signature: string,
  webhookSecret: string
): Stripe.Event {
  return stripe.webhooks.constructEvent(rawBody, signature, webhookSecret);
}
