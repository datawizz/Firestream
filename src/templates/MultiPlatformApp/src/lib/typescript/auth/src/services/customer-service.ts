/**
 * Customer service
 *
 * High-level service for customer and billing operations that delegates to platform adapters
 *
 * @packageDocumentation
 */

import type { ISubscriptionAdapter } from '../interfaces/subscription-adapter';
import type { Customer, CustomerWithBilling } from '../types/customer';
import type { ActionResponse } from '../types/auth';

/**
 * Customer service
 *
 * Provides a unified API for customer and billing operations across platforms
 */
export class CustomerService {
  private adapter: ISubscriptionAdapter;

  constructor(adapter: ISubscriptionAdapter) {
    this.adapter = adapter;
  }

  /**
   * Initialize the customer service
   */
  async initialize(): Promise<void> {
    await this.adapter.initialize();
  }

  /**
   * Get customer information for a user
   *
   * @param userId - The user ID to get customer info for
   * @returns Customer record or null if not found
   */
  async getCustomer(userId: string): Promise<ActionResponse<Customer | null>> {
    return this.adapter.getCustomer(userId);
  }

  /**
   * Get Stripe customer ID for a user
   *
   * @param userId - The user ID to get Stripe customer ID for
   * @returns Stripe customer ID or null if not found
   */
  async getCustomerId(userId: string): Promise<string | null> {
    const result = await this.adapter.getCustomer(userId);

    if (result.error || !result.data) {
      return null;
    }

    return result.data.stripe_customer_id;
  }

  /**
   * Check if a user has a Stripe customer record
   *
   * @param userId - The user ID to check
   * @returns True if customer record exists with Stripe ID
   */
  async hasStripeCustomer(userId: string): Promise<boolean> {
    const customerId = await this.getCustomerId(userId);
    return customerId !== null;
  }

  /**
   * Get billing details for a user
   *
   * This would typically fetch from the users table which contains
   * billing_address and payment_method JSON fields
   *
   * @param userId - The user ID to get billing details for
   * @returns Customer with billing details
   */
  async getBillingDetails(userId: string): Promise<ActionResponse<CustomerWithBilling | null>> {
    // This is a placeholder - the actual implementation would need to:
    // 1. Query the users table for billing_address and payment_method
    // 2. Join with customers table for stripe_customer_id
    // 3. Parse the JSON fields into typed structures
    //
    // For now, we delegate to the adapter's getCustomer method
    // which may or may not include billing details depending on implementation
    const result = await this.adapter.getCustomer(userId);

    if (result.error || !result.data) {
      return result as ActionResponse<CustomerWithBilling | null>;
    }

    // Cast to CustomerWithBilling - actual billing details would need
    // to be fetched separately from the users table
    return {
      data: result.data as CustomerWithBilling,
      error: null,
    };
  }

  /**
   * Get invoices for a customer
   *
   * @param customerId - The Stripe customer ID
   * @returns List of invoices
   */
  async getInvoices(customerId: string) {
    return this.adapter.getInvoices(customerId);
  }
}
