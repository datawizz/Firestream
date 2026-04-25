import Link from 'next/link';
import { redirect } from 'next/navigation';
import { getSession, getSubscription, getProducts } from '@/lib/auth';
import { hasValidSupabaseConfig } from '@/lib/supabase-config';
import { signOut } from '@/app/(auth)/auth-actions';
import { DevAccountPage } from '../dev-account';

export const metadata = {
  title: 'Account',
  description: 'Manage your account and subscription',
};

export default async function AccountPage() {
  if (!hasValidSupabaseConfig()) {
    return <DevAccountPage />;
  }

  const [session, subscription, products] = await Promise.all([
    getSession(),
    getSubscription(),
    getProducts(),
  ]);

  if (!session) {
    redirect('/login');
  }

  // Find the product and price for the user's subscription
  let userProduct = null;
  let userPrice = null;

  if (subscription) {
    for (const product of products) {
      const price = product.prices?.find((p: any) => p.id === subscription.price_id);
      if (price) {
        userProduct = product;
        userPrice = price;
        break;
      }
    }
  }

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          Account
        </h1>
        <p className="mt-2 text-sm text-gray-500 dark:text-gray-400">
          Manage your account settings and subscription
        </p>
      </div>

      {/* User Info Card */}
      <div className="overflow-hidden rounded-lg bg-white shadow dark:bg-gray-800">
        <div className="border-b border-gray-200 px-6 py-4 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Profile Information
          </h2>
        </div>
        <div className="px-6 py-4">
          <dl className="space-y-4">
            <div>
              <dt className="text-sm font-medium text-gray-500 dark:text-gray-400">Email</dt>
              <dd className="mt-1 text-sm text-gray-900 dark:text-white">{session.user.email}</dd>
            </div>
            <div>
              <dt className="text-sm font-medium text-gray-500 dark:text-gray-400">User ID</dt>
              <dd className="mt-1 text-sm font-mono text-gray-900 dark:text-white">
                {session.user.id}
              </dd>
            </div>
          </dl>
        </div>
        <div className="border-t border-gray-200 px-6 py-4 dark:border-gray-700">
          <form action={signOut}>
            <button
              type="submit"
              className="rounded-md bg-red-600 px-4 py-2 text-sm font-semibold text-white hover:bg-red-700"
            >
              Sign Out
            </button>
          </form>
        </div>
      </div>

      {/* Subscription Card */}
      <div className="overflow-hidden rounded-lg bg-white shadow dark:bg-gray-800">
        <div className="border-b border-gray-200 px-6 py-4 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Your Plan</h2>
        </div>
        <div className="px-6 py-4">
          {subscription && userProduct && userPrice ? (
            <div className="space-y-4">
              <div>
                <h3 className="text-xl font-bold text-gray-900 dark:text-white">
                  {userProduct.name}
                </h3>
                <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
                  {userProduct.description}
                </p>
              </div>
              <div className="flex items-baseline gap-2">
                <span className="text-3xl font-bold text-gray-900 dark:text-white">
                  ${((userPrice as any).unit_amount / 100).toFixed(2)}
                </span>
                <span className="text-sm text-gray-500 dark:text-gray-400">
                  / {(userPrice as any).interval}
                </span>
              </div>
              <div>
                <dt className="text-sm font-medium text-gray-500 dark:text-gray-400">Status</dt>
                <dd className="mt-1">
                  <span className="inline-flex items-center rounded-full bg-green-100 px-3 py-1 text-xs font-medium text-green-800 dark:bg-green-900/50 dark:text-green-200">
                    {subscription.status}
                  </span>
                </dd>
              </div>
            </div>
          ) : (
            <div className="text-center py-8">
              <p className="text-gray-500 dark:text-gray-400">
                You don&apos;t have an active subscription
              </p>
            </div>
          )}
        </div>
        <div className="border-t border-gray-200 px-6 py-4 dark:border-gray-700">
          {subscription ? (
            <Link
              href="/manage-subscription"
              className="inline-flex items-center justify-center rounded-md bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700"
            >
              Manage Subscription
            </Link>
          ) : (
            <Link
              href="/pricing"
              className="inline-flex items-center justify-center rounded-md bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700"
            >
              Start a Subscription
            </Link>
          )}
        </div>
      </div>
    </div>
  );
}
