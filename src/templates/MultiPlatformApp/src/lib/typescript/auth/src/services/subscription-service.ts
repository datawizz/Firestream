/**
 * Subscription service
 *
 * High-level service for subscription operations that delegates to platform adapters
 *
 * @packageDocumentation
 */

import type { ISubscriptionAdapter } from '../interfaces/subscription-adapter';
import type {
  CreateCheckoutSessionOptions,
  CreatePortalSessionOptions,
} from '../types/subscription';

/**
 * Subscription service
 *
 * Provides a unified API for subscription and billing operations across platforms
 */
export class SubscriptionService {
  private adapter: ISubscriptionAdapter;

  constructor(adapter: ISubscriptionAdapter) {
    this.adapter = adapter;
  }

  /**
   * Initialize the subscription service
   */
  async initialize(): Promise<void> {
    await this.adapter.initialize();
  }

  /**
   * Get all available products with their prices
   */
  async getProducts() {
    return this.adapter.getProducts();
  }

  /**
   * Get the current user's subscription
   */
  async getSubscription(userId: string) {
    return this.adapter.getSubscription(userId);
  }

  /**
   * Get customer information
   */
  async getCustomer(userId: string) {
    return this.adapter.getCustomer(userId);
  }

  /**
   * Create a checkout session for subscribing to a plan
   */
  async createCheckoutSession(options: CreateCheckoutSessionOptions) {
    return this.adapter.createCheckoutSession(options);
  }

  /**
   * Create a portal session for managing subscriptions
   */
  async createPortalSession(options: CreatePortalSessionOptions) {
    return this.adapter.createPortalSession(options);
  }

  /**
   * Cancel a subscription
   */
  async cancelSubscription(subscriptionId: string) {
    return this.adapter.cancelSubscription(subscriptionId);
  }

  /**
   * Resume a canceled subscription
   */
  async resumeSubscription(subscriptionId: string) {
    return this.adapter.resumeSubscription(subscriptionId);
  }

  /**
   * Get invoices for the current user
   */
  async getInvoices(customerId: string) {
    return this.adapter.getInvoices(customerId);
  }

  /**
   * Check if user has access to a specific feature
   */
  async hasFeatureAccess(userId: string, feature: string) {
    return this.adapter.hasFeatureAccess(userId, feature);
  }

  /**
   * Check if a user has an active subscription
   *
   * A subscription is considered active if it has status 'trialing' or 'active'
   *
   * @param userId - The user ID to check
   * @returns True if user has an active or trialing subscription
   */
  async hasActiveSubscription(userId: string): Promise<boolean> {
    const result = await this.adapter.getSubscription(userId);

    if (result.error || !result.data) {
      return false;
    }

    const status = result.data.status;
    return status === 'trialing' || status === 'active';
  }

  /**
   * Check if a user is currently in a trial period
   *
   * @param userId - The user ID to check
   * @returns True if user has a trialing subscription
   */
  async isTrialing(userId: string): Promise<boolean> {
    const result = await this.adapter.getSubscription(userId);

    if (result.error || !result.data) {
      return false;
    }

    return result.data.status === 'trialing';
  }

  /**
   * Check if a user's subscription is past due
   *
   * @param userId - The user ID to check
   * @returns True if user has a past_due subscription
   */
  async isPastDue(userId: string): Promise<boolean> {
    const result = await this.adapter.getSubscription(userId);

    if (result.error || !result.data) {
      return false;
    }

    return result.data.status === 'past_due';
  }

  /**
   * Get subscription status for a user
   *
   * @param userId - The user ID to check
   * @returns Subscription status or null if no subscription exists
   */
  async getSubscriptionStatus(userId: string): Promise<string | null> {
    const result = await this.adapter.getSubscription(userId);

    if (result.error || !result.data) {
      return null;
    }

    return result.data.status;
  }
}
