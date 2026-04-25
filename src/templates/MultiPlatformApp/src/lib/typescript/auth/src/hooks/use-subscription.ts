/**
 * useSubscription hook
 *
 * React hook for accessing subscription state and operations
 *
 * @packageDocumentation
 */

import { useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useSubscriptionStore } from '../stores/subscription-store';
import { useSubscriptionService } from '../context/auth-provider';
import { useAuthStore } from '../stores/auth-store';
import type {
  SubscriptionWithProduct,
  CreateCheckoutSessionOptions,
  CreatePortalSessionOptions,
} from '../types/subscription';

/**
 * Subscription hook
 *
 * Provides subscription state and billing operations using React Query and Zustand.
 * Automatically syncs subscription data with local store.
 *
 * @example
 * ```tsx
 * function SubscriptionStatus() {
 *   const {
 *     subscription,
 *     hasActiveSubscription,
 *     isTrialing,
 *     createCheckout,
 *     createPortal,
 *     isLoading
 *   } = useSubscription();
 *
 *   const handleUpgrade = async () => {
 *     const result = await createCheckout({
 *       priceId: 'price_123',
 *       successUrl: '/success',
 *       cancelUrl: '/pricing'
 *     });
 *     if (result.data) {
 *       window.location.href = result.data.url;
 *     }
 *   };
 *
 *   return (
 *     <div>
 *       {hasActiveSubscription ? (
 *         <div>
 *           <p>Status: {subscription?.status}</p>
 *           {isTrialing && <p>Trial ends soon!</p>}
 *           <button onClick={() => createPortal()}>Manage Subscription</button>
 *         </div>
 *       ) : (
 *         <button onClick={handleUpgrade}>Upgrade Now</button>
 *       )}
 *     </div>
 *   );
 * }
 * ```
 */
export function useSubscription() {
  const subscriptionService = useSubscriptionService();
  const queryClient = useQueryClient();
  const { user } = useAuthStore();
  const {
    subscription: storedSubscription,
    setSubscription,
    hasActiveSubscription: hasActive,
    isTrialing: isInTrial,
    isPastDue: isOverdue,
    isCanceled: isCancelled,
    getStatus,
  } = useSubscriptionStore();

  // Fetch subscription data
  const {
    data: subscription,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['subscription', user?.id],
    queryFn: async () => {
      if (!user?.id) return null;
      const result = await subscriptionService.getSubscription(user.id);
      return result.data;
    },
    enabled: !!user?.id,
    staleTime: 1000 * 60 * 5, // 5 minutes
    retry: false,
  });

  // Sync subscription to store
  useEffect(() => {
    if (subscription !== undefined) {
      setSubscription(subscription);
    }
  }, [subscription, setSubscription]);

  // Create checkout session mutation
  const createCheckoutMutation = useMutation({
    mutationFn: async (options: CreateCheckoutSessionOptions) => {
      return subscriptionService.createCheckoutSession(options);
    },
  });

  // Create portal session mutation
  const createPortalMutation = useMutation({
    mutationFn: async (options: CreatePortalSessionOptions = {}) => {
      return subscriptionService.createPortalSession(options);
    },
  });

  // Cancel subscription mutation
  const cancelMutation = useMutation({
    mutationFn: async (subscriptionId: string) => {
      return subscriptionService.cancelSubscription(subscriptionId);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['subscription', user?.id] });
    },
  });

  // Resume subscription mutation
  const resumeMutation = useMutation({
    mutationFn: async (subscriptionId: string) => {
      return subscriptionService.resumeSubscription(subscriptionId);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['subscription', user?.id] });
    },
  });

  // Convenience methods
  const createCheckout = async (options: CreateCheckoutSessionOptions) => {
    return createCheckoutMutation.mutateAsync(options);
  };

  const createPortal = async (options: CreatePortalSessionOptions = {}) => {
    return createPortalMutation.mutateAsync(options);
  };

  const cancelSubscription = async () => {
    if (!subscription?.id) {
      throw new Error('No active subscription to cancel');
    }
    return cancelMutation.mutateAsync(subscription.id);
  };

  const resumeSubscription = async () => {
    if (!subscription?.id) {
      throw new Error('No subscription to resume');
    }
    return resumeMutation.mutateAsync(subscription.id);
  };

  return {
    // State
    subscription: (subscription ?? storedSubscription) as SubscriptionWithProduct | null,
    isLoading,
    error: error as Error | null,

    // Derived state
    hasActiveSubscription: hasActive(),
    isTrialing: isInTrial(),
    isPastDue: isOverdue(),
    isCanceled: isCancelled(),
    status: getStatus(),

    // Methods
    createCheckout,
    createPortal,
    cancelSubscription,
    resumeSubscription,
    refetch,

    // Mutation states
    isCreatingCheckout: createCheckoutMutation.isPending,
    isCreatingPortal: createPortalMutation.isPending,
    isCanceling: cancelMutation.isPending,
    isResuming: resumeMutation.isPending,
  };
}
