import { useState } from 'react';
import { NavLink, Outlet } from 'react-router-dom';

import { useSession } from '@/features/auth/hooks/useSession';
import { todayIso, formatLongDate } from '@/lib/date';

const NAV_ITEMS = [
  { to: '/', label: 'Today', end: true },
  { to: '/entries', label: 'Entries' },
  { to: '/calendar', label: 'Calendar' },
  { to: '/insights', label: 'Reflection' },
  { to: '/search', label: 'Search' },
  { to: '/backup/import', label: 'Import' },
  { to: '/backup/export', label: 'Export' },
  { to: '/settings', label: 'Settings' },
];

export function AppShell() {
  const { user, logout } = useSession();
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  return (
    <div className="shell">
      <header className="shell__header">
        <NavLink to="/" className="shell__brand">
          Dear My Diary
        </NavLink>
        <button
          type="button"
          className="shell__menu-button"
          aria-expanded={isMenuOpen}
          onClick={() => setIsMenuOpen((open) => !open)}
        >
          Menu
        </button>
      </header>

      <div className="shell__body">
        <nav className={`shell__nav ${isMenuOpen ? 'shell__nav--open' : ''}`}>
          {NAV_ITEMS.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.end ?? false}
              onClick={() => setIsMenuOpen(false)}
              className={({ isActive }) => (isActive ? 'nav-link nav-link--active' : 'nav-link')}
            >
              {item.label}
            </NavLink>
          ))}

          <div className="shell__footer">
            <p className="hint">{user?.display_name}</p>
            <button type="button" className="link" onClick={() => void logout()}>
              Sign out
            </button>
          </div>
        </nav>

        <main className="shell__main">
          <p className="shell__today">{formatLongDate(todayIso())}</p>
          <Outlet />
        </main>
      </div>
    </div>
  );
}