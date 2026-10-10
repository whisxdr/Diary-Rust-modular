import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';

import { ErrorMessage } from '@/components/feedback/ErrorMessage';
import { Card } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Field } from '@/components/ui/Field';
import { changePasswordSchema, fieldErrors } from '@/features/auth/schemas';
import { useSession } from '@/features/auth/hooks/useSession';
import { useAsyncData, messageOf } from '@/hooks/useAsyncData';

import { authApi } from '@/features/auth/api';
import { settingsApi } from '../api';
import type { Theme } from '../api';

export function SettingsPage() {
  const { user, logout } = useSession();
  const preferences = useAsyncData((signal) => settingsApi.preferences(signal), []);
  const profile = useAsyncData((signal) => settingsApi.profile(signal), []);

  const [theme, setTheme] = useState<Theme>('light');
  const [displayName, setDisplayName] = useState('');
  const [passwords, setPasswords] = useState({ current_password: '', new_password: '' });
  const [passwordErrors, setPasswordErrors] = useState<Record<string, string>>({});
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (preferences.data) {
      setTheme(preferences.data.theme);
      applyTheme(preferences.data.theme);
    }
  }, [preferences.data]);

  useEffect(() => {
    if (profile.data) {
      setDisplayName(profile.data.display_name);
    }
  }, [profile.data]);

  async function saveTheme(next: Theme) {
    setTheme(next);
    applyTheme(next);
    await settingsApi.updatePreferences({ theme: next });
  }

  async function saveProfile(event: FormEvent) {
    event.preventDefault();
    setNotice(null);
    setError(null);

    try {
      await settingsApi.updateProfile({ display_name: displayName });
      setNotice('Profile updated.');
      profile.reload();
    } catch (cause) {
      setError(messageOf(cause));
    }
  }

  async function savePassword(event: FormEvent) {
    event.preventDefault();
    setNotice(null);
    setError(null);

    const parsed = changePasswordSchema.safeParse(passwords);
    if (!parsed.success) {
      setPasswordErrors(fieldErrors(parsed.error));
      return;
    }

    setPasswordErrors({});
    try {
      await authApi.changePassword(parsed.data);
      setPasswords({ current_password: '', new_password: '' });
      setNotice('Password updated. Other devices were signed out.');
    } catch (cause) {
      setError(messageOf(cause));
    }
  }

  return (
    <div className="page">
      <header className="page__header">
        <h1>Settings</h1>
      </header>

      {notice ? <p className="notice">{notice}</p> : null}
      {error ? <ErrorMessage message={error} /> : null}

      <Card title="Appearance">
        <Field label="Theme">
          <select value={theme} onChange={(event) => void saveTheme(event.target.value as Theme)}>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
            <option value="system">System</option>
          </select>
        </Field>
      </Card>

      <Card title="Profile">
        <form onSubmit={saveProfile} className="stack">
          <Field label="Name">
            <input value={displayName} onChange={(event) => setDisplayName(event.target.value)} />
          </Field>
          <Field label="Email" hint="Email cannot be changed">
            <input value={profile.data?.email ?? user?.email ?? ''} readOnly />
          </Field>
          <Button type="submit" variant="primary">
            Save profile
          </Button>
        </form>
      </Card>

      <Card title="Password">
        <form onSubmit={savePassword} className="stack">
          <Field label="Current password" error={passwordErrors['current_password']}>
            <input
              type="password"
              autoComplete="current-password"
              value={passwords.current_password}
              onChange={(event) =>
                setPasswords((current) => ({ ...current, current_password: event.target.value }))
              }
            />
          </Field>
          <Field label="New password" error={passwordErrors['new_password']}>
            <input
              type="password"
              autoComplete="new-password"
              value={passwords.new_password}
              onChange={(event) =>
                setPasswords((current) => ({ ...current, new_password: event.target.value }))
              }
            />
          </Field>
          <Button type="submit" variant="primary">
            Change password
          </Button>
        </form>
      </Card>

      <Card title="Session">
        <Button onClick={() => void logout()}>Sign out</Button>
      </Card>
    </div>
  );
}

function applyTheme(theme: Theme) {
  document.documentElement.dataset['theme'] = theme;
}