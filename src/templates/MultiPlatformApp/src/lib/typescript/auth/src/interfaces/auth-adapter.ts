/**
 * Authentication adapter interface
 *
 * Platform-agnostic interface for authentication operations.
 * Implementations should handle platform-specific details.
 *
 * @packageDocumentation
 */

import type {
  AuthUser,
  Session,
  SignInWithEmailOptions,
  SignInWithPasswordOptions,
  SignInWithOAuthOptions,
  SignUpWithEmailOptions,
  ActionResponse,
  AuthEventCallback,
} from '../types/auth';

/**
 * Base interface for authentication adapters
 *
 * Platform-specific adapters (web, Tauri, mobile) should implement this interface
 * to provide consistent authentication functionality across platforms.
 */
export interface IAuthAdapter {
  /**
   * Initialize the authentication adapter
   */
  initialize(): Promise<void>;

  /**
   * Sign in with email (magic link)
   */
  signInWithEmail(options: SignInWithEmailOptions): Promise<ActionResponse<void>>;

  /**
   * Sign in with email and password
   */
  signInWithPassword(options: SignInWithPasswordOptions): Promise<ActionResponse<Session>>;

  /**
   * Sign in with OAuth provider
   */
  signInWithOAuth(options: SignInWithOAuthOptions): Promise<ActionResponse<{ url: string }>>;

  /**
   * Sign up with email and password
   */
  signUpWithEmail(options: SignUpWithEmailOptions): Promise<ActionResponse<Session>>;

  /**
   * Sign out the current user
   */
  signOut(): Promise<ActionResponse<void>>;

  /**
   * Get the current session
   */
  getSession(): Promise<Session | null>;

  /**
   * Get the current user
   */
  getUser(): Promise<AuthUser | null>;

  /**
   * Refresh the current session
   */
  refreshSession(): Promise<ActionResponse<Session>>;

  /**
   * Reset password for a user
   */
  resetPassword(email: string): Promise<ActionResponse<void>>;

  /**
   * Update user password
   */
  updatePassword(newPassword: string): Promise<ActionResponse<void>>;

  /**
   * Update user email
   */
  updateEmail(newEmail: string): Promise<ActionResponse<void>>;

  /**
   * Subscribe to auth state changes
   */
  onAuthStateChange(callback: AuthEventCallback): () => void;
}
