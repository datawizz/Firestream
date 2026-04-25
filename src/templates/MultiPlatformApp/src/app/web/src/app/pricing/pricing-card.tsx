'use client';

import { useState } from 'react';
import Link from 'next/link';
import { createCheckout } from './actions';

interface Price {
  id: string;
  product_id: string;
  active: boolean;
  currency: string;
  type: 'one_time' | 'recurring';
  unit_amount: number;
  interval: 'month' | 'year' | null;
  interval_count: number;
  trial_period_days: number | null;
}

interface Product {
  id: string;
  name: string;
  description: string;
  active: boolean;
  metadata: Record<string, any>;
  prices: Price[];
}

interface PricingCardProps {
  product: Product;
  isAuthenticated: boolean;
}

export function PricingCard({ product, isAuthenticated }: PricingCardProps) {
  const [billingInterval, setBillingInterval] = useState<'month' | 'year'>('month');
  const [loading, setLoading] = useState(false);

  // Get the current price based on billing interval
  const currentPrice =
    product.prices.find((price) => price.interval === billingInterval) || product.prices[0];

  const monthlyPrice = product.prices.find((price) => price.interval === 'month');
  const yearlyPrice = product.prices.find((price) => price.interval === 'year');

  const hasMultiplePrices = product.prices.length > 1;

  async function handleSubscribe() {
    if (!currentPrice) return;

    setLoading(true);
    try {
      const response = await createCheckout(currentPrice.id);
      if (response.url) {
        window.location.href = response.url;
      } else if (response.error) {
        alert(response.error);
      }
    } catch (error) {
      console.error('Checkout error:', error);
      alert('An error occurred. Please try again.');
    } finally {
      setLoading(false);
    }
  }

  // Parse features from metadata or use defaults
  const features = product.metadata?.features
    ? JSON.parse(product.metadata.features)
    : ['Feature 1', 'Feature 2', 'Feature 3'];

  const isPopular = product.metadata?.popular === 'true';

  return (
    <div
      className={`relative rounded-2xl ${
        isPopular
          ? 'border-2 border-blue-600 shadow-xl dark:border-blue-500'
          : 'border border-gray-200 dark:border-gray-700'
      } bg-white p-8 dark:bg-gray-800`}
    >
      {isPopular && (
        <div className="absolute -top-4 left-1/2 -translate-x-1/2">
          <span className="inline-flex rounded-full bg-blue-600 px-4 py-1 text-sm font-semibold text-white">
            Most Popular
          </span>
        </div>
      )}

      <div className="mb-8">
        <h3 className="text-2xl font-bold text-gray-900 dark:text-white">{product.name}</h3>
        <p className="mt-4 text-sm text-gray-500 dark:text-gray-400">{product.description}</p>
      </div>

      {hasMultiplePrices && monthlyPrice && yearlyPrice && (
        <div className="mb-6 flex rounded-lg bg-gray-100 p-1 dark:bg-gray-700">
          <button
            onClick={() => setBillingInterval('month')}
            className={`flex-1 rounded-md px-4 py-2 text-sm font-medium transition-colors ${
              billingInterval === 'month'
                ? 'bg-white text-gray-900 shadow dark:bg-gray-800 dark:text-white'
                : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white'
            }`}
          >
            Monthly
          </button>
          <button
            onClick={() => setBillingInterval('year')}
            className={`flex-1 rounded-md px-4 py-2 text-sm font-medium transition-colors ${
              billingInterval === 'year'
                ? 'bg-white text-gray-900 shadow dark:bg-gray-800 dark:text-white'
                : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white'
            }`}
          >
            Yearly
          </button>
        </div>
      )}

      {currentPrice ? (
        <div className="mb-8">
          <div className="flex items-baseline">
            <span className="text-5xl font-bold tracking-tight text-gray-900 dark:text-white">
              ${(currentPrice.unit_amount / 100).toFixed(0)}
            </span>
            {currentPrice.interval && (
              <span className="ml-2 text-base text-gray-500 dark:text-gray-400">
                /{currentPrice.interval}
              </span>
            )}
          </div>
          {billingInterval === 'year' && yearlyPrice && monthlyPrice && (
            <p className="mt-2 text-sm text-gray-500 dark:text-gray-400">
              Save ${(((monthlyPrice.unit_amount * 12 - yearlyPrice.unit_amount) / 100).toFixed(0))} per year
            </p>
          )}
        </div>
      ) : (
        <div className="mb-8">
          <div className="text-5xl font-bold tracking-tight text-gray-900 dark:text-white">
            Custom
          </div>
          <p className="mt-2 text-sm text-gray-500 dark:text-gray-400">Contact us for pricing</p>
        </div>
      )}

      <ul className="mb-8 space-y-4">
        {features.map((feature: string, index: number) => (
          <li key={index} className="flex items-start">
            <svg
              className="h-6 w-6 flex-shrink-0 text-green-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M5 13l4 4L19 7"
              />
            </svg>
            <span className="ml-3 text-sm text-gray-700 dark:text-gray-300">{feature}</span>
          </li>
        ))}
      </ul>

      {currentPrice ? (
        isAuthenticated ? (
          <button
            onClick={handleSubscribe}
            disabled={loading}
            className={`w-full rounded-lg px-6 py-3 text-center text-sm font-semibold transition-colors ${
              isPopular
                ? 'bg-blue-600 text-white hover:bg-blue-700 disabled:bg-blue-400'
                : 'bg-gray-900 text-white hover:bg-gray-800 dark:bg-white dark:text-gray-900 dark:hover:bg-gray-100 disabled:bg-gray-400'
            } disabled:cursor-not-allowed`}
          >
            {loading ? 'Loading...' : 'Subscribe'}
          </button>
        ) : (
          <Link
            href="/signup"
            className={`block w-full rounded-lg px-6 py-3 text-center text-sm font-semibold transition-colors ${
              isPopular
                ? 'bg-blue-600 text-white hover:bg-blue-700'
                : 'bg-gray-900 text-white hover:bg-gray-800 dark:bg-white dark:text-gray-900 dark:hover:bg-gray-100'
            }`}
          >
            Get Started
          </Link>
        )
      ) : (
        <Link
          href="/contact"
          className="block w-full rounded-lg border-2 border-gray-900 px-6 py-3 text-center text-sm font-semibold text-gray-900 transition-colors hover:bg-gray-900 hover:text-white dark:border-white dark:text-white dark:hover:bg-white dark:hover:text-gray-900"
        >
          Contact Sales
        </Link>
      )}
    </div>
  );
}
