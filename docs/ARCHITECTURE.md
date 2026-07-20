# NClientT → NClientV3 mapping

A reference table so contributors can see exactly which NClientV3 source file
each Rust module was ported from.

| NClientT (this project)                       | NClientV3 original                                                |
|-----------------------------------------------|-------------------------------------------------------------------|
| `src-tauri/src/http.rs`                       | `settings/Global.java` (`initHttpClient`, `reloadHttpClient`), `settings/ApiAuthInterceptor.java`, `components/CustomCookieJar.java` |
| `src-tauri/src/api.rs`                        | `api/InspectorV3.java`, `api/SimpleGallery.java`, `api/components/GalleryData.java`, `api/comments/CommentsFetcher.java`, `loginapi/User.java` |
| `src-tauri/src/models.rs`                     | `api/components/{Gallery,GalleryData,GenericGallery,Page,Tag,TagList,Ranges}.java`, `api/enums/{TagType,TagStatus,SortType,Language,SpecialTagIds,ApiRequestType,ImageType,TitleType}.java` |
| `src-tauri/src/cloudflare.rs`                 | `components/CookieInterceptor.java`, `components/views/CFTokenView.java` |
| `src-tauri/src/downloader.rs`                 | `async/downloader/{GalleryDownloaderManager,GalleryDownloaderV2,DownloadQueue,DownloadObserver,PageChecker}.java` |
| `src-tauri/src/db.rs`                         | `async/database/Queries.java` (+ its sub-tables)                  |
| `src-tauri/src/export.rs`                     | (PDF/ZIP export channels — create_pdf / create_zip)               |
| `src-tauri/src/config.rs`                     | `settings/Global.java` (SharedPreferences bits), `settings/{AuthStore,AuthCredentials,Login,Favorites}.java` |
| `src-tauri/src/commands.rs`                   | (no direct original — bridges frontend ↔ backend, a new layer)    |
| `src-tauri/src/error.rs`                      | (new — unified error type)                                        |

## Key behavioural ports

### User-Agent & auth header (`ApiAuthInterceptor`)
Every request to `/api/v2/...` carries:
- `User-Agent: NClientT/<version> (https://github.com/maxwai/NClientV3)`
- `Authorization: Key <api_key>` (when an API key is set)

The 401/403 → `valid = false` invalidation flag is mirrored by
`AuthCredentials.valid` and `auth_status`.

### Cookie persistence (`CustomCookieJar`)
`reqwest_cookie_store` with a JSON file at `<app_data>/cookies.json`,
reloaded on startup. `cf_clearance` and session cookies survive restarts.

### Search query construction (`InspectorV3#createUrl`)
`api.rs::search` reproduces the v2 URL building:
- `query=<urlencode>` joined with `+` separators
- each tag appended as `+tag:"name"` (or `-tag:"name"` when avoided)
- `&page=N`
- `&sort=<popular\|popular-week\|popular-today\|popular-month>` (when not "recent")

### Random endpoint quirk
`galleries/random` returns just `{"id": N}`; we follow up with a full
`galleries/<id>` request — same as `InspectorV3#doSingleV2`.

### Download folder layout (`GalleryDownloaderV2`)
`<download_dir>/<title>/.<id>` — the `.<id>` marker file lets the local
library scanner identify the gallery and resume downloads without re-fetching.
Pages are saved as `001.<ext>`, `002.<ext>`, ... matching
`PageContainer#getPageName`. A `.nomedia` JSON file stores full metadata.

### JPEG integrity check (`Global#isJPEGCorrupted`)
`config.rs::is_jpeg_corrupted` checks the trailing `FF D9` bytes; the
downloader skips re-downloading pages whose existing file passes the check.
