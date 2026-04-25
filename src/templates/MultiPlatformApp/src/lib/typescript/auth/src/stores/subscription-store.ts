/**
 * Subscription state store
 *
 * Zustand store for managing subscription state
 *
 * @packageDocumentation
 */

import { create } from 'zustand';
import type {
  SubscriptionWithProduct,
  ProductWithPrices,
  SubscriptionStatus,
} from '../types/subscription';

/**
 * Subscription store state
 */
interface SubscriptionStore {
  subscription: SubscriptionWithProduct | null;
  products: ProductWithPrices[];
  isLoading: boolean;

  // Actions
  setSubscription: (subscription: SubscriptionWithProduct | null) => void;
  setProducts: (products: ProductWithPrices[]) => void;
  setLoading: (loading: boolean) => void;
  clear: () => void;

  // Derived state getters
  hasActiveSubscription: () => boolean;
  isTrialing: () => boolean;
  isPastDue: () => boolean;
  isCanceled: () => boolean;
  getStatus: () => SubscriptionStatus | null;
}

/**
 * Create the subscription store
 *
 * Note: We don't persist subscription data as it should be fetched fresh
 * from the server on each session
 */
export const useSubscriptionStore = create<SubscriptionStore>((set, get) => ({
  subscription: null,
  products: [],
  isLoading: false,

  setSubscription: (subscription) =>
    set({
      subscription,
    }),

  setProducts: (products) =>
    set({
      products,
    }),

  setLoading: (loading) =>
    set({
      isLoading: loading,
    }),

  clear: () =>
    set({
      subscription: null,
      products: [],
      isLoading: false,
    }),

  // Derived state
  hasActiveSubscription: () => {
    const { subscription } = get();
    if (!subscription) return false;
    return subscription.status === 'active' || subscription.status === 'trialing';
  },

  isTrialing: () => {
    const { subscription } = get();
    if (!subscription) return false;
    return subscription.status === 'trialing';
  },

  isPastDue: () => {
    const { subscription } = get();
    if (!subscription) return false;
    return subscription.status === 'past_due';
  },

  isCanceled: () => {
    const { subscription } = get();
    if (!subscription) return false;
    return subscription.status === 'canceled';
  },

  getStatus: () => {
    const { subscription } = get();
    return subscription?.status ?? null;
  },
}));
