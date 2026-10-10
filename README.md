# Dear My Diary

A local-first diary: a Rust backend, a React frontend, and PostgreSQL for storage.

The application is a **modular monolith**. Business features own their routes,
handlers, services, repositories and API shapes; nothing reaches into another
feature's tables.

## Layout

```text
backend/
  migrations/          versioned schema, applied by the migrate binary
  src/
    main.rs            bootstrapping only
    app.rs             router composition
    config.rs          environment configuration
    state.rs           shared runtime dependencies
    shared/            errors, pagination, dates, validation, portable format
    infrastructure/    database, sessions, file storage, telemetry
    middleware/        authentication, csrf, rate limit
    features/          auth, accounts, entries, tags, moods, calendar,
                       attachments, imports, exports, search, insights, settings
  tests/               feature-grouped tests

frontend/
  src/
    app/               router, providers, route guards
    components/        ui, layout, feedback
    features/          same feature split as the backend
    hooks/, lib/       shared hooks and the API client
    styles/            tokens, themes, component styles
```

## Running it

Prerequisites: Rust 1.82+, Node 20+, PostgreSQL 14+.

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

**4. Frontend.** Vite proxies `/api` to the backend, so cookies just work.

```bash
cd ../frontend
npm install
npm run dev
```

Open http://localhost:5173 and create an account.

## Deploying the frontend to Vercel

Importing the repository works with no project settings to change. The
`vercel.json` at the repository root points Vercel at `frontend/`: install
`npm install`, build `npm run build`, serve `frontend/dist`, and rewrite unknown
paths to `index.html` so deep links like `/settings` survive a refresh.

1. In Vercel, **Add New → Project** and import this repository. Leave the root
   directory as the repository root and the framework as it is detected.
2. Set one environment variable:

   ```bash
   VITE_API_BASE_URL=https://your-api-host.example.com
   ```

   Leave it empty when the API serves the built frontend from the same origin.

`.vercelignore` keeps build output out of the upload, so importing does not ship
the Rust `target` directory.

The API itself is a Rust server that needs PostgreSQL and a persistent volume for
attachment bytes, so it belongs on a normal host (a VPS, Fly.io, Railway), not on
Vercel's serverless runtime. When the two run on different sites, cookies have to
cross origins. Configure the API for that:

```bash
ALLOWED_ORIGINS=https://your-app.vercel.app
COOKIE_SAME_SITE=None
COOKIE_SECURE=auto     # auto means: Secure exactly when SameSite=None
```

`SameSite=None` is only accepted by browsers over HTTPS, so the API must be
served over TLS in this setup. Add your custom domain to `ALLOWED_ORIGINS` when
you have one.

## Checks

```bash
cd backend
cargo fmt
cargo clippy --all-targets
cargo test

cd ../frontend
npm run build      # tsc --noEmit && vite build
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