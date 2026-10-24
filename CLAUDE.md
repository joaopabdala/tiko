# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Tiko is a Rust (edition 2024) async CLI that downloads TikTok videos (watermark-free, HD) and photo posts. Usage: `tiko <URL_or_ID> ...`; args may be bare IDs, URLs (query strings ignored), or several URLs pasted together without separators. Files are written to the current working directory.

## Commands

```bash
cargo build                      # debug build
cargo build --release            # release binary (target/release/tiko)
cargo run -- <tiktok_url> ...    # run against one or more URLs
cargo test                       # all tests (unit + integration)
cargo test --lib                 # unit tests only (no network to tikwm)
cargo test --test integration_test   # integration tests only
cargo test can_parse             # single test by name
cargo clippy                     # lint
cargo fmt                        # format
```

Test caveats:
- `tests/integration_test.rs` calls the **live** tikwm.com API with hardcoded video/task IDs and an exact expected CDN URL, so these tests are network-dependent, rate-limit sensitive, and can break when upstream data changes. Prefer `cargo test --lib` when verifying local changes.
- Unit tests in `src/downloader.rs` use `mockito` and write/delete a real file (`test_user_12345.mp4`) in the working directory.

## Architecture

Pipeline per input: **extract ID → resolve username/type via tikwm → resolve media URL(s) → stream download to disk**.

- `src/main.rs` — clap `Args { links }`. Each arg is split into URLs with `split_tiktok_urls`, then spawns one task per URL in a `JoinSet`, all sharing a single `Arc<Semaphore>` with **1 permit**. Per-URL errors are logged, not fatal.
- `src/lib.rs` — `download_from_url` orchestrates one input. The semaphore permit is held only during tikwm API calls (each followed by a 1s sleep) to serialize/throttle them; it's released before the actual file download, so downloads run concurrently. Media-URL lookups retry up to 3 times with a 2s backoff; if a video still fails, it falls back to `get_video_url_fallback` (lower quality) and the file gets an `_lq` suffix.
- `src/parser.rs` — `parse_tiktok_id` takes only the ID from a URL (`tiktok.com/@<user>/<type>/<id>`) or a bare numeric ID. Username and item type are deliberately **not** trusted from the URL; `api::resolve_tiktok_info` gets them from tikwm (`data.author.unique_id`, `data.images` present ⇒ photo). Short links (e.g. `vm.tiktok.com`) are not supported.
- `src/types/mod.rs` — `ItemType` (`Video` / `Photo`).
- `src/api.rs` — tikwm.com client (`API_URL`):
  - Video (HD): two-step — POST `video/task/submit` → `task_id`, then GET `video/task/result?task_id=...` → `data.detail.play_url`. tikwm rejects stories here (HTTP 531) and sometimes rate-limits it (`code: -1`).
  - Info / photo / video fallback: single GET `?url=<id>&hd=1` → `data.author`, `data.images[]`, `data.hdplay`/`data.play` (lower quality than the task flow; `data.is_story` is present for stories).
  - Note the TikTok **ID** (not the full URL) is passed as the `url` parameter.
  - `parse_response` turns non-2xx statuses and non-zero JSON `code` into errors naming the step and including the body.
- `src/downloader.rs` — streams response bodies to files named `<username>_<id>.mp4` (`_lq` before `.mp4` for fallback downloads) or `<username>_<id>_<n>.jpeg` (1-based index).

Conventions: errors are `Box<dyn Error + Send + Sync>` throughout (required because work runs in spawned tokio tasks); progress messages go to stderr via `eprintln!`. `api`, `parser`, and `types` are `pub` so integration tests can use them; `downloader` is private.
