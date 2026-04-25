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

export interface DevAuthAdapterConfig {
  credentials?: {
    email?: string;
    password?: string;
  };
  allowInProduction?: boolean;
}

const DEV_PREFIX = '[DEV AUTH]';

function createMockUser(email: string): AuthUser {
  const now = new Date().toISOString();
  return {
    id: 'dev-user-00000000-0000-0000-0000-000000000000',
    aud: 'authenticated',
    role: 'authenticated',
    email,
    email_confirmed_at: now,
    phone: '',
    confirmed_at: now,
    last_sign_in_at: now,
    app_metadata: { provider: 'dev', providers: ['dev'] },
    user_metadata: { full_name: 'Dev Admin' },
    identities: [],
    created_at: now,
    updated_at: now,
    is_anonymous: false,
    factors: [],
  } as AuthUser;
}

function createMockSession(user: AuthUser): Session {
  return {
    access_token: 'dev-access-token',
    refresh_token: 'dev-refresh-token',
    expires_in: 3600,
    expires_at: Math.floor(Date.now() / 1000) + 3600,
    token_type: 'bearer',
    user,
  } as Session;
}

export class DevAuthAdapter implements IAuthAdapter {
  private currentUser: AuthUser | null = null;
  private currentSession: Session | null = null;
  private listeners: AuthEventCallback[] = [];
  private email: string;
  private password: string;

  constructor(config?: DevAuthAdapterConfig) {
    if (
      typeof process !== 'undefined' &&
      process.env?.NODE_ENV === 'production' &&
      !config?.allowInProduction
    ) {
      throw new Error(`${DEV_PREFIX} DevAuthAdapter cannot be used in production`);
    }

    this.email = config?.credentials?.email ?? 'admin@localhost';
    this.password = config?.credentials?.password ?? 'admin';
  }

  async initialize(): Promise<void> {
    console.warn(
      `%c${DEV_PREFIX} Using dev auth — no Supabase configured.\n` +
        `Sign in with: ${this.email} / ${this.password}`,
      'color: #f59e0b; font-weight: bold;'
    );
  }

  private fireEvent(event: AuthEvent, session: Session | null): void {
    for (const cb of this.listeners) {
      cb(event, session);
    }
  }

  async signInWithPassword(
    options: SignInWithPasswordOptions
  ): Promise<ActionResponse<Session>> {
    if (options.email === this.email && options.password === this.password) {
      this.currentUser = createMockUser(this.email);
      this.currentSession = createMockSession(this.currentUser);
      this.fireEvent('SIGNED_IN', this.currentSession);
      return { data: this.currentSession, error: null };
    }
    return {
      data: null,
      error: new Error('Invalid credentials'),
    };
  }

  async signInWithEmail(
    _options: SignInWithEmailOptions
  ): Promise<ActionResponse<void>> {
    console.warn(`${DEV_PREFIX} Magic links are not functional in dev mode. Use password sign-in.`);
    return { data: null, error: null };
  }

  async signInWithOAuth(
    _options: SignInWithOAuthOptions
  ): Promise<ActionResponse<{ url: string }>> {
    console.warn(`${DEV_PREFIX} OAuth is not functional in dev mode. Use password sign-in.`);
    return { data: null, error: new Error('OAuth not available in dev mode') };
  }

  async signUpWithEmail(
    options: SignUpWithEmailOptions
  ): Promise<ActionResponse<Session>> {
    console.warn(`${DEV_PREFIX} Sign-up simulated. Using provided credentials.`);
    this.currentUser = createMockUser(options.email);
    this.currentSession = createMockSession(this.currentUser);
    this.fireEvent('SIGNED_IN', this.currentSession);
    return { data: this.currentSession, error: null };
  }

  async signOut(): Promise<ActionResponse<void>> {
    this.currentUser = null;
    this.currentSession = null;
    this.fireEvent('SIGNED_OUT', null);
    return { data: null, error: null };
  }

  async getSession(): Promise<Session | null> {
    return this.currentSession;
  }

  async getUser(): Promise<AuthUser | null> {
    return this.currentUser;
  }

  async refreshSession(): Promise<ActionResponse<Session>> {
    if (this.currentSession) {
      this.currentSession = createMockSession(this.currentUser!);
      return { data: this.currentSession, error: null };
    }
    return { data: null, error: new Error('No session to refresh') };
  }

  async resetPassword(_email: string): Promise<ActionResponse<void>> {
    console.warn(`${DEV_PREFIX} Password reset is a no-op in dev mode.`);
    return { data: null, error: null };
  }

  async updatePassword(_newPassword: string): Promise<ActionResponse<void>> {
    console.warn(`${DEV_PREFIX} Password update is a no-op in dev mode.`);
    return { data: null, error: null };
  }

  async updateEmail(_newEmail: string): Promise<ActionResponse<void>> {
    console.warn(`${DEV_PREFIX} Email update is a no-op in dev mode.`);
    return { data: null, error: null };
  }

  onAuthStateChange(callback: AuthEventCallback): () => void {
    this.listeners.push(callback);
    return () => {
      this.listeners = this.listeners.filter((cb) => cb !== callback);
    };
  }
}
