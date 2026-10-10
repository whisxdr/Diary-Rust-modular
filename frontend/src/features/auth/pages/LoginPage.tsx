import { useState } from 'react';
import type { FormEvent } from 'react';
import { useNavigate } from 'react-router-dom';

import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Button } from '@/components/ui/Button';
import { Field } from '@/components/ui/Field';
import { messageOf } from '@/hooks/useAsyncData';

import { fieldErrors, loginSchema } from '../schemas';
import { useSession } from '../hooks/useSession';

export function LoginPage() {
  const { login } = useSession();
  const navigate = useNavigate();

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [failure, setFailure] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setFailure(null);

    const parsed = loginSchema.safeParse({ email, password });
    if (!parsed.success) {
      setErrors(fieldErrors(parsed.error));
      return;
    }

    setErrors({});
    setIsSubmitting(true);
    try {
      await login(parsed.data);
      navigate('/');
    } catch (cause) {
      setFailure(messageOf(cause));
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="auth">
      <h1>Welcome back</h1>
      <form className="auth__form" onSubmit={handleSubmit}>
        <Field label="Email" error={errors['email']}>
          <input
            type="email"
            value={email}
            autoComplete="email"
            onChange={(event) => setEmail(event.target.value)}
          />
        </Field>

        <Field label="Password" error={errors['password']}>
          <input
            type="password"
            value={password}
            autoComplete="current-password"
            onChange={(event) => setPassword(event.target.value)}
          />
        </Field>

        {failure ? <ErrorMessage message={failure} /> : null}

        <Button type="submit" variant="primary" disabled={isSubmitting}>
          {isSubmitting ? 'Signing in…' : 'Sign in'}
        </Button>
      </form>

      <p>
        No account yet? <a href="/register">Create one</a>
      </p>
    </div>
  );
}