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
  stored, and the cookie is `HttpOnly` + `SameSite=Lax`. Mutating requests must
  echo the `dmd_csrf` cookie in an `X-CSRF-Token` header.
- **Entries** allow one per account per day. Word counts are computed on write so
  reflection statistics never rescan the body text.
- **Imports** parse and validate the archive first, show what would change, and
  only then apply it inside a single transaction. Credentials are never part of a
  backup, so an import can never take over an account.
- **Exports and imports** share one versioned contract in
  `shared/portable/schema.rs`.