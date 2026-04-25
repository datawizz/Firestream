import { NextResponse } from 'next/server';
import Stripe from 'stripe';
import { createSupabaseAdminClient } from '@/lib/auth';
import { createWebhookHandler } from '@multi-platform-app/auth/server';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!, {
  apiVersion: '2024-11-20.acacia',
});

const webhookSecret = process.env.STRIPE_WEBHOOK_SECRET!;

// Create the webhook handler using the auth package utility
const handleWebhook = createWebhookHandler({
  stripe,
  supabase: createSupabaseAdminClient(),
  webhookSecret,
  onSuccess: async (event) => {
    console.log(`Successfully processed webhook event: ${event.type}`);
  },
  onError: async (error, event) => {
    console.error(`Webhook error: ${error.message}`, event ? `Event: ${event.type}` : '');
  },
});

export async function POST(request: Request) {
  try {
    const rawBody = await request.text();
    const signature = request.headers.get('stripe-signature');

    if (!signature) {
      return NextResponse.json(
        { error: 'Missing stripe-signature header' },
        { status: 400 }
      );
    }

    const result = await handleWebhook(rawBody, signature);

    if (result.received) {
      return NextResponse.json({ received: true });
    } else {
      return NextResponse.json(
        { error: result.error || 'Webhook processing failed' },
        { status: 400 }
      );
    }
  } catch (error) {
    console.error('Webhook route error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
