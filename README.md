# NClientT

[![Release](https://img.shields.io/github/v/release/msprivate67-commits/NClientT?color=green)](https://github.com/msprivate67-commits/NClientT/releases/latest)
[![License](https://img.shields.io/github/license/msprivate67-commits/NClientT?color=blue)](LICENSE)

**NClientT** is an unofficial [nhentai](https://nhentai.net) cross-platform client — browse, search, read, and download doujinshi galleries with a modern native experience. Available on Android, Windows, macOS, and Linux. A full rewrite of [NClientV3](https://github.com/maxwai/NClientV3).

> ⚠️ This is a hobbyist project for personal use only. Respect nhentai's Terms of Service and your local laws.

### Supported Platforms

<p align="center">
  <img src="https://img.shields.io/badge/Android-3DDC84?style=flat-square&logo=android&logoColor=white" alt="Android" />
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

### Platform Notes

| Platform | Extra Dependencies |
|----------|--------------------|
| 🪟 **Windows** | Visual Studio Build Tools ("Desktop development with C++"), WebView2 (preinstalled on Win 10/11) |
| 🍎 **macOS** | `xcode-select --install` |
| 🐧 **Linux** | `webkit2gtk`, `libssl-dev`, `librsvg2`, etc. (see [Tauri docs](https://tauri.app/start/prerequisites/)) |

First build compiles all Rust crates (5–15 minutes); subsequent builds are fast.

## 📂 Project Structure

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

## 🔑 API Key (Optional)

Browsing, searching, reading, and downloading all work without an API key. An API key enables online favorites and comments. Add yours under **Settings → API Key Authentication**.

## ☁️ Cloudflare

When nhentai is behind Cloudflare, a banner prompts you to solve the challenge. Click **Solve now** — a webview opens, complete the captcha, and the `cf_clearance` cookie is captured for all subsequent requests.

## 🔄 Migration from NClientV3

NClientT is a **ground-up rewrite**, not an upgrade. The same API endpoints and download folder layout are used, so downloaded galleries are compatible. Settings and favorites do not migrate automatically — re-add them in NClientT.

For the full module-by-module porting reference, see [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## 📄 License

[Apache-2.0](LICENSE) — same as NClientV3.
