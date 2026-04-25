/**
 * Tauri platform Supabase adapter
 *
 * Implementation of IAuthAdapter for Tauri desktop applications
 *
 * @packageDocumentation
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { IAuthAdapter } from '../../interfaces/auth-adapter';
import type {
  AuthUser,
  Session,
  SignInWithEmailOptions,
  SignInWithPasswordOptions,
  SignInWithOAuthOptions,
  SignUpWithEmailOptions,
  ActionResponse,
  AuthEventCallback,
  AuthEvent,
} from '../../types/auth';
import type { Database } from '../../types/database';
import { createTauriSupabaseClient, type TauriSupabaseClientConfig } from './supabase-client.adapter';
import { TauriSessionStorageAdapter } from './session-storage.adapter';

/**
 * Configuration for Tauri Supabase adapter
 */
export interface SupabaseTauriAdapterConfig {
  /**
   * Supabase client configuration
   */
  supabase: TauriSupabaseClientConfig;

  /**
   * Redirect URLs
   */
  redirectUrls?: {
    login?: string;
    callback?: string;
    error?: string;
  };
}

/**
 * Supabase authentication adapter for Tauri desktop platforms
 *
 * Uses @tauri-apps/api for native desktop integration
 */
export class SupabaseTauriAdapter implements IAuthAdapter {
  private client: SupabaseClient<Database>;
  private sessionStorage: TauriSessionStorageAdapter;
  private redirectUrls: { login: string; callback: string; error: string };
  private initialized = false;

  constructor(config: SupabaseTauriAdapterConfig) {
    this.sessionStorage =
      config.supabase.sessionStorage ||
      new TauriSessionStorageAdapter({ useFallback: config.supabase.useFallback });

    this.client = createTauriSupabaseClient({
      ...config.supabase,
      sessionStorage: this.sessionStorage,
    });

    this.redirectUrls = {
      login: config.redirectUrls?.login || '/login',
      callback: config.redirectUrls?.callback || '/auth/callback',
      error: config.redirectUrls?.error || '/auth/error',
    };
  }

  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    // Initialize auth state
    await this.getSession();
    this.initialized = true;
  }

  async signInWithEmail(options: SignInWithEmailOptions): Promise<ActionResponse<void>> {
    try {
      const { error } = await this.client.auth.signInWithOtp({
        email: options.email,
        options: {
          emailRedirectTo: options.redirectTo || this.redirectUrls.callback,
        },
      });

      if (error) {
        return { data: null, error };
      }

      return { data: undefined, error: null };
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Unknown error'),
      };
    }
  }

  async signInWithPassword(
    options: SignInWithPasswordOptions
  ): Promise<ActionResponse<Session>> {
    try {
      const { data, error } = await this.client.auth.signInWithPassword({
        email: options.email,
        password: options.password,
      });

      if (error) {
        return { data: null, error };
      }

      return { data: data.session, error: null };
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Unknown error'),
      };
    }
  }

  async signInWithOAuth(
    options: SignInWithOAuthOptions
  ): Promise<ActionResponse<{ url: string }>> {
    try {
      const { data, error } = await this.client.auth.signInWithOAuth({
        provider: options.provider,
        options: {
          redirectTo: options.redirectTo || this.redirectUrls.callback,
        },
      });

      if (error) {
        return { data: null, error };
      }

      return { data: { url: data.url }, error: null };
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Unknown error'),
      };
    }
  }

  async signUpWithEmail(options: SignUpWithEmailOptions): Promise<ActionResponse<Session>> {
    try {
      const { data, error } = await this.client.auth.signUp({
        email: options.email,
        password: options.password,
        options: {
          data: {
            full_name: options.fullName,
          },
          emailRedirectTo: options.redirectTo || this.redirectUrls.callback,
        },
      });

      if (error) {
        return { data: null, error };
      }

      return { data: data.session, error: null };
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Unknown error'),
      };
    }
  }

  async signOut(): Promise<ActionResponse<void>> {
    try {
      const { error } = await this.client.auth.signOut();

      if (error) {
        return { data: null, error };
      }

      await this.sessionStorage.clearSession();

      return { data: undefined, error: null };
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Unknown error'),
      };
    }
  }

  async getSession(): Promise<Session | null> {
    try {
      const { data } = await this.client.auth.getSession();
      return data.session;
    } catch (error) {
      console.error('Failed to get session:', error);
      return null;
    }
  }

  async getUser(): Promise<AuthUser | null> {
    try {
      const { data } = await this.client.auth.getUser();
      return data.user as AuthUser;
    } catch (error) {
      console.error('Failed to get user:', error);
      return null;
    }
  }

  async refreshSession(): Promise<ActionResponse<Session>> {
    try {
      const { data, error } = await this.client.auth.refreshSession();

      if (error) {
        return { data: null, error };
      }

      return { data: data.session, error: null };
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Unknown error'),
      };
    }
  }

  async resetPassword(email: string): Promise<ActionResponse<void>> {
    try {
      const { error } = await this.client.auth.resetPasswordForEmail(email, {
        redirectTo: this.redirectUrls.callback,
      });

      if (error) {
        return { data: null, error };
      }

      return { data: undefined, error: null };
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Unknown error'),
      };
    }
  }

  async updatePassword(newPassword: string): Promise<ActionResponse<void>> {
    try {
      const { error } = await this.client.auth.updateUser({
        password: newPassword,
      });

      if (error) {
        return { data: null, error };
      }

      return { data: undefined, error: null };
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Unknown error'),
      };
    }
  }

  async updateEmail(newEmail: string): Promise<ActionResponse<void>> {
    try {
      const { error } = await this.client.auth.updateUser({
        email: newEmail,
      });

      if (error) {
        return { data: null, error };
      }

      return { data: undefined, error: null };
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Unknown error'),
      };
    }
  }

  onAuthStateChange(callback: AuthEventCallback): () => void {
    const { data: subscription } = this.client.auth.onAuthStateChange(
      (event, session) => {
        // Map Supabase auth events to our auth events
        const authEvent = this.mapAuthEvent(event);
        if (authEvent) {
          callback(authEvent, session);
        }
      }
    );

    return () => {
      subscription.subscription.unsubscribe();
    };
  }

  private mapAuthEvent(event: string): AuthEvent | null {
    switch (event) {
      case 'SIGNED_IN':
        return 'SIGNED_IN';
      case 'SIGNED_OUT':
        return 'SIGNED_OUT';
      case 'USER_UPDATED':
        return 'USER_UPDATED';
      case 'PASSWORD_RECOVERY':
        return 'PASSWORD_RECOVERY';
      case 'TOKEN_REFRESHED':
        return 'TOKEN_REFRESHED';
      default:
        return null;
    }
  }
}
