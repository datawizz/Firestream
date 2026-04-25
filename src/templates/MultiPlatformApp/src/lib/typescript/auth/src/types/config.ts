/**
 * Configuration type definitions
 *
 * Types for configuring the auth package across different platforms.
 *
 * @packageDocumentation
 */

import type { OAuthProvider } from './auth';

/**
 * Environment variable names used by the auth package
 *
 * These constants define the base names of environment variables.
 * The PUBLIC_ prefix is used for client-safe variables across all platforms.
 * Legacy prefixes (NEXT_PUBLIC_, VITE_) are supported as fallbacks by the env utilities.
 */
export const ENV_VARS = {
  // Supabase
  SUPABASE_URL: 'SUPABASE_URL',
  SUPABASE_ANON_KEY: 'SUPABASE_ANON_KEY',
  SUPABASE_SERVICE_ROLE_KEY: 'SUPABASE_SERVICE_ROLE_KEY',

  // Stripe
  STRIPE_PUBLISHABLE_KEY: 'STRIPE_PUBLISHABLE_KEY',
  STRIPE_SECRET_KEY: 'STRIPE_SECRET_KEY',
  STRIPE_WEBHOOK_SECRET: 'STRIPE_WEBHOOK_SECRET',

  // Email (Resend)
  RESEND_API_KEY: 'RESEND_API_KEY',
  RESEND_FROM_EMAIL: 'RESEND_FROM_EMAIL',
  RESEND_FROM_NAME: 'RESEND_FROM_NAME',

  // Site
  SITE_URL: 'SITE_URL',
  API_URL: 'API_URL',
} as const;

/**
 * Required environment variables for client-side auth
 */
export const REQUIRED_CLIENT_ENV_VARS = [
  ENV_VARS.SUPABASE_URL,
  ENV_VARS.SUPABASE_ANON_KEY,
] as const;

/**
 * Required environment variables for server-side auth
 */
export const REQUIRED_SERVER_ENV_VARS = [
  ENV_VARS.SUPABASE_URL,
  ENV_VARS.SUPABASE_ANON_KEY,
  ENV_VARS.SUPABASE_SERVICE_ROLE_KEY,
] as const;

/**
 * Required environment variables for Stripe integration
 */
export const REQUIRED_STRIPE_ENV_VARS = [
  ENV_VARS.STRIPE_PUBLISHABLE_KEY,
  ENV_VARS.STRIPE_SECRET_KEY,
] as const;

/**
 * Required environment variables for email service
 */
export const REQUIRED_EMAIL_ENV_VARS = [ENV_VARS.RESEND_API_KEY] as const;

/**
 * Supabase configuration
 */
export interface SupabaseConfig {
  url: string;
  anonKey: string;
  serviceRoleKey?: string;
}

/**
 * Stripe configuration
 */
export interface StripeConfig {
  publishableKey: string;
  secretKey?: string;
  webhookSecret?: string;
}

/**
 * Email service configuration (Resend)
 */
export interface EmailConfig {
  apiKey: string;
  fromEmail: string;
  fromName?: string;
}

/**
 * OAuth provider configuration
 */
export interface OAuthConfig {
  enabled: boolean;
  providers: OAuthProvider[];
  redirectUrl?: string;
}

/**
 * Email authentication configuration
 */
export interface EmailAuthConfig {
  enabled: boolean;
  requireEmailConfirmation: boolean;
  magicLink: boolean;
  password: boolean;
}

/**
 * URL configuration
 */
export interface URLConfig {
  siteUrl: string;
  loginUrl: string;
  signupUrl: string;
  callbackUrl: string;
  errorUrl: string;
}

/**
 * Feature flags for authentication
 */
export interface AuthFeatureFlags {
  subscriptions: boolean;
  oauth: boolean;
  magicLink: boolean;
  passwordAuth: boolean;
  emailConfirmation: boolean;
  multiFactorAuth: boolean;
}

/**
 * Complete auth package configuration
 */
export interface AuthPackageConfig {
  supabase: SupabaseConfig;
  stripe?: StripeConfig;
  email?: EmailConfig;
  oauth: OAuthConfig;
  emailAuth: EmailAuthConfig;
  urls: URLConfig;
  features: AuthFeatureFlags;
}

/**
 * Environment-specific configuration
 */
export type Environment = 'development' | 'staging' | 'production';

/**
 * Configuration factory options
 */
export interface ConfigFactoryOptions {
  environment: Environment;
  overrides?: Partial<AuthPackageConfig>;
}
