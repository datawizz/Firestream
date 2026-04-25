/**
 * Authentication type definitions
 *
 * Core types for user authentication, sessions, and sign-in flows.
 *
 * @packageDocumentation
 */

import type { User as SupabaseUser, Session as SupabaseSession } from '@supabase/supabase-js';
import type { Database } from './database';

/**
 * Extended user type with profile information from database
 */
export type AuthUser = SupabaseUser & {
  profile?: Database['public']['Tables']['users']['Row'];
};

/**
 * Session type from Supabase
 */
export type Session = SupabaseSession;

/**
 * OAuth provider options
 */
export type OAuthProvider = 'github' | 'google';

/**
 * Sign-in method types
 */
export type SignInMethod = 'oauth' | 'email' | 'magic_link' | 'password';

/**
 * Options for signing in with email
 */
export interface SignInWithEmailOptions {
  email: string;
  redirectTo?: string;
}

/**
 * Options for signing in with password
 */
export interface SignInWithPasswordOptions {
  email: string;
  password: string;
}

/**
 * Options for signing in with OAuth
 */
export interface SignInWithOAuthOptions {
  provider: OAuthProvider;
  redirectTo?: string;
}

/**
 * Options for signing up with email
 */
export interface SignUpWithEmailOptions {
  email: string;
  password: string;
  fullName?: string;
  redirectTo?: string;
}

/**
 * Generic action response type
 */
export interface ActionResponse<T = any> {
  data: T | null;
  error: Error | null;
}

/**
 * Authentication state
 */
export interface AuthState {
  user: AuthUser | null;
  session: Session | null;
  isLoading: boolean;
  isAuthenticated: boolean;
}

/**
 * Authentication error types
 */
export type AuthError =
  | 'invalid_credentials'
  | 'user_not_found'
  | 'email_not_confirmed'
  | 'session_expired'
  | 'network_error'
  | 'unknown_error';

/**
 * Authentication event types
 */
export type AuthEvent =
  | 'SIGNED_IN'
  | 'SIGNED_OUT'
  | 'USER_UPDATED'
  | 'PASSWORD_RECOVERY'
  | 'TOKEN_REFRESHED';

/**
 * Authentication event callback
 */
export type AuthEventCallback = (event: AuthEvent, session: Session | null) => void;
