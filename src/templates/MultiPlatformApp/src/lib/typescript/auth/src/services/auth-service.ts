/**
 * Authentication service
 *
 * High-level service for authentication operations that delegates to platform adapters
 *
 * @packageDocumentation
 */

import type { IAuthAdapter } from '../interfaces/auth-adapter';
import type {
  SignInWithEmailOptions,
  SignInWithPasswordOptions,
  SignInWithOAuthOptions,
  SignUpWithEmailOptions,
  ActionResponse,
  Session,
} from '../types/auth';

/**
 * Authentication service
 *
 * Provides a unified API for authentication operations across platforms
 */
export class AuthService {
  private adapter: IAuthAdapter;

  constructor(adapter: IAuthAdapter) {
    this.adapter = adapter;
  }

  /**
   * Initialize the authentication service
   */
  async initialize(): Promise<void> {
    await this.adapter.initialize();
  }

  /**
   * Sign in with email (magic link)
   */
  async signInWithEmail(options: SignInWithEmailOptions): Promise<ActionResponse<void>> {
    return this.adapter.signInWithEmail(options);
  }

  /**
   * Sign in with email and password
   */
  async signInWithPassword(options: SignInWithPasswordOptions): Promise<ActionResponse<Session>> {
    return this.adapter.signInWithPassword(options);
  }

  /**
   * Sign in with OAuth provider
   */
  async signInWithOAuth(options: SignInWithOAuthOptions): Promise<ActionResponse<{ url: string }>> {
    return this.adapter.signInWithOAuth(options);
  }

  /**
   * Sign up with email and password
   */
  async signUpWithEmail(options: SignUpWithEmailOptions): Promise<ActionResponse<Session>> {
    return this.adapter.signUpWithEmail(options);
  }

  /**
   * Sign out the current user
   */
  async signOut(): Promise<ActionResponse<void>> {
    return this.adapter.signOut();
  }

  /**
   * Get the current session
   */
  async getSession() {
    return this.adapter.getSession();
  }

  /**
   * Get the current user
   */
  async getUser() {
    return this.adapter.getUser();
  }

  /**
   * Refresh the current session
   */
  async refreshSession() {
    return this.adapter.refreshSession();
  }

  /**
   * Reset password for a user
   */
  async resetPassword(email: string) {
    return this.adapter.resetPassword(email);
  }

  /**
   * Update user password
   */
  async updatePassword(newPassword: string) {
    return this.adapter.updatePassword(newPassword);
  }

  /**
   * Update user email
   */
  async updateEmail(newEmail: string) {
    return this.adapter.updateEmail(newEmail);
  }

  /**
   * Get current user (convenience method)
   *
   * Alias for getUser() - returns the current authenticated user
   */
  async getCurrentUser() {
    return this.getUser();
  }

  /**
   * Check if a user is authenticated
   *
   * @returns True if a valid session exists
   */
  async isAuthenticated(): Promise<boolean> {
    const session = await this.getSession();
    return session !== null;
  }

  /**
   * Sign in helper for OAuth providers
   *
   * Simplified method for OAuth sign-in with common providers
   *
   * @param provider - The OAuth provider to use
   * @param redirectTo - Optional redirect URL after authentication
   */
  async signIn(provider: 'github' | 'google', redirectTo?: string) {
    return this.signInWithOAuth({ provider, redirectTo });
  }
}
