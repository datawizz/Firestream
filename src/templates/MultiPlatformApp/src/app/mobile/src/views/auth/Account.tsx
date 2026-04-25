import { useAuth, useSubscription } from '@multi-platform-app/auth/hooks';
import { Button } from '@multi-platform-app/shared/components';
import { openUrl } from '@tauri-apps/plugin-opener';
import { MobileLayout } from '../../components/MobileLayout';

export function Account() {
  const { user, signOut, isLoading: authLoading } = useAuth();
  const { subscription, createCheckout, createPortal, isLoading: subLoading } =
    useSubscription();

  const handleSignOut = async () => {
    try {
      await signOut();
    } catch (error) {
      console.error('Sign out failed:', error);
    }
  };

  const handleSubscribe = async () => {
    try {
      const result = await createCheckout({
        priceId: 'price_xxx',
        successUrl: 'multiplatformapp://auth/callback?checkout=success',
        cancelUrl: 'multiplatformapp://auth/callback?checkout=cancel',
      });

      if (result.data?.url) {
        await openUrl(result.data.url);
      }
    } catch (error) {
      console.error('Failed to create checkout session:', error);
    }
  };

  const handleManageBilling = async () => {
    try {
      const result = await createPortal({
        returnUrl: 'multiplatformapp://auth/callback?portal=return',
      });

      if (result.data?.url) {
        await openUrl(result.data.url);
      }
    } catch (error) {
      console.error('Failed to create portal session:', error);
    }
  };

  if (authLoading) {
    return (
      <MobileLayout>
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
            <p className="text-muted-foreground">Loading...</p>
          </div>
        </div>
      </MobileLayout>
    );
  }

  if (!user) {
    return (
      <MobileLayout>
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <h2 className="text-2xl font-bold mb-4">Not Signed In</h2>
            <p className="text-muted-foreground mb-6">Please sign in to view your account.</p>
          </div>
        </div>
      </MobileLayout>
    );
  }

  return (
    <MobileLayout>
      <div className="p-6 max-w-lg mx-auto">
        <h1 className="text-3xl font-bold mb-8">Account</h1>

        {/* User Profile */}
        <div className="mb-6 p-5 border rounded-lg bg-card">
          <h2 className="text-xl font-semibold mb-4">Profile</h2>
          <div className="space-y-3">
            <div>
              <label className="text-sm font-medium text-muted-foreground">Email</label>
              <p className="text-base">{user.email}</p>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">User ID</label>
              <p className="text-xs font-mono break-all">{user.id}</p>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">Created</label>
              <p className="text-sm">
                {new Date(user.created_at).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'long',
                  day: 'numeric',
                })}
              </p>
            </div>
          </div>
        </div>

        {/* Subscription Status */}
        <div className="mb-6 p-5 border rounded-lg bg-card">
          <h2 className="text-xl font-semibold mb-4">Subscription</h2>

          {subLoading ? (
            <div className="py-4">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
            </div>
          ) : subscription ? (
            <div className="space-y-4">
              <div>
                <label className="text-sm font-medium text-muted-foreground">Plan</label>
                <p className="text-base capitalize">{subscription.prices?.products?.name ?? 'Unknown'}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-muted-foreground">Status</label>
                <p className="text-base">
                  <span
                    className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      subscription.status === 'active'
                        ? 'bg-green-100 text-green-800'
                        : subscription.status === 'trialing'
                          ? 'bg-blue-100 text-blue-800'
                          : subscription.status === 'past_due'
                            ? 'bg-yellow-100 text-yellow-800'
                            : 'bg-gray-100 text-gray-800'
                    }`}
                  >
                    {subscription.status}
                  </span>
                </p>
              </div>
              {subscription.current_period_end && (
                <div>
                  <label className="text-sm font-medium text-muted-foreground">
                    {subscription.cancel_at_period_end ? 'Expires' : 'Renews'}
                  </label>
                  <p className="text-base">
                    {new Date(subscription.current_period_end).toLocaleDateString('en-US', {
                      year: 'numeric',
                      month: 'long',
                      day: 'numeric',
                    })}
                  </p>
                </div>
              )}
              <div className="pt-2">
                <Button onClick={handleManageBilling} variant="outline" className="w-full min-h-[44px]">
                  Manage Billing
                </Button>
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              <p className="text-muted-foreground">
                You don't have an active subscription. Subscribe to unlock premium features.
              </p>
              <Button onClick={handleSubscribe} className="w-full min-h-[44px]">
                Subscribe Now
              </Button>
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="p-5 border rounded-lg bg-card">
          <h2 className="text-xl font-semibold mb-4">Account Actions</h2>
          <Button onClick={handleSignOut} variant="destructive" className="w-full min-h-[44px]">
            Sign Out
          </Button>
        </div>
      </div>
    </MobileLayout>
  );
}
