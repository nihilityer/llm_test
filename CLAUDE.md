# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

LLM Test is an open, community-driven AI model evaluation platform. Users test models directly from their browser using their own API keys — keys stay in memory, never uploaded to the server. Supports OpenAI, Anthropic, and Gemini API styles.

**Stack:** Vue 3 + TypeScript SPA (Vite) → Cloudflare Pages | Rust WASM (axum + worker-rs) → Cloudflare Workers | Cloudflare D1 (SQLite)

## Common Commands

### Frontend (`front/`)
```bash
npm run dev          # Vite dev server on :5173, proxies /api → :8787
npm run build        # type-check + vite build (runs both in parallel)
npm run type-check   # vue-tsc --build (no emit)
npm run lint         # oxlint + eslint (both with --fix)
npm run format       # prettier --write
```

### Backend (`api/`)
```bash
npx wrangler dev     # Local worker on :8787 (needs wrangler.toml with vars/secrets)
wrangler deploy      # Deploy to Cloudflare Workers
wrangler d1 execute rank-data --file=schema.sql  # Initialize D1 database
```

Both `npm run build` (which includes type-check) and `npm run lint` must pass before submitting PRs. There are **no test frameworks** configured — type-checking and linting are the only automated verification.

## Architecture

### Test execution flow (runs entirely in the browser)

1. User configures API credentials and selects models in `ApiConfigPanel`
2. `engine/runner.ts` iterates over test cases from `test-suite/v1/`, calling `engine/sdk-client.ts` which instantiates official AI SDKs (openai / @anthropic-ai/sdk / @google/generative-ai) with the user's keys
3. Each test case expects the model to invoke a `submit_answer` function with specific arguments; the runner evaluates the function call against expected parameters
4. `engine/scoring.ts` computes scores across 4 dimensions (correctness, first-token latency, token efficiency, consistency)
5. Results are submitted to the backend API via `stores/test.ts` → `api/submissions.ts`

### Auth (dual-path)
- **GitHub OAuth:** Redirects through `/api/auth/github/login` → callback exchanges code for JWT → stored in `localStorage` (30-day expiry)
- **Anonymous:** Cloudflare Turnstile verification → JWT stored in `sessionStorage` (24h expiry)
- Tokens are sent as `Authorization: Bearer <token>` header, or `X-Anonymous-Token` for anonymous endpoints
- The HTTP client (`api/client.ts`) auto-attaches the appropriate token and clears on 401

### LLM proxy (`/api/llm-proxy`)
Optional server-side relay for AI API calls, used when CORS blocks direct browser-to-provider connections. The frontend wraps `fetch` through the proxy endpoint when `useProxy` is enabled. The backend strips hop-by-hop headers and blocks private-host proxying.

### Database (D1, 5 tables)
- `submissions` is the core table (unique per user+website+model or ip+website+model)
- `website_models` joins websites to models with per-website weights
- Rankings calculate weighted composite scores from submission data, applying version weights and submitter-type weights (OAuth=1.0, anonymous=0.7)

### Key frontend patterns
- All API calls go through typed helpers in `api/client.ts` (`apiGet<T>`, `apiPost<T>`)
- Pinia stores (`stores/`) own all mutable state; views are thin orchestration layers
- The test store (`stores/test.ts`) manages a tab-based workflow: API config → test execution → results upload
- Test cases are JSON files in `test-suite/v1/cases/` registered in `test-suite/v1/index.ts`

### Key backend patterns
- `state.rs` holds `AppState` (D1 handle + parsed config from env vars) — available via Axum extension
- DB queries are organized by domain (`db/submissions.rs`, `db/users.rs`, etc.), not by table
- Auth helpers in `handlers/auth.rs` extract and validate JWT, returning typed claims
- `error.rs` maps `ApiError` variants to HTTP status codes via `From<ApiError> for StatusCode`

## Adding Test Cases

1. Create a JSON file in `front/src/test-suite/v1/cases/` following the existing format (must include `id`, `description`, `dimensions`, `expected_function`, `expected_key_arguments`)
2. Register it in `front/src/test-suite/v1/index.ts`
3. Update `CURRENT_TEST_SUITE` in `api/wrangler.toml` if creating a new suite version
