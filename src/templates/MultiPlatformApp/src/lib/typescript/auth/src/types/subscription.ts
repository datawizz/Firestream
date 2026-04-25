/**
 * Subscription and pricing type definitions
 *
 * Types for managing user subscriptions, products, and pricing plans.
 *
 * @packageDocumentation
 */

import type { Database } from './database';

/**
 * Billing interval for subscriptions
 */
export type BillingInterval = 'day' | 'week' | 'month' | 'year';

/**
 * Subscription status from database enum
 */
export type SubscriptionStatus = Database['public']['Enums']['subscription_status'];

/**
 * Pricing type (one-time or recurring)
 */
export type PricingType = Database['public']['Enums']['pricing_type'];

/**
 * Subscription from database
 */
export type Subscription = Database['public']['Tables']['subscriptions']['Row'];

/**
 * Product from database
 */
export type Product = Database['public']['Tables']['products']['Row'];

/**
 * Price from database
 */
export type Price = Database['public']['Tables']['prices']['Row'];

/**
 * Product with associated prices
 */
export type ProductWithPrices = Product & {
  prices: Price[];
};

/**
 * Price with associated product
 */
export type PriceWithProduct = Price & {
  products: Product | null;
};

/**
 * Subscription with associated price and product information
 */
export type SubscriptionWithProduct = Subscription & {
  prices: PriceWithProduct | null;
};

/**
 * Subscription plan features
 */
export interface SubscriptionFeatures {
  [key: string]: boolean | number | string;
}

/**
 * Subscription limits
 */
export interface SubscriptionLimits {
  maxUsers?: number;
  maxProjects?: number;
  maxStorage?: number;
  maxApiCalls?: number;
  [key: string]: number | undefined;
}

/**
 * Subscription tier information
 */
export interface SubscriptionTier {
  id: string;
  name: string;
  description: string;
  features: SubscriptionFeatures;
  limits: SubscriptionLimits;
}

/**
 * Subscription state for the current user
 */
export interface SubscriptionState {
  subscription: SubscriptionWithProduct | null;
  products: ProductWithPrices[];
  isLoading: boolean;
  isSubscribed: boolean;
  isTrialing: boolean;
  isPastDue: boolean;
  currentTier: SubscriptionTier | null;
}

/**
 * Options for creating a checkout session
 */
export interface CreateCheckoutSessionOptions {
  priceId: string;
  successUrl?: string;
  cancelUrl?: string;
  trialPeriodDays?: number;
  metadata?: Record<string, string>;
}

/**
 * Options for creating a portal session
 */
export interface CreatePortalSessionOptions {
  returnUrl?: string;
}

/**
 * Checkout session response
 */
export interface CheckoutSessionResponse {
  sessionId: string;
  url: string;
}

/**
 * Portal session response
 */
export interface PortalSessionResponse {
  url: string;
}
