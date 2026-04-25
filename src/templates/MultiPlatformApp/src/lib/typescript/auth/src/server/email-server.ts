/**
 * Server-side email utilities
 *
 * Utilities for sending emails using various providers
 *
 * @packageDocumentation
 */

import { Resend } from 'resend';
import type { ReactNode } from 'react';

/**
 * Supported email providers
 */
export type EmailProvider = 'resend';

/**
 * Email provider configuration
 */
export interface EmailProviderConfig {
  provider: EmailProvider;
  apiKey: string;
  from: string;
  fromName?: string;
}

/**
 * Email send options
 */
export interface SendEmailOptions {
  to: string | string[];
  subject: string;
  react?: ReactNode;
  html?: string;
  text?: string;
  cc?: string | string[];
  bcc?: string | string[];
  replyTo?: string | string[];
  tags?: { name: string; value: string }[];
}

/**
 * Email service interface
 */
export interface EmailService {
  sendEmail(options: SendEmailOptions): Promise<{ id: string }>;
}

/**
 * Resend email service implementation
 */
class ResendEmailService implements EmailService {
  private client: Resend;
  private from: string;

  constructor(config: { apiKey: string; from: string }) {
    this.client = new Resend(config.apiKey);
    this.from = config.from;
  }

  async sendEmail(options: SendEmailOptions): Promise<{ id: string }> {
    const { data, error } = await this.client.emails.send({
      from: this.from,
      to: options.to,
      subject: options.subject,
      react: options.react as React.ReactElement,
      html: options.html,
      text: options.text,
      cc: options.cc,
      bcc: options.bcc,
      replyTo: options.replyTo,
      tags: options.tags,
    });

    if (error) {
      throw new Error(`Failed to send email: ${error.message}`);
    }

    if (!data) {
      throw new Error('Failed to send email: No response data');
    }

    return { id: data.id };
  }
}

/**
 * Create an email service client
 *
 * @param config - Email provider configuration
 * @returns Email service instance
 *
 * @example
 * ```typescript
 * const emailService = createEmailService({
 *   provider: 'resend',
 *   apiKey: process.env.RESEND_API_KEY,
 *   from: 'noreply@example.com',
 *   fromName: 'My App',
 * });
 *
 * await emailService.sendEmail({
 *   to: 'user@example.com',
 *   subject: 'Welcome!',
 *   html: '<h1>Welcome to our app</h1>',
 * });
 * ```
 */
export function createEmailService(config: EmailProviderConfig): EmailService {
  const { provider, apiKey, from, fromName } = config;

  const fromAddress = fromName ? `${fromName} <${from}>` : from;

  switch (provider) {
    case 'resend':
      return new ResendEmailService({
        apiKey,
        from: fromAddress,
      });

    default:
      throw new Error(`Unsupported email provider: ${provider}`);
  }
}

/**
 * Send a welcome email to a new user
 *
 * @param emailService - Email service instance
 * @param to - Recipient email address
 * @param name - User's name
 * @param template - React email template
 * @returns Email send result
 *
 * @example
 * ```typescript
 * await sendWelcomeEmail(
 *   emailService,
 *   'user@example.com',
 *   'John Doe',
 *   WelcomeEmailTemplate
 * );
 * ```
 */
export async function sendWelcomeEmail(
  emailService: EmailService,
  to: string,
  _name: string,
  template: ReactNode
): Promise<{ id: string }> {
  return emailService.sendEmail({
    to,
    subject: 'Welcome!',
    react: template,
  });
}

/**
 * Send a password reset email
 *
 * @param emailService - Email service instance
 * @param to - Recipient email address
 * @param resetLink - Password reset link
 * @param template - React email template
 * @returns Email send result
 *
 * @example
 * ```typescript
 * await sendPasswordResetEmail(
 *   emailService,
 *   'user@example.com',
 *   'https://example.com/reset?token=xxx',
 *   PasswordResetTemplate
 * );
 * ```
 */
export async function sendPasswordResetEmail(
  emailService: EmailService,
  to: string,
  _resetLink: string,
  template: ReactNode
): Promise<{ id: string }> {
  return emailService.sendEmail({
    to,
    subject: 'Reset your password',
    react: template,
  });
}
