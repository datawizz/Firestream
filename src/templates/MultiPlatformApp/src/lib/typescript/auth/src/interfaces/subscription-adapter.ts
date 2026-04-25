/**
 * Subscription adapter interface
 *
 * Platform-agnostic interface for subscription and billing operations.
 * Implementations should handle platform-specific details.
 *
 * @packageDocumentation
 */

import type {
  Subscription,
  ProductWithPrices,
  SubscriptionWithProduct,
  CreateCheckoutSessionOptions,
  CreatePortalSessionOptions,
  CheckoutSessionResponse,
  PortalSessionResponse,
} from '../types/subscription';
import type { ActionResponse } from '../types/auth';
import type { Customer, Invoice } from '../types/customer';

/**
 * Base interface for subscription adapters
 *
 * Platform-specific adapters should implement this interface to provide
 * consistent subscription and billing functionality across platforms.
 */
export interface ISubscriptionAdapter {
  /**
   * Initialize the subscription adapter
   */
  initialize(): Promise<void>;

  /**
   * Get all available products with their prices
   */
  getProducts(): Promise<ActionResponse<ProductWithPrices[]>>;

  /**
   * Get the current user's subscription
   */
  getSubscription(userId: string): Promise<ActionResponse<SubscriptionWithProduct | null>>;

  /**
   * Get customer information
   */
  getCustomer(userId: string): Promise<ActionResponse<Customer | null>>;

  /**
   * Create a checkout session for subscribing to a plan
   */
  createCheckoutSession(
    options: CreateCheckoutSessionOptions
  ): Promise<ActionResponse<CheckoutSessionResponse>>;

  /**
   * Create a portal session for managing subscriptions
   */
  createPortalSession(
    options: CreatePortalSessionOptions
  ): Promise<ActionResponse<PortalSessionResponse>>;

  /**
   * Cancel a subscription
   */
  cancelSubscription(subscriptionId: string): Promise<ActionResponse<Subscription>>;

  /**
   * Resume a canceled subscription
   */
  resumeSubscription(subscriptionId: string): Promise<ActionResponse<Subscription>>;

  /**
   * Get invoices for the current user
   */
  getInvoices(customerId: string): Promise<ActionResponse<Invoice[]>>;

  /**
   * Check if user has access to a specific feature
   */
  hasFeatureAccess(userId: string, feature: string): Promise<boolean>;
}
