/**
 * Welcome email template
 *
 * Email sent to new users after successful signup
 *
 * @packageDocumentation
 */

import {
  Body,
  Button,
  Container,
  Head,
  Heading,
  Html,
  Link,
  Preview,
  Section,
  Text,
} from '@react-email/components';
import { Tailwind } from '@react-email/tailwind';

/**
 * Props for the welcome email template
 */
export interface WelcomeEmailProps {
  /**
   * User's name
   */
  name?: string;

  /**
   * URL to the dashboard
   */
  dashboardUrl?: string;

  /**
   * URL to account settings
   */
  accountUrl?: string;

  /**
   * Base URL for the site (for loading images)
   */
  siteUrl?: string;
}

/**
 * Welcome email component
 *
 * This email is sent to users after they successfully sign up.
 * It welcomes them and provides a link to their dashboard.
 *
 * @param props - Email template props
 * @returns React email component
 *
 * @example
 * ```typescript
 * const emailHtml = render(WelcomeEmail({
 *   name: 'John',
 *   dashboardUrl: 'https://example.com/dashboard',
 *   siteUrl: 'https://example.com',
 * }));
 * ```
 */
export function WelcomeEmail({
  name = 'there',
  dashboardUrl = '#',
  accountUrl = '#',
  siteUrl: _siteUrl = '',
}: WelcomeEmailProps) {
  return (
    <Html>
      <Head />
      <Preview>Welcome to our platform!</Preview>
      <Tailwind>
        <Body className="mx-auto my-auto bg-slate-50 px-2 py-10 font-sans">
          <Container className="mx-auto mt-10 w-full max-w-[464px] overflow-hidden rounded-lg bg-white shadow-lg">
            {/* Header Section */}
            <Section className="h-64 w-full bg-gradient-to-br from-blue-600 to-purple-600 p-8">
              <Heading className="mb-0 mt-16 text-center text-5xl font-bold text-white">
                Welcome{name ? `, ${name}` : ''}!
              </Heading>
            </Section>

            {/* Content Section */}
            <Section className="p-8">
              <Heading as="h2" className="m-0 mb-4 text-2xl font-bold text-gray-900">
                Thanks for signing up
              </Heading>

              <Text className="my-4 text-base leading-relaxed text-gray-700">
                We're excited to have you on board. You now have access to all of our features and
                can start exploring right away.
              </Text>

              <Text className="my-4 text-base leading-relaxed text-gray-700">
                Get started by visiting your dashboard where you can:
              </Text>

              <ul className="my-4 ml-6 list-disc text-base text-gray-700">
                <li>Set up your profile</li>
                <li>Explore available features</li>
                <li>Customize your settings</li>
                <li>Invite team members</li>
              </ul>

              <Button
                href={dashboardUrl}
                className="mt-6 rounded-lg bg-blue-600 px-6 py-3 font-semibold text-white no-underline hover:bg-blue-700"
              >
                Go to Dashboard
              </Button>
            </Section>

            {/* Divider */}
            <Section className="border-t border-gray-200" />

            {/* Footer Section */}
            <Section className="p-8 pt-4">
              <Text className="mb-2 text-sm text-gray-600">
                Need help getting started? Check out our documentation or contact our support team.
              </Text>

              <Text className="mb-0 text-sm text-gray-600">
                Best regards,
                <br />
                The Team
              </Text>
            </Section>
          </Container>

          {/* Unsubscribe Section */}
          <Container className="mx-auto mt-4 max-w-[464px]">
            <Section className="text-center">
              <Text className="m-0 text-xs text-gray-500">
                Not interested in receiving these emails?
              </Text>
              <Link
                className="text-center text-xs text-gray-500 underline"
                href={accountUrl}
              >
                Manage your notification settings
              </Link>
            </Section>
          </Container>
        </Body>
      </Tailwind>
    </Html>
  );
}

/**
 * Default export for React Email preview
 */
export default WelcomeEmail;
