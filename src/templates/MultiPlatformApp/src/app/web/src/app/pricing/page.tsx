import { getProducts, getSession } from '@/lib/auth';
import { PricingCard } from './pricing-card';

export const metadata = {
  title: 'Pricing',
  description: 'Choose the plan that works for you',
};

export default async function PricingPage() {
  const [products, session] = await Promise.all([getProducts(), getSession()]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="text-center">
          <h1 className="text-4xl font-bold tracking-tight text-gray-900 dark:text-white sm:text-5xl md:text-6xl">
            Pricing Plans
          </h1>
          <p className="mx-auto mt-3 max-w-md text-base text-gray-500 dark:text-gray-400 sm:text-lg md:mt-5 md:max-w-3xl md:text-xl">
            Choose the plan that works best for you. All plans include a 14-day free trial.
          </p>
        </div>

        <div className="mt-16 grid gap-8 lg:grid-cols-3 lg:gap-x-8">
          {products.map((product) => (
            <PricingCard key={product.id} product={product} isAuthenticated={!!session} />
          ))}
        </div>

        {!session && (
          <div className="mt-16 text-center">
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Already have an account?{' '}
              <a
                href="/login"
                className="font-medium text-blue-600 hover:text-blue-500 dark:text-blue-400"
              >
                Sign in
              </a>
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
