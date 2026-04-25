import { useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route, Navigate, Link } from 'react-router-dom';
import { useAuth } from '@multi-platform-app/auth/hooks';
import { DemoView } from '@multi-platform-app/shared/components';
import { Button } from '@multi-platform-app/shared/components';
import { useTheme } from '@multi-platform-app/shared/hooks';
import { Login } from './views/auth/Login';
import { Account } from './views/auth/Account';
import { ProtectedRoute } from './components/ProtectedRoute';
import { initDeepLinkListener } from './lib/deep-link';

function HomePage() {
  const { user } = useAuth();

  return (
    <DemoView
      platformLabel="Desktop"
      headerExtra={
        <Link to="/account">
          <Button variant="outline">
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
              Auth tokens are stored securely in the system keychain
            </p>
          </div>
        )
      }
    />
  );
}

function App() {
  useTheme(); // Initialize dark mode detection
  const [deepLinkInitialized, setDeepLinkInitialized] = useState(false);

  // Initialize deep link listener on mount
  useEffect(() => {
    initDeepLinkListener()
      .then(() => {
        console.log('Deep link listener ready');
        setDeepLinkInitialized(true);
      })
      .catch((error) => {
        console.error('Failed to initialize deep link listener:', error);
        // Still allow the app to run even if deep links fail
        setDeepLinkInitialized(true);
      });
  }, []);

  // Show loading while initializing deep links
  if (!deepLinkInitialized) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
          <p className="text-muted-foreground">Initializing...</p>
        </div>
      </div>
    );
  }

  return (
    <BrowserRouter>
      <Routes>
        {/* Public routes */}
        <Route path="/login" element={<Login />} />

        {/* Protected routes */}
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

        {/* Catch all - redirect to home */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
