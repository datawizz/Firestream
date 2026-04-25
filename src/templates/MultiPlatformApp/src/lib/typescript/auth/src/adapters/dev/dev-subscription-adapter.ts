import type { ISubscriptionAdapter } from '../../interfaces/subscription-adapter';
import type { ActionResponse } from '../../types/auth';
import type {
  ProductWithPrices,
  SubscriptionWithProduct,
  Subscription,
  CreateCheckoutSessionOptions,
  CreatePortalSessionOptions,
  CheckoutSessionResponse,
  PortalSessionResponse,
} from '../../types/subscription';
import type { Customer, Invoice } from '../../types/customer';

const DEV_PREFIX = '[DEV AUTH]';

export class DevSubscriptionAdapter implements ISubscriptionAdapter {
  async initialize(): Promise<void> {}

  async getProducts(): Promise<ActionResponse<ProductWithPrices[]>> {
    return { data: [], error: null };
  }

  async getSubscription(
    _userId: string
  ): Promise<ActionResponse<SubscriptionWithProduct | null>> {
    return { data: null, error: null };
  }

  async getCustomer(_userId: string): Promise<ActionResponse<Customer | null>> {
    return { data: null, error: null };
  }

  async createCheckoutSession(
    _options: CreateCheckoutSessionOptions
  ): Promise<ActionResponse<CheckoutSessionResponse>> {
    console.warn(`${DEV_PREFIX} Checkout is not available in dev mode.`);
    return { data: null, error: new Error('Checkout not available in dev mode') };
  }

  async createPortalSession(
    _options: CreatePortalSessionOptions
  ): Promise<ActionResponse<PortalSessionResponse>> {
    console.warn(`${DEV_PREFIX} Billing portal is not available in dev mode.`);
    return { data: null, error: new Error('Billing portal not available in dev mode') };
  }

  async cancelSubscription(
    _subscriptionId: string
  ): Promise<ActionResponse<Subscription>> {
    return { data: null, error: new Error('Not available in dev mode') };
  }

  async resumeSubscription(
    _subscriptionId: string
  ): Promise<ActionResponse<Subscription>> {
    return { data: null, error: new Error('Not available in dev mode') };
  }

  async getInvoices(_customerId: string): Promise<ActionResponse<Invoice[]>> {
    return { data: [], error: null };
  }

  async hasFeatureAccess(_userId: string, _feature: string): Promise<boolean> {
    return true;
  }
}
