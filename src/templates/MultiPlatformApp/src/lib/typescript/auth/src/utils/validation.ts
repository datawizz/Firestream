/**
 * Validation utilities
 *
 * Utility functions for validating authentication data
 *
 * @packageDocumentation
 */

/**
 * Validate email format
 */
export function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

/**
 * Validate password strength
 */
export function isValidPassword(password: string, minLength: number = 8): boolean {
  return password.length >= minLength;
}

/**
 * Get password strength score
 */
export function getPasswordStrength(password: string): number {
  let strength = 0;

  if (password.length >= 8) strength += 1;
  if (password.length >= 12) strength += 1;
  if (/[a-z]/.test(password)) strength += 1;
  if (/[A-Z]/.test(password)) strength += 1;
  if (/[0-9]/.test(password)) strength += 1;
  if (/[^a-zA-Z0-9]/.test(password)) strength += 1;

  return strength;
}
