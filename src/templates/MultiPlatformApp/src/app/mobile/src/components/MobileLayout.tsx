import { type ReactNode } from 'react';

interface MobileLayoutProps {
  children: ReactNode;
}

export function MobileLayout({ children }: MobileLayoutProps) {
  return (
    <div
      className="min-h-screen bg-background"
      style={{
        paddingTop: 'var(--safe-area-top)',
        paddingBottom: 'var(--safe-area-bottom)',
        paddingLeft: 'var(--safe-area-left)',
        paddingRight: 'var(--safe-area-right)',
      }}
    >
      {children}
    </div>
  );
}
