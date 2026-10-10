import { useState } from 'react';
import type { FormEvent } from 'react';
import { Link, useNavigate } from 'react-router-dom';

import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Button } from '@/components/ui/Button';
import { Field } from '@/components/ui/Field';
import { messageOf } from '@/hooks/useAsyncData';

import { useSession } from '../hooks/useSession';
import { fieldErrors, registerSchema } from '../schemas';

export function RegisterPage() {
  const { register } = useSession();
  const navigate = useNavigate();

  const [form, setForm] = useState({ email: '', display_name: '', password: '' });
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [failure, setFailure] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  function update(field: keyof typeof form) {
    return (event: { target: { value: string } }) =>
      setForm((current) => ({ ...current, [field]: event.target.value }));
  }

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setFailure(null);

    const parsed = registerSchema.safeParse(form);
    if (!parsed.success) {
      setErrors(fieldErrors(parsed.error));
      return;
    }

    setErrors({});
    setIsSubmitting(true);
    try {
      await register(parsed.data);
      navigate('/');
    } catch (cause) {
      setFailure(messageOf(cause));
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="auth">
      <h1>Start your diary</h1>
      <p className="auth__lead">Everything stays on your machine.</p>

      <form className="auth__form" onSubmit={handleSubmit}>
        <Field label="Name" error={errors['display_name']}>
          <input value={form.display_name} autoComplete="name" onChange={update('display_name')} />
        </Field>

        <Field label="Email" error={errors['email']}>
          <input type="email" value={form.email} autoComplete="email" onChange={update('email')} />
        </Field>

        <Field label="Password" hint="At least 12 characters" error={errors['password']}>
          <input
            type="password"
            value={form.password}
            autoComplete="new-password"
            onChange={update('password')}
          />
        </Field>

        {failure ? <ErrorMessage message={failure} /> : null}

        <Button type="submit" variant="primary" disabled={isSubmitting}>
          {isSubmitting ? 'Creating account…' : 'Create account'}
        </Button>
      </form>

      <p>
        Already have an account? <Link to="/login">Sign in</Link>
      </p>
    </div>
  );
}