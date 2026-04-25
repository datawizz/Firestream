import { useAuth, useSubscription } from '@multi-platform-app/auth/hooks';
import { Button } from '@multi-platform-app/shared/components';
import { openUrl } from '@tauri-apps/plugin-opener';

export function Account() {
  const { user, signOut, loading: authLoading } = useAuth();
  const { subscription, createCheckoutSession, createPortalSession, loading: subLoading } =
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
      const { url } = await createCheckoutSession({
        priceId: 'price_xxx', // Replace with your actual price ID
        successUrl: 'multiplatformapp://auth/callback?checkout=success',
        cancelUrl: 'multiplatformapp://auth/callback?checkout=cancel',
      });

      if (url) {
        await openUrl(url);
      }
    } catch (error) {
      console.error('Failed to create checkout session:', error);
    }
  };

  const handleManageBilling = async () => {
    try {
      const { url } = await createPortalSession({
        returnUrl: 'multiplatformapp://auth/callback?portal=return',
      });

      if (url) {
        await openUrl(url);
      }
    } catch (error) {
      console.error('Failed to create portal session:', error);
    }
  };

  if (authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
          <p className="text-muted-foreground">Loading...</p>
        </div>
      </div>
    );
  }

  if (!user) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-4">Not Signed In</h2>
          <p className="text-muted-foreground mb-6">Please sign in to view your account.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto p-8 max-w-4xl">
        <h1 className="text-4xl font-bold mb-8">Account Settings</h1>

        {/* User Profile */}
        <div className="mb-8 p-6 border rounded-lg bg-card">
          <h2 className="text-2xl font-semibold mb-4">Profile</h2>
          <div className="space-y-3">
            <div>
              <label className="text-sm font-medium text-muted-foreground">Email</label>
              <p className="text-lg">{user.email}</p>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">User ID</label>
              <p className="text-sm font-mono">{user.id}</p>
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
        <div className="mb-8 p-6 border rounded-lg bg-card">
          <h2 className="text-2xl font-semibold mb-4">Subscription</h2>

          {subLoading ? (
            <div className="py-4">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
            </div>
          ) : subscription ? (
            <div className="space-y-4">
              <div>
                <label className="text-sm font-medium text-muted-foreground">Plan</label>
                <p className="text-lg capitalize">{subscription.plan}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-muted-foreground">Status</label>
                <p className="text-lg">
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
                  <p className="text-lg">
                    {new Date(subscription.current_period_end).toLocaleDateString('en-US', {
                      year: 'numeric',
                      month: 'long',
                      day: 'numeric',
                    })}
                  </p>
                </div>
              )}
              <div className="pt-2">
                <Button onClick={handleManageBilling} variant="outline">
                  Manage Billing
                </Button>
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              <p className="text-muted-foreground">
                You don't have an active subscription. Subscribe to unlock premium features.
              </p>
              <Button onClick={handleSubscribe}>Subscribe Now</Button>
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="p-6 border rounded-lg bg-card">
          <h2 className="text-2xl font-semibold mb-4">Account Actions</h2>
          <div className="space-y-3">
            <Button onClick={handleSignOut} variant="destructive" className="w-full sm:w-auto">
              Sign Out
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
