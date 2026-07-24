# NClientT

[English](README.md) | [简体中文](README_zh.md)

[![Release](https://img.shields.io/github/v/release/msprivate67-commits/NClientT?color=green)](https://github.com/msprivate67-commits/NClientT/releases/latest)
[![License](https://img.shields.io/github/license/msprivate67-commits/NClientT?color=blue)](LICENSE)

**NClientT** is an unofficial [nhentai](https://nhentai.net) cross-platform client — browse, search, read, and download doujinshi galleries with a modern native experience. Available on Android, Windows, macOS, and Linux. A full rewrite of [NClientV3](https://github.com/maxwai/NClientV3).

> ⚠️ This is a hobbyist project for personal use only. Respect nhentai's Terms of Service and your local laws.

### Supported Platforms

<p align="center">
  <img src="https://img.shields.io/badge/Android-7.0%2B-3DDC84?style=flat-square&logo=android&logoColor=white" alt="Android 7.0+" />
  <img src="https://img.shields.io/badge/Windows-10%2B-0078D6?style=flat-square&logo=windows&logoColor=white" alt="Windows" />
  <img src="https://img.shields.io/badge/macOS-12%2B-000000?style=flat-square&logo=apple&logoColor=white" alt="macOS" />
  <img src="https://img.shields.io/badge/Linux-x86__64-FCC624?style=flat-square&logo=linux&logoColor=black" alt="Linux" />
</p>

### Tech Stack

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.77%2B-DEA584?style=flat-square&logo=rust&logoColor=black" alt="Rust" />
  <img src="https://img.shields.io/badge/Vue-3.5-4FC08D?style=flat-square&logo=vuedotjs&logoColor=white" alt="Vue 3" />
  <img src="https://img.shields.io/badge/Tauri-2.0-24C8DB?style=flat-square&logo=tauri&logoColor=white" alt="Tauri" />
  <img src="https://img.shields.io/badge/TypeScript-5.6-3178C6?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/i18n-multi--lang-8B8B8B?style=flat-square&logo=weblate&logoColor=white" alt="i18n" />
</p>

### Download Link

<p align="center">
  <a href="https://github.com/msprivate67-commits/NClientT/releases/latest">
    <strong>📥 Download Latest Release</strong>
  </a>
</p>

---

## 📱 Phone Screenshots

<div align="center">
  <img src="screenshoot/gallery.jpg" alt="Gallery" width="30%" />
  <img src="screenshoot/manga-info-page.jpg" alt="Manga Info" width="30%" />
  <img src="screenshoot/manga-info-thumbs.jpg" alt="Thumbnail View" width="30%" />
</div>

<div align="center">
  <img src="screenshoot/reader-phone-adaptive.jpg" alt="Phone Reader" width="60%" />
</div>

## 📟 Tablet Screenshots

<div align="center">
  <img src="screenshoot/reader-pad-adaptive.jpg" alt="Tablet Reader" width="60%" />
  <br/>
  <img src="screenshoot/setings-pad.jpg" alt="Tablet Settings" width="60%" />
</div>

---

### Download Link

<p align="center">
  <a href="https://github.com/msprivate67-commits/NClientT/releases/latest">
    <strong>📥 Download Latest Release</strong>
  </a>
</p>

---

## ✨ Features

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

---

## 🛠️ For Developers

### 🚀 Quick Start

#### Prerequisites

- 🦀 **Rust** (1.77+) — [rustup.rs](https://rustup.rs/)
- 📦 **Node.js** (18+) — [nodejs.org](https://nodejs.org/)
- Platform-specific dependencies — see [Tauri prerequisites](https://tauri.app/start/prerequisites/)

```bash
# Clone & install
git clone https://github.com/msprivate67-commits/NClientT.git
cd NClientT

# Install frontend dependencies
npm ci

# Development (hot reload)
npm run tauri:dev

# Production build
npm run tauri:build
```

For frontend-only work, use `npm run dev`. Before submitting changes, run:

```bash
npm run typecheck
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Use `npm run icon` only when regenerating application icons.

### Platform Notes

| Platform | Extra Dependencies |
|----------|--------------------|
| 🪟 **Windows** | Visual Studio Build Tools ("Desktop development with C++"), WebView2 (preinstalled on Win 10/11) |
| 🍎 **macOS** | `xcode-select --install` |
| 🐧 **Linux** | `webkit2gtk`, `libssl-dev`, `librsvg2`, etc. (see [Tauri docs](https://tauri.app/start/prerequisites/)) |

First build compiles all Rust crates (5–15 minutes); subsequent builds are fast.

### Building for legacy Android devices

Official Android releases target modern 64-bit ARM devices (`arm64-v8a`). For
older 32-bit ARM phones and tablets, build the `armeabi-v7a` APK locally. The
legacy build still requires Android 7.0 or newer because the project's minimum
SDK is API 24.

Install the Android prerequisites from the
[Tauri documentation](https://tauri.app/start/prerequisites/), set
`ANDROID_HOME` (and your NDK environment variable), then run:

```bash
npm ci
npm run android:build:legacy
```

The script installs the Rust armv7 target when needed, builds a release APK,
signs it with `src-tauri/nclientt.keystore` when available, verifies the
signature, and writes the result to:

```text
artifacts/NClientT-<version>-android-armeabi-v7a.apk
```

To use another signing key, set `ANDROID_KEYSTORE_PATH`,
`ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, and optionally
`ANDROID_KEY_PASSWORD` before running the command. If no signing tools or
keystore are available, the script keeps an `-unsigned.apk` artifact instead.

## 📂 Project Structure

```
NClientT/
├── src/                    # Vue 3 frontend
│   ├── api/                # Domain-based Tauri/service gateway
│   ├── components/         # GalleryCard, TagChip, DownloadItem, ...
│   ├── composables/        # Reusable Vue and browser interactions
│   ├── i18n/               # Locale setup and translations
│   ├── stores/             # Pinia stores (settings, gallery, downloads, ...)
│   ├── types/              # Shared frontend domain contracts
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
├── docs/
│   ├── ARCHITECTURE.md     # NClientV3 → NClientT porting reference
│   └── FRONTEND_ARCHITECTURE.md # Frontend layers and module ownership
├── AGENTS.md               # Contributor guidelines
└── package.json
```

Frontend consumers should import backend operations from `@/api`; its public
barrel keeps call sites stable while implementations remain grouped by domain.
See [`docs/FRONTEND_ARCHITECTURE.md`](docs/FRONTEND_ARCHITECTURE.md) for the
dependency rules and module responsibilities.

## 🔑 API Key (Optional)

Browsing, searching, reading, and downloading all work without an API key. An API key enables online favorites and comments. Add yours under **Settings → API Key Authentication**.

## ☁️ Cloudflare

When nhentai is behind Cloudflare, a banner prompts you to solve the challenge. Click **Solve now** — a webview opens, complete the captcha, and the `cf_clearance` cookie is captured for all subsequent requests.

## 🔄 Migration from NClientV3

NClientT is a **ground-up rewrite**, not an upgrade. The same API endpoints and download folder layout are used, so downloaded galleries are compatible. Settings and favorites do not migrate automatically — re-add them in NClientT.

For the full module-by-module porting reference, see [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## 📄 License

[Apache-2.0](LICENSE) — same as NClientV3.
