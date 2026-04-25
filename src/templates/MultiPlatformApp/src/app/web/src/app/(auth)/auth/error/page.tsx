import Link from 'next/link';

export const metadata = {
  title: 'Authentication Error',
  description: 'An error occurred during authentication',
};

interface ErrorPageProps {
  searchParams: Promise<{ error?: string; error_description?: string }>;
}

export default async function AuthErrorPage({ searchParams }: ErrorPageProps) {
  const params = await searchParams;
  const error = params.error;
  const errorDescription = params.error_description;

  // Map common error codes to user-friendly messages
  const errorMessages: Record<string, { title: string; description: string }> = {
    access_denied: {
      title: 'Access Denied',
      description: 'You denied the authentication request or do not have permission to access this resource.',
    },
    server_error: {
      title: 'Server Error',
      description: 'An unexpected error occurred on our servers. Please try again later.',
    },
    temporarily_unavailable: {
      title: 'Service Unavailable',
      description: 'The authentication service is temporarily unavailable. Please try again later.',
    },
    invalid_request: {
      title: 'Invalid Request',
      description: 'The authentication request was invalid. Please try signing in again.',
    },
    unauthorized_client: {
      title: 'Unauthorized',
      description: 'This application is not authorized to perform this authentication request.',
    },
    unsupported_response_type: {
      title: 'Unsupported Response',
      description: 'The authentication response type is not supported.',
    },
    invalid_scope: {
      title: 'Invalid Scope',
      description: 'The requested permissions are invalid or not available.',
    },
    otp_expired: {
      title: 'Link Expired',
      description: 'The magic link has expired. Please request a new one.',
    },
    otp_disabled: {
      title: 'Magic Links Disabled',
      description: 'Magic link authentication is not enabled for this application.',
    },
  };

  const errorInfo = error
    ? errorMessages[error] || {
        title: 'Authentication Error',
        description: errorDescription || 'An unexpected error occurred during authentication.',
      }
    : {
        title: 'Authentication Error',
        description: 'An unexpected error occurred during authentication.',
      };

  return (
    <div className="w-full max-w-md space-y-8">
      <div className="text-center">
        {/* Error Icon */}
        <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-red-100 dark:bg-red-900/30">
          <svg
            className="h-8 w-8 text-red-600 dark:text-red-400"
            fill="none"
            viewBox="0 0 24 24"
            strokeWidth="1.5"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z"
            />
          </svg>
        </div>

        <h2 className="mt-6 text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          {errorInfo.title}
        </h2>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          {errorInfo.description}
        </p>
      </div>

      {/* Error Details (for debugging) */}
      {error && (
        <div className="rounded-md bg-red-50 p-4 dark:bg-red-900/20">
          <p className="text-xs text-red-700 dark:text-red-300">
            Error code: <code className="font-mono">{error}</code>
          </p>
        </div>
      )}

      {/* Action Buttons */}
      <div className="space-y-4">
        <Link
          href="/login"
          className="w-full flex items-center justify-center gap-2 rounded-lg bg-blue-600 px-4 py-3 text-sm font-semibold text-white shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
        >
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth="1.5" stroke="currentColor">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M15.75 9V5.25A2.25 2.25 0 0013.5 3h-6a2.25 2.25 0 00-2.25 2.25v13.5A2.25 2.25 0 007.5 21h6a2.25 2.25 0 002.25-2.25V15m3 0l3-3m0 0l-3-3m3 3H9"
            />
          </svg>
          Try signing in again
        </Link>

        <Link
          href="/"
          className="w-full flex items-center justify-center gap-2 rounded-lg border border-gray-300 bg-white px-4 py-3 text-sm font-semibold text-gray-700 shadow-sm hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700"
        >
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth="1.5" stroke="currentColor">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25"
            />
          </svg>
          Go to homepage
        </Link>
      </div>

      {/* Help Text */}
      <p className="text-center text-xs text-gray-500 dark:text-gray-400">
        If this problem persists, please{' '}
        <a href="mailto:support@example.com" className="underline hover:text-gray-700 dark:hover:text-gray-300">
          contact support
        </a>
        .
      </p>
    </div>
  );
}
