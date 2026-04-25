/**
 * Server-side upsert utilities
 *
 * Functions for syncing Stripe data to Supabase database
 *
 * @packageDocumentation
 */

import Stripe from 'stripe';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '../types/database';
import { toDateTime } from '../utils/to-datetime';

/**
 * Upsert a Stripe product to the database
 *
 * @param supabase - Supabase admin client
 * @param product - Stripe product object
 * @throws Error if the database operation fails
 *
 * @example
 * ```typescript
 * await upsertProduct(supabase, stripeProduct);
 * ```
 */
export async function upsertProduct(
  supabase: SupabaseClient<Database>,
  product: Stripe.Product
): Promise<void> {
  const productData: Database['public']['Tables']['products']['Insert'] = {
    id: product.id,
    active: product.active,
    name: product.name,
    description: product.description ?? null,
    image: product.images?.[0] ?? null,
    metadata: product.metadata,
  };

  const { error } = await supabase.from('products').upsert([productData]);

  if (error) {
    throw error;
  }

  console.info(`Product inserted/updated: ${product.id}`);
}

/**
 * Upsert a Stripe price to the database
 *
 * @param supabase - Supabase admin client
 * @param price - Stripe price object
 * @throws Error if the database operation fails
 *
 * @example
 * ```typescript
 * await upsertPrice(supabase, stripePrice);
 * ```
 */
export async function upsertPrice(
  supabase: SupabaseClient<Database>,
  price: Stripe.Price
): Promise<void> {
  const priceData: Database['public']['Tables']['prices']['Insert'] = {
    id: price.id,
    product_id: typeof price.product === 'string' ? price.product : '',
    active: price.active,
    currency: price.currency,
    description: price.nickname ?? null,
    type: price.type as Database['public']['Enums']['pricing_type'],
    unit_amount: price.unit_amount ?? null,
    interval: (price.recurring?.interval as Database['public']['Enums']['pricing_plan_interval']) ?? null,
    interval_count: price.recurring?.interval_count ?? null,
    trial_period_days: price.recurring?.trial_period_days ?? null,
    metadata: price.metadata,
  };

  const { error } = await supabase.from('prices').upsert([priceData]);

  if (error) {
    throw error;
  }

  console.info(`Price inserted/updated: ${price.id}`);
}

/**
 * Options for upserting a subscription
 */
export interface UpsertSubscriptionOptions {
  subscriptionId: string;
  customerId: string;
  isCreateAction?: boolean;
}

/**
 * Upsert a Stripe subscription to the database
 *
 * This function retrieves the subscription from Stripe, syncs it to the database,
 * and optionally copies billing details to the customer record.
 *
 * @param supabase - Supabase admin client
 * @param stripe - Stripe client
 * @param options - Subscription upsert options
 * @throws Error if the database operation fails
 *
 * @example
 * ```typescript
 * await upsertSubscription(supabase, stripe, {
 *   subscriptionId: 'sub_xxx',
 *   customerId: 'cus_xxx',
 *   isCreateAction: true,
 * });
 * ```
 */
export async function upsertSubscription(
  supabase: SupabaseClient<Database>,
  stripe: Stripe,
  options: UpsertSubscriptionOptions
): Promise<void> {
  const { subscriptionId, customerId, isCreateAction } = options;

  // Get customer's userId from mapping table
  const { data: customerData, error: noCustomerError } = await supabase
    .from('customers')
    .select('id')
    .eq('stripe_customer_id', customerId)
    .single();

  if (noCustomerError) {
    throw noCustomerError;
  }

  const { id: userId } = customerData;

  // Retrieve subscription from Stripe
  const subscription = await stripe.subscriptions.retrieve(subscriptionId, {
    expand: ['default_payment_method'],
  });

  // Prepare subscription data for database
  const subscriptionData: Database['public']['Tables']['subscriptions']['Insert'] = {
    id: subscription.id,
    user_id: userId,
    metadata: subscription.metadata,
    status: subscription.status as Database['public']['Enums']['subscription_status'],
    price_id: subscription.items.data[0].price.id,
    cancel_at_period_end: subscription.cancel_at_period_end,
    cancel_at: subscription.cancel_at ? toDateTime(subscription.cancel_at).toISOString() : null,
    canceled_at: subscription.canceled_at ? toDateTime(subscription.canceled_at).toISOString() : null,
    current_period_start: toDateTime((subscription as any).current_period_start).toISOString(),
    current_period_end: toDateTime((subscription as any).current_period_end).toISOString(),
    created: toDateTime(subscription.created).toISOString(),
    ended_at: subscription.ended_at ? toDateTime(subscription.ended_at).toISOString() : null,
    trial_start: subscription.trial_start ? toDateTime(subscription.trial_start).toISOString() : null,
    trial_end: subscription.trial_end ? toDateTime(subscription.trial_end).toISOString() : null,
  };

  const { error } = await supabase.from('subscriptions').upsert([subscriptionData]);

  if (error) {
    throw error;
  }

  console.info(`Inserted/updated subscription [${subscription.id}] for user [${userId}]`);

  // For a new subscription, copy billing details to customer
  if (isCreateAction && subscription.default_payment_method && userId) {
    await copyBillingDetailsToCustomer(
      supabase,
      stripe,
      userId,
      subscription.default_payment_method as Stripe.PaymentMethod
    );
  }
}

/**
 * Copy billing details from payment method to customer record
 *
 * @param supabase - Supabase admin client
 * @param stripe - Stripe client
 * @param userId - User ID
 * @param paymentMethod - Stripe payment method
 * @throws Error if the operation fails
 */
async function copyBillingDetailsToCustomer(
  supabase: SupabaseClient<Database>,
  stripe: Stripe,
  userId: string,
  paymentMethod: Stripe.PaymentMethod
): Promise<void> {
  const customer = paymentMethod.customer;

  if (typeof customer !== 'string') {
    throw new Error('Customer id not found');
  }

  const { name, phone, address } = paymentMethod.billing_details;

  if (!name || !phone || !address) {
    return;
  }

  // Update Stripe customer
  await stripe.customers.update(customer, {
    name,
    phone,
    address: address as Stripe.AddressParam,
  });

  // Update Supabase user record
  const { error } = await supabase
    .from('users')
    .update({
      billing_address: { ...address },
      payment_method: { ...paymentMethod[paymentMethod.type] },
    })
    .eq('id', userId);

  if (error) {
    throw error;
  }
}
