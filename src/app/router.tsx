import { Navigate, Route, Routes } from 'react-router-dom';

import { AppShell } from '@/components/layout/AppShell';
import { CalendarPage } from '@/features/calendar/pages/CalendarPage';
import { EntriesPage } from '@/features/entries/pages/EntriesPage';
import { EntryPage } from '@/features/entries/pages/EntryPage';
import { ExportPage } from '@/features/exports/pages/ExportPage';
import { ImportPage } from '@/features/imports/pages/ImportPage';
import { InsightsPage } from '@/features/insights/pages/InsightsPage';
import { SearchPage } from '@/features/search/pages/SearchPage';
import { SettingsPage } from '@/features/settings/pages/SettingsPage';
import { TodayPage } from '@/features/dashboard/pages/TodayPage';

import { LoginPage, RedirectIfSignedIn, RegisterPage, RequireSession } from './route-guards';

/** Route table. Feature pages stay in their own folders. */
export function AppRouter() {
  return (
    <Routes>
      <Route
        path="/login"
        element={
          <RedirectIfSignedIn>
            <LoginPage />
          </RedirectIfSignedIn>
        }
      />
      <Route
        path="/register"
        element={
          <RedirectIfSignedIn>
            <RegisterPage />
          </RedirectIfSignedIn>
        }
      />

      <Route
        element={
          <RequireSession>
            <AppShell />
          </RequireSession>
        }
      >
        <Route path="/" element={<TodayPage />} />
        <Route path="/entries" element={<EntriesPage />} />
        <Route path="/entries/:id" element={<EntryPage />} />
        <Route path="/calendar" element={<CalendarPage />} />
        <Route path="/insights" element={<InsightsPage />} />
        <Route path="/search" element={<SearchPage />} />
        <Route path="/backup/import" element={<ImportPage />} />
        <Route path="/backup/export" element={<ExportPage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Route>

      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}