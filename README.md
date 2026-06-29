# Pantry Chef

A small Rust web app for creating recipes from what you already have on hand. Save your pantry staples, pick a meal type, optionally name ingredients you want to use, and let Google Gemini generate a recipe.

Built for personal and family use: one binary, one SQLite file, zero hosting cost.

## Features

- Create an account and log in
- Save always-on-hand pantry staples
- Generate recipes by meal type with optional must-use ingredients and notes
- View saved recipe history
- Powered by Gemini free tier (`gemini-2.0-flash`)

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
cargo run
```

Open [http://localhost:8090](http://localhost:8090), register, add pantry items, and generate a recipe.

## Docker

```bash
cp .env.example .env
# edit .env with your keys
docker compose up --build
```

The SQLite database is stored in `./data`.

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

## License

MIT (or your choice — update as needed)
