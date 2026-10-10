import { Spinner } from '@/components/feedback/Spinner';
import { useSession } from '@/features/auth/hooks/useSession';

import { LoginPage } from '@/features/auth/pages/LoginPage';
import { RegisterPage } from '@/features/auth/pages/RegisterPage';
import type { ReactNode } from 'react';
import { Navigate, useLocation } from 'react-router-dom';

/** Signed-out visitors are sent to the sign-in page. */
export function RequireSession({ children }: { children: ReactNode }) {
  const { user, isLoading } = useSession();
  const location = useLocation();

  if (isLoading) {
    return <Spinner label="Checking your session" />;
  }
  if (!user) {
    return <Navigate to="/login" replace state={{ from: location.pathname }} />;
  }
  return <>{children}</>;
}

/** Signed-in visitors skip the sign-in page. */
export function RedirectIfSignedIn({ children }: { children: ReactNode }) {
  const { user, isLoading } = useSession();

  if (isLoading) {
    return <Spinner label="Checking your session" />;
  }
  return user ? <Navigate to="/" replace /> : <>{children}</>;
}

export { LoginPage, RegisterPage };