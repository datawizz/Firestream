/**
 * Server-side checkout utilities
 *
 * Functions for creating Stripe checkout and customer portal sessions
 *
 * @packageDocumentation
 */

import Stripe from 'stripe';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '../types/database';

/**
 * Options for creating a checkout session
 */
export interface CreateCheckoutSessionOptions {
  priceId: string;
  mode?: 'payment' | 'subscription';
  successUrl: string;
  cancelUrl: string;
  customerId?: string;
  trialPeriodDays?: number;
  allowPromotionCodes?: boolean;
  billingAddressCollection?: 'auto' | 'required';
  metadata?: Record<string, string>;
}

/**
 * Create a Stripe checkout session
 *
 * @param stripe - Stripe client
 * @param options - Checkout session options
 * @returns Checkout session with URL
 *
 * @example
 * ```typescript
 * const session = await createCheckoutSession(stripe, {
 *   priceId: 'price_xxx',
 *   customerId: 'cus_xxx',
 *   successUrl: 'https://example.com/success',
 *   cancelUrl: 'https://example.com/cancel',
 * });
 * console.log(session.url);
 * ```
 */
export async function createCheckoutSession(
  stripe: Stripe,
  options: CreateCheckoutSessionOptions
): Promise<Stripe.Checkout.Session> {
  const {
    priceId,
    mode,
    successUrl,
    cancelUrl,
    customerId,
    trialPeriodDays,
    allowPromotionCodes = true,
    billingAddressCollection = 'required',
    metadata,
  } = options;

  const checkoutSession = await stripe.checkout.sessions.create({
    payment_method_types: ['card'],
    billing_address_collection: billingAddressCollection,
    customer: customerId,
    customer_update: customerId
      ? {
          address: 'auto',
        }
      : undefined,
    line_items: [
      {
        price: priceId,
        quantity: 1,
      },
    ],
    mode: mode || 'subscription',
    allow_promotion_codes: allowPromotionCodes,
    subscription_data: trialPeriodDays
      ? {
          trial_period_days: trialPeriodDays,
        }
      : undefined,
    success_url: successUrl,
    cancel_url: cancelUrl,
    metadata,
  });

  if (!checkoutSession || !checkoutSession.url) {
    throw new Error('Failed to create checkout session');
  }

  return checkoutSession;
}

/**
 * Options for creating a customer portal session
 */
export interface CreatePortalSessionOptions {
  customerId: string;
  returnUrl: string;
}

/**
 * Create a Stripe customer portal session
 *
 * The customer portal allows users to manage their subscription,
 * update payment methods, and view billing history.
 *
 * @param stripe - Stripe client
 * @param options - Portal session options
 * @returns Portal session with URL
 *
 * @example
 * ```typescript
 * const session = await createCustomerPortalSession(stripe, {
 *   customerId: 'cus_xxx',
 *   returnUrl: 'https://example.com/account',
 * });
 * console.log(session.url);
 * ```
 */
export async function createCustomerPortalSession(
  stripe: Stripe,
  options: CreatePortalSessionOptions
): Promise<Stripe.BillingPortal.Session> {
  const { customerId, returnUrl } = options;

  const portalSession = await stripe.billingPortal.sessions.create({
    customer: customerId,
    return_url: returnUrl,
  });

  if (!portalSession || !portalSession.url) {
    throw new Error('Failed to create portal session');
  }

  return portalSession;
}

/**
 * Options for getting or creating a customer
 */
export interface GetOrCreateCustomerOptions {
  userId: string;
  email: string;
  name?: string;
  metadata?: Record<string, string>;
}

/**
 * Get or create a Stripe customer for a user
 *
 * This function checks if a customer record exists in the database.
 * If not, it creates a new Stripe customer and stores the mapping.
 *
 * @param supabase - Supabase admin client
 * @param stripe - Stripe client
 * @param options - Customer options
 * @returns Stripe customer ID
 *
 * @example
 * ```typescript
 * const customerId = await getOrCreateCustomer(supabase, stripe, {
 *   userId: 'user_xxx',
 *   email: 'user@example.com',
 * });
 * ```
 */
export async function getOrCreateCustomer(
  supabase: SupabaseClient<Database>,
  stripe: Stripe,
  options: GetOrCreateCustomerOptions
): Promise<string> {
  const { userId, email, name, metadata } = options;

  // Check if customer already exists
  const { data, error } = await supabase
    .from('customers')
    .select('stripe_customer_id')
    .eq('id', userId)
    .single();

  if (error || !data?.stripe_customer_id) {
    // No customer record found, create one
    const customerData: Stripe.CustomerCreateParams = {
      email,
      name,
      metadata: {
        userId,
        ...metadata,
      },
    };

    const customer = await stripe.customers.create(customerData);

    // Insert the customer ID into our Supabase mapping table
    const { error: supabaseError } = await supabase
      .from('customers')
      .insert([{ id: userId, stripe_customer_id: customer.id }]);

    if (supabaseError) {
      throw supabaseError;
    }

    console.info(`Created new Stripe customer: ${customer.id} for user: ${userId}`);

    return customer.id;
  }

  return data.stripe_customer_id;
}
