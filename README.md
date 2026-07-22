# NClientT

[![License](https://img.shields.io/github/license/maxwai/NClientT?color=blue)](https://github.com/maxwai/NClientT/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.77%2B-orange)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-2.0-67b5d1)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/vue-3.5-42b883)](https://vuejs.org/)

An **unofficial** [nhentai](https://nhentai.net) client — a full cross-platform
rewrite of [NClientV3](https://github.com/maxwai/NClientV3) using
**Tauri 2** (Rust backend) + **Vue 3** (TypeScript frontend).

> NClientV3 was an Android-only app built with OkHttp, JSoup, and Glide.
> NClientT brings the same experience to **Windows**, **macOS**, and **Linux**
> as a native desktop application.

> ⚠️ This is an **unofficial, hobbyist client** for personal use only.
> Respect nhentai's Terms of Service and your local laws.

## Features

- 🏠 **Browse** — recent, popular (today / week / month / all-time)
- 🔍 **Search** — by title, keyword, or tag; include/exclude tags; filter by language and page range
- 🎲 **Random** — discover random galleries
- 📖 **Reader** — fit to width / height / 1:1, RTL mode, keyboard navigation
- ⬇️ **Download manager** — concurrent page downloads, pause / resume / cancel, progress tracking
- ⭐ **Favorites** — local favorites (offline) and online favorites (requires API key)
- 🕓 **History** — automatically records viewed galleries
- 📁 **Local library** — scan downloaded galleries, offline browsing with metadata
- 🏷 **Tags** — cached, searchable, mark as included or excluded
- ☁️ **Cloudflare bypass** — solve challenges in an embedded webview, token reused for all requests
- 🔑 **API key** — authenticates with nhentai for premium features (optional)
- 🍪 **Persistent cookies** — session and CF tokens survive restarts
- ⚙️ **Settings** — mirror, User-Agent, timeouts, grid columns, zoom, RTL, download directory
- 📦 **Export** — convert downloaded galleries to PDF or ZIP
- 🖥️ **Cross-platform** — native builds for Windows, macOS, and Linux

## Quick start

### Prerequisites
- **Rust** (1.77+) — [rustup.rs](https://rustup.rs/)
- **Node.js** (18+) — [nodejs.org](https://nodejs.org/)
- Platform-specific dependencies — see [Tauri prerequisites](https://tauri.app/start/prerequisites/)

```bash
# Clone & install
git clone https://github.com/maxwai/NClientT.git
cd NClientT

# Install frontend dependencies
npm install

# Generate icons
npm run icon

# Development (hot reload)
npm run tauri:dev

# Production build
npm run tauri:build
```

### Platform notes

| Platform | Extra dependencies |
|----------|--------------------|
| **Windows** | Visual Studio Build Tools ("Desktop development with C++"), WebView2 (preinstalled on Win 10/11) |
| **macOS** | `xcode-select --install` |
| **Linux** | `webkit2gtk`, `libssl-dev`, `librsvg2`, etc. (see [Tauri docs](https://tauri.app/start/prerequisites/)) |

First build compiles all Rust crates (5–15 minutes); subsequent builds are fast.

## Project structure

```
NClientT/
├── src/                    # Vue 3 frontend
│   ├── api/                # Rust command wrappers
│   ├── components/         # GalleryCard, TagChip, DownloadItem, ...
│   ├── stores/             # Pinia stores (settings, gallery, downloads, ...)
│   ├── views/              # Home, Search, Gallery, Reader, Favorites, ...
│   └── main.ts
├── src-tauri/              # Rust backend
│   └── src/
│       ├── api.rs          # nhentai API v2 client
│       ├── cloudflare.rs   # Cloudflare challenge solver
│       ├── commands.rs     # Tauri command handlers
│       ├── config.rs       # Settings & auth
│       ├── db.rs           # SQLite (favorites, history, tags, downloads)
│       ├── downloader.rs   # Concurrent download manager
│       ├── export.rs       # PDF / ZIP export
│       ├── http.rs         # reqwest + cookies + auth
│       └── models.rs       # Shared data models
├── docs/ARCHITECTURE.md    # NClientV3 → NClientT porting reference
└── package.json
```

## API key (optional)

Browsing, searching, reading, and downloading all work without an API key.
An API key enables online favorites and comments.

Add yours under **Settings → API Key Authentication**.

## Cloudflare

When nhentai is behind Cloudflare, a banner prompts you to solve the challenge.
Click **Solve now** — a webview opens, complete the captcha, and the
`cf_clearance` cookie is captured for all subsequent requests.

## Migration from NClientV3

NClientT is a **ground-up rewrite**, not an upgrade. The same API endpoints and
download folder layout are used, so downloaded galleries are compatible.
Settings and favorites do not migrate automatically — re-add them in NClientT.

For the full module-by-module porting reference, see [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## License

[Apache-2.0](LICENSE) — same as NClientV3.
