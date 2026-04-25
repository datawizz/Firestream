import { NextResponse } from 'next/server';
import { getSession } from '@/lib/auth';
import { createStripePortalSession } from '@multi-platform-app/auth/server';
import Stripe from 'stripe';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!, {
  apiVersion: '2024-11-20.acacia',
});

export async function GET(request: Request) {
  try {
    const session = await getSession();

    if (!session) {
      return NextResponse.redirect(new URL('/login', request.url));
    }

    // Create a Stripe billing portal session
    const { url } = await createStripePortalSession({
      stripe,
      customerId: session.user.id,
      returnUrl: new URL('/account', request.url).toString(),
    });

    return NextResponse.redirect(url);
  } catch (error) {
    console.error('Error creating portal session:', error);
    return NextResponse.redirect(new URL('/account?error=portal', request.url));
  }
}
