import { ReactNode } from 'react';
import { Navigate } from 'react-router-dom';
import { useAuth } from '@multi-platform-app/auth/hooks';

interface ProtectedRouteProps {
  children: ReactNode;
  redirectTo?: string;
}

/**
 * Protected route component that redirects to login if not authenticated
 */
export function ProtectedRoute({ children, redirectTo = '/login' }: ProtectedRouteProps) {
  const { user, isLoading } = useAuth();

  // Show loading state while checking auth
  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
          <p className="text-muted-foreground">Loading...</p>
        </div>
      </div>
    );
  }

  // Redirect to login if not authenticated
  if (!user) {
    return <Navigate to={redirectTo} replace />;
  }

  // Render children if authenticated
  return <>{children}</>;
}
