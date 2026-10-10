# Dear My Diary

A local-first diary. The web app runs entirely in your browser — account,
entries, tags and backups stay on your device — and the optional Rust API in
`backend/` serves the same features when you want them stored server-side.

The application is a **modular monolith**. Business features own their routes,
handlers, services, repositories and API shapes; nothing reaches into another
feature's tables.

## Two modes, one interface

Every feature talks to `src/lib/api-client.ts` and nothing else. What sits
behind it depends on one environment variable:

| `VITE_API_BASE_URL` | Where the data lives |
| --- | --- |
| empty (default) | The browser. Accounts, entries, tags, moods and backups are stored in IndexedDB on this device. No server, no network. |
| set, e.g. `https://api.example.com` | That API, over HTTP with cookie sessions. |

Because the mode is chosen in one place, the feature code, screens and types
are identical either way. The browser mode answers the same URLs the API
answers, so nothing above `api-client` knows the difference.

Passwords in browser mode are hashed with PBKDF2-SHA256 in Web Crypto before
they are stored; the server uses Argon2id. Neither ever keeps a plain password.

Data in browser mode is tied to that one browser profile. Clearing site data
erases the diary, so use **Backup → Export** if it matters.

## Layout

```text
src/                    the React app (deployable to Vercel as-is)
  app/                  router, providers, route guards
  components/           ui, layout, feedback
  features/             same feature split as the backend
  hooks/, lib/          shared hooks and the API client
  styles/               tokens, themes, component styles
index.html, vite.config.ts, package.json

backend/
  migrations/           versioned schema, applied by the migrate binary
  src/
    main.rs             bootstrapping only
    app.rs              router composition
    config.rs           environment configuration
    state.rs            shared runtime dependencies
    shared/             errors, pagination, dates, validation, portable format
    infrastructure/     database, sessions, file storage, telemetry
    middleware/         authentication, csrf, rate limit
    features/           auth, accounts, entries, tags, moods, calendar,
                        attachments, imports, exports, search, insights, settings
  tests/                feature-grouped tests
```

The web app sits at the repository root and the API in `backend/`, so importing
the repository into Vercel finds one application instead of two competing ones.

## Running it

Only Node 20+ is needed for the default mode.

```bash
npm install
npm run dev
```

Open http://localhost:5173 and create an account. Nothing leaves the device.

Prerequisites for the API mode: Rust 1.82+, Node 20+, PostgreSQL 14+.

**1. Database.** Create the database, then copy the environment file and point
`DATABASE_URL` at it.

```bash
cd backend
cp .env.example .env
```

**2. Migrations.** The schema never changes during normal startup; the
`migrate` binary is the only path that applies it.

```bash
cargo run --bin migrate
```

**3. API.**

```bash
cargo run
```

**4. Point the app at it** by creating `.env` next to `package.json`:

```bash
VITE_API_BASE_URL=http://127.0.0.1:8080
```

## Deploying to Vercel

The repository root *is* the web app, so importing it into Vercel finds a single
application. `vercel.json` pins framework `vite`, build command `npm run build`,
output `dist`, and rewrites unknown paths to `index.html` so deep links like
`/settings` survive a refresh.

1. In Vercel, **Add New → Project**, import this repository, deploy.

That is the whole setup for browser mode: with no environment variables the
deployed site works as a complete diary on its own. Add `VITE_API_BASE_URL` only
if you also deploy the API and want the data server-side.

The API is a Rust server that needs PostgreSQL and a persistent volume for
attachment bytes, so it runs on a normal host (a VPS, Fly.io, Railway), not on
Vercel's serverless runtime. When the two sit on different sites, cookies have to
cross origins, so configure the API:

```bash
ALLOWED_ORIGINS=https://your-app.vercel.app
COOKIE_SAME_SITE=None
COOKIE_SECURE=auto     # auto means: Secure exactly when SameSite=None
```

`SameSite=None` is only accepted by browsers over HTTPS, so the API must be
served over TLS in that setup.

`.vercelignore` keeps build output out of the upload, so importing does not ship
the Rust `target` directory.

## Checks

```bash
# Web
npm run check:local   # browser-local backend, against an in-memory IndexedDB
npm run build         # tsc --noEmit && vite build

# API
cd backend
cargo fmt
cargo clippy --all-targets
cargo test
```

## How the pieces fit

- **Authentication** issues an opaque session token; only its SHA-256 hash is
  stored, and the cookie is `HttpOnly`. Mutating requests must echo the
  `dmd_csrf` cookie in an `X-CSRF-Token` header.
- **Entries** allow one per account per day. Word counts are computed on write so
  reflection statistics never rescan the body text.
- **Imports** parse and validate the archive first, show what would change, and
  only then apply it inside a single transaction. Credentials are never part of a
  backup, so an import can never take over an account.
- **Exports and imports** share one versioned contract in
  `shared/portable/schema.rs`.