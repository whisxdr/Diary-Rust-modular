import { SessionProvider } from '@/features/auth/hooks/useSession';
import type { ReactNode } from 'react';
import { BrowserRouter } from 'react-router-dom';

/** Application-level providers: routing and the signed-in account. */
export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <BrowserRouter>
      <SessionProvider>{children}</SessionProvider>
    </BrowserRouter>
  );
}