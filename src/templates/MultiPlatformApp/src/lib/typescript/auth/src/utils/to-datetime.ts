/**
 * Date/time conversion utilities
 *
 * Utilities for converting between different timestamp formats.
 *
 * @packageDocumentation
 */

/**
 * Convert a Stripe timestamp (Unix seconds) to a JavaScript Date object
 *
 * @param secs - Unix timestamp in seconds
 * @returns Date object
 */
export function toDateTime(secs: number): Date {
  const t = new Date(0); // Unix epoch start
  t.setUTCSeconds(secs);
  return t;
}

/**
 * Convert a Stripe timestamp to an ISO string
 *
 * @param secs - Unix timestamp in seconds
 * @returns ISO 8601 date string
 */
export function toISOString(secs: number): string {
  return toDateTime(secs).toISOString();
}

/**
 * Convert a Date object to Unix timestamp (seconds)
 *
 * @param date - Date object to convert
 * @returns Unix timestamp in seconds
 */
export function fromDateTime(date: Date): number {
  return Math.floor(date.getTime() / 1000);
}
