import { useEffect, useState } from 'react';
import { MemoryRouter, Routes, Route, Navigate, Link } from 'react-router-dom';
import { useAuth } from '@multi-platform-app/auth/hooks';
import { DemoView } from '@multi-platform-app/shared/components';
import { Button } from '@multi-platform-app/shared/components';
import { useTheme } from '@multi-platform-app/shared/hooks';
import { Login } from './views/auth/Login';
import { Account } from './views/auth/Account';
import { ProtectedRoute } from './components/ProtectedRoute';
import { MobileLayout } from './components/MobileLayout';
import { initDeepLinkListener } from './lib/deep-link';

function HomePage() {
  const { user } = useAuth();

  return (
    <MobileLayout>
      <DemoView
        platformLabel="Mobile"
        headerExtra={
          <Link to="/account">
            <Button variant="outline" className="min-h-[44px]">
              {user ? `Account (${user.email})` : 'Sign In'}
            </Button>
          </Link>
        }
        footerExtra={
          user && (
            <div className="mb-8 p-6 border rounded-lg bg-green-50 dark:bg-green-900/20">
              <h2 className="text-2xl font-semibold mb-4">Authentication Status</h2>
              <p className="text-lg">
                Signed in as: <span className="font-mono font-bold">{user.email}</span>
              </p>
              <p className="text-sm text-muted-foreground mt-2">
                Auth tokens are stored securely in the iOS Keychain
              </p>
            </div>
          )
        }
      />
    </MobileLayout>
  );
}

function App() {
  useTheme();
  const [deepLinkInitialized, setDeepLinkInitialized] = useState(false);

  useEffect(() => {
    initDeepLinkListener()
      .then(() => {
        console.log('Deep link listener ready');
        setDeepLinkInitialized(true);
      })
      .catch((error) => {
        console.error('Failed to initialize deep link listener:', error);
        setDeepLinkInitialized(true);
      });
  }, []);

  if (!deepLinkInitialized) {
    return (
      <MobileLayout>
        <div className="min-h-screen flex items-center justify-center">
          <div className="text-center">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
            <p className="text-muted-foreground">Initializing...</p>
          </div>
        </div>
      </MobileLayout>
    );
  }

  return (
    <MemoryRouter>
      <Routes>
        <Route path="/login" element={<Login />} />

        <Route
          path="/"
          element={
            <ProtectedRoute>
              <HomePage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/account"
          element={
            <ProtectedRoute>
              <Account />
            </ProtectedRoute>
          }
        />

        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </MemoryRouter>
  );
}

export default App;
