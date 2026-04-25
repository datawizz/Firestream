import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { createServerClient } from '@supabase/ssr';
import { getEnvVar } from '@multi-platform-app/auth/server';
import type { Database } from '@multi-platform-app/auth/types';

export async function GET(request: Request) {
  const requestUrl = new URL(request.url);
  const code = requestUrl.searchParams.get('code');
  const next = requestUrl.searchParams.get('next') || '/account';

  if (code) {
    const cookieStore = await cookies();
    const supabase = createServerClient<Database>(
      getEnvVar(process.env.PUBLIC_SUPABASE_URL, 'PUBLIC_SUPABASE_URL'),
      getEnvVar(process.env.PUBLIC_SUPABASE_ANON_KEY, 'PUBLIC_SUPABASE_ANON_KEY'),
      {
        cookies: {
          getAll() {
            return cookieStore.getAll();
          },
          setAll(cookiesToSet) {
            try {
              cookiesToSet.forEach(({ name, value, options }) => {
                cookieStore.set(name, value, options);
              });
            } catch {
              // The `setAll` method was called from a Server Component.
              // This can be ignored if you have middleware refreshing
              // user sessions.
            }
          },
        },
      }
    );

    const { error } = await supabase.auth.exchangeCodeForSession(code);

    if (!error) {
      // Check if user has a subscription
      const { data: session } = await supabase.auth.getSession();

      if (session.session) {
        const { data: subscription } = await supabase
          .from('subscriptions')
          .select('*')
          .eq('user_id', session.session.user.id)
          .in('status', ['active', 'trialing'])
          .maybeSingle();

        // If no subscription, redirect to pricing page
        if (!subscription) {
          return NextResponse.redirect(new URL('/pricing', requestUrl.origin));
        }
      }

      return NextResponse.redirect(new URL(next, requestUrl.origin));
    }
  }

  // Return the user to an error page with some instructions
  return NextResponse.redirect(new URL('/auth/error', requestUrl.origin));
}
