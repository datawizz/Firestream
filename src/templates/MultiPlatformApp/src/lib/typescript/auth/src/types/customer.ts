/**
 * Customer and billing type definitions
 *
 * Types for managing customer billing information and payment methods.
 *
 * @packageDocumentation
 */

import type { Database } from './database';

/**
 * Customer from database
 */
export type Customer = Database['public']['Tables']['customers']['Row'];

/**
 * Billing address structure
 */
export interface BillingAddress {
  city: string | null;
  country: string | null;
  line1: string | null;
  line2: string | null;
  postal_code: string | null;
  state: string | null;
}

/**
 * Payment method card details
 */
export interface PaymentMethodCard {
  brand: string;
  last4: string;
  exp_month: number;
  exp_year: number;
}

/**
 * Payment method structure
 */
export interface PaymentMethod {
  id: string;
  type: 'card' | 'bank_account' | 'other';
  card?: PaymentMethodCard;
  billing_details?: {
    address?: BillingAddress;
    email?: string;
    name?: string;
    phone?: string;
  };
}

/**
 * Customer with billing details
 */
export interface CustomerWithBilling extends Customer {
  billing_address?: BillingAddress;
  payment_method?: PaymentMethod;
}

/**
 * Invoice status
 */
export type InvoiceStatus = 'draft' | 'open' | 'paid' | 'uncollectible' | 'void';

/**
 * Invoice information
 */
export interface Invoice {
  id: string;
  amount_due: number;
  amount_paid: number;
  currency: string;
  status: InvoiceStatus;
  created: number;
  due_date: number | null;
  hosted_invoice_url: string | null;
  invoice_pdf: string | null;
  period_start: number;
  period_end: number;
}

/**
 * Payment intent status
 */
export type PaymentIntentStatus =
  | 'requires_payment_method'
  | 'requires_confirmation'
  | 'requires_action'
  | 'processing'
  | 'requires_capture'
  | 'canceled'
  | 'succeeded';

/**
 * Payment intent information
 */
export interface PaymentIntent {
  id: string;
  amount: number;
  currency: string;
  status: PaymentIntentStatus;
  client_secret: string;
  payment_method?: string;
}
