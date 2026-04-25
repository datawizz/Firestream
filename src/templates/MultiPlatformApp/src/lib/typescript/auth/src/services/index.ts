/**
 * Core authentication and subscription services
 *
 * @packageDocumentation
 */

import type { IAuthAdapter } from '../interfaces/auth-adapter';
import type { ISubscriptionAdapter } from '../interfaces/subscription-adapter';
import { AuthService } from './auth-service';
import { SubscriptionService } from './subscription-service';
import { CustomerService } from './customer-service';

export * from './auth-service';
export * from './subscription-service';
export * from './customer-service';

/**
 * Configuration for creating auth services
 */
export interface AuthServicesConfig {
  authAdapter: IAuthAdapter;
  subscriptionAdapter: ISubscriptionAdapter;
}

/**
 * Collection of all auth-related services
 */
export interface AuthServices {
  auth: AuthService;
  subscription: SubscriptionService;
  customer: CustomerService;
}

/**
 * Create all auth-related services from adapters
 *
 * Factory function that creates and returns all authentication,
 * subscription, and customer services configured with the provided adapters.
 *
 * @param config - Configuration with auth and subscription adapters
 * @returns Object containing initialized auth, subscription, and customer services
 *
 * @example
 * ```typescript
 * import { createAuthServices } from '@/lib/auth/services';
 * import { SupabaseAuthAdapter, SupabaseSubscriptionAdapter } from '@/lib/auth/adapters';
 *
 * const services = createAuthServices({
 *   authAdapter: new SupabaseAuthAdapter(supabase),
 *   subscriptionAdapter: new SupabaseSubscriptionAdapter(supabase),
 * });
 *
 * // Use the services
 * await services.auth.signIn('github');
 * const hasSubscription = await services.subscription.hasActiveSubscription(userId);
 * const customerId = await services.customer.getCustomerId(userId);
 * ```
 */
export function createAuthServices(config: AuthServicesConfig): AuthServices {
  const { authAdapter, subscriptionAdapter } = config;

  return {
    auth: new AuthService(authAdapter),
    subscription: new SubscriptionService(subscriptionAdapter),
    customer: new CustomerService(subscriptionAdapter),
  };
}
