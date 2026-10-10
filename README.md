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

## Deploying to Vercel

The repository is configured as a Vercel project with two services, in
`vercel.json`:

- `backend` — the Rust API, runtime `rust`, served from `api/handler.rs`
- `frontend` — the Vite app

Public routing lives only at the top level: `/api/(.*)` goes to `backend`,
everything else to `frontend`. Vercel forwards the original path, so the API
receives `/api/...` and serves it unchanged — the frontend needs no
`VITE_API_BASE_URL` in this setup, since it calls the same domain.

There are **no service bindings**. Bindings inject a service's internal URL into
another service's *server-side* code; here the frontend is a static browser app
and the browser calls the public `/api` path directly, so a binding would have
nothing to bind.

`api/handler.rs` reuses the same router as `main.rs`; only the process entry
differs. `main.rs` binds a TCP listener for a normal host, `api/handler.rs` goes
through Vercel's function bridge.

**Environment variables for the backend service:**

```bash
DATABASE_URL=postgres://...            # Vercel Postgres or any provider
STORAGE_ROOT=/tmp                     # the only writable path in a function
COOKIE_SAME_SITE=Lax                  # same domain, so Lax is correct
ALLOWED_ORIGINS=https://your-app.vercel.app
```

**Known limits of this deployment.** Vercel functions have a read-only
filesystem with scratch space in `/tmp` (500 MB, ephemeral), so uploaded
attachment bytes do not survive an instance recycle. Request bodies are capped
well below what `MAX_ARCHIVE_BYTES` (64 MB) allows, so lower that setting for
imports if the deployment rejects large files. Migrations still have to be
applied out of band with `cargo run --bin migrate`; a function never changes the
schema on startup. The Rust runtime is in public beta.

For a diary that keeps attachments durably, deploying the backend to a host with
a persistent volume and the frontend to Vercel remains the better shape — see
the two-origin cookie settings in `backend/.env.example`.

`vercel dev` runs both services together locally.

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