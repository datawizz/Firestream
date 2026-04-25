'use server';

import { redirect } from 'next/navigation';
import { getSession } from '@/lib/auth';
import { createStripeCheckoutSession } from '@multi-platform-app/auth/server';
import { getURL } from '@/lib/auth';
import Stripe from 'stripe';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!, {
  apiVersion: '2024-11-20.acacia',
});

export async function createCheckout(priceId: string): Promise<{ url?: string; error?: string }> {
  try {
    const session = await getSession();

    if (!session) {
      return { error: 'You must be logged in to subscribe' };
    }

    // Create a Stripe checkout session
    const checkoutSession = await createStripeCheckoutSession({
      stripe,
      priceId,
      customerId: session.user.id,
      customerEmail: session.user.email!,
      successUrl: getURL('/account'),
      cancelUrl: getURL('/pricing'),
    });

    if (!checkoutSession.url) {
      return { error: 'Failed to create checkout session' };
    }

    return { url: checkoutSession.url };
  } catch (error) {
    console.error('Checkout error:', error);
    return { error: 'An error occurred during checkout' };
  }
}
