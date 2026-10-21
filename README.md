# Pantry Chef

A small Rust web app for creating recipes from what you already have on hand in your home. Save your pantry staples, pick a meal type, optionally name ingredients you want to use, and generate a recipe.

Built for personal and family use: one binary, one SQLite file, zero hosting cost.

## Features

- Create an account and log in
- Save always-on-hand pantry staples
- Generate recipes by meal type with optional must-use ingredients and notes
- View saved recipe history
- Powered by Gemini free tier (`gemini-2.5-flash`)

## Quick start

Requires **Rust 1.85+** (latest stable recommended).

### 1. Get a free Gemini API key

1. Visit [ai.google.dev](https://ai.google.dev)
2. Sign in and create an API key

### 2. Configure the app

```bash
cp .env.example .env
```

Edit `.env`:

```env
PORT=8080
DATABASE_URL=sqlite:data/recipe_creator.db?mode=rwc
APP_SECRET=replace-with-a-long-random-string
GEMINI_API_KEY=your-gemini-api-key
GEMINI_MODEL=gemini-2.0-flash
```

`APP_SECRET` must be at least 16 characters. It secures login sessions.

### 3. Run locally

```bash
cargo run --bin recipe_creator
```

Open [http://localhost:8090](http://localhost:8090), register, add pantry items, and generate a recipe.

## Docker

```bash
cp .env.example .env
# edit .env with your keys
docker compose up --build
```

The SQLite database is stored in `./data`.

## Deploy to Fly.io (share with friends on any device)

Hosting one copy online gives everyone a permanent `https://` link that works on phones and computers. Friends register their own accounts; recipes use your Gemini key.

```bash
brew install flyctl
fly auth login
fly launch --no-deploy            # reuses fly.toml; pick an app name and region
fly volumes create pantry_data --size 1
fly secrets set APP_SECRET=$(openssl rand -hex 32) GEMINI_API_KEY=your-key INVITE_CODE=pick-a-code
fly deploy
```

Then send your friend the `https://<app>.fly.dev` link and the invite code. On a phone, use **Add to Home Screen** so it opens like an app.

- The SQLite database lives on the `pantry_data` volume, so it survives deploys and restarts. Keep the app on a single machine.
- The machine stops when idle and starts on the next visit, so the first page load after a quiet period takes a few seconds.
- `fly logs` shows server output.
- To reset someone's password on the server: `fly ssh console -C "reset_password <username>"`.

## Project structure

- `src/` — Axum server, auth, pantry, recipes, Gemini client
- `templates/` — Askama HTML templates
- `static/` — CSS
- `migrations/` — SQLite schema

## Sharing with family

Each person can either:

1. Run their own copy with their own `.env` and Gemini key, or
2. Share one instance on a home machine and create separate accounts

Gemini’s free tier is generous for casual personal use. If usage grows, consider giving each household their own API key in `.env`.

## Release builds

```bash
cargo build --release
./target/release/recipe_creator
```

For cross-platform binaries, use GitHub Actions or `cargo-zigbuild` to produce macOS, Linux, and Windows artifacts.

## Environment variables

| Variable | Required | Description |
|----------|----------|-------------|
| `APP_SECRET` | yes | Session signing secret (16+ chars) |
| `GEMINI_API_KEY` | yes | Google Gemini API key |
| `PORT` | no | HTTP port (default `8090`) |
| `DATABASE_URL` | no | SQLite URL (default `sqlite:data/recipe_creator.db?mode=rwc`) |
| `GEMINI_MODEL` | no | Gemini model name (default `gemini-2.0-flash`) |
| `INVITE_CODE` | no | If set, new accounts must enter this code to register |


## Password reset helper (this was a later fix to me forgetting my password)

A small helper binary is provided to reset a local user's password.

- File: [src/bin/reset_password.rs](src/bin/reset_password.rs#L1-L200)

Usage (from the project root):

- Non-interactive (passing password on the command line):
```bash
cargo run --bin reset_password -- <username> <new-password>
```

- Interactive (secure, no-echo prompt — recommended):
```bash
cargo run --bin reset_password -- <username>
# you'll be prompted to enter and confirm the new password
```

Notes:
- The tool reads `DATABASE_URL` from `.env` or defaults to `sqlite:data/recipe_creator.db?mode=rwc`.
- Back up your DB before running:
```bash
cp data/recipe_creator.db data/recipe_creator.db.bak
```
- Interactive mode avoids exposing the password in shell history or process listings.

