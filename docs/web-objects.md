# XPDE Web Objects (Phase 6b)

Web objects are pinned websites with optional sitemap trees, RSS/Atom feeds, and keyword search routing. All network I/O runs inside **`xpde-shelld`**; shell UI talks D-Bus only.

## Paths

| Purpose | Location |
|--------|----------|
| Pinned site TOML | `~/.config/xpde/web-objects/<id>.toml` |
| User search providers | `~/.config/xpde/search-providers/*.toml` |
| System search providers | `/usr/share/xpde/search-providers/*.toml` (from packages) |
| HTTP response spill | `~/.cache/xpde/web/` |
| Per-site state (optional) | `~/.local/share/xpde/web/<id>/` |
| Generated `.desktop` files | `~/.local/share/applications/xpde-web-<id>.desktop` |

## D-Bus

- **Bus name:** `org.xpde.Shell`
- **Web API:** object path `/org/xpde/Web1`, interface `org.xpde.Web1`
- **Shell API:** object path `/org/xpde/Shell1`, interface `org.xpde.Shell1`

Methods return JSON **strings** for complex types (easy to bind from Tauri later).

### `org.xpde.Web1`

| Method | Args | Returns |
|--------|------|---------|
| `Ping` | — | `s` (`pong`) |
| `ProbeUrl` | `s` url | `s` JSON `ProbeResult` (`xpde-ipc`) |
| `ListPinned` | — | `s` JSON `[SiteSummary]` |
| `PinSite` | `s` JSON `PinSiteSpec` (`xpde-ipc`) | `s` site id |
| `UnpinSite` | `s` site id | — |
| `GetSitemap` | `s` site id, `u` max_depth | `s` JSON `SitemapNode` tree |
| `RefreshSite` | `s` site id | — |
| `Search` | `s` query | `s` JSON `[FederatedHit]` |
| `ResolveAddress` | `s` input | `s` JSON `ResolvedAddress` |
| `OpenSite` | `s` site id, `s` path suffix | — (spawns `xpde-webview`) |
| `ListFeeds` | — | `s` JSON `[FeedSummary]` |
| `FeedItems` | `s` feed id, `u` limit | `s` JSON `[FeedItem]` |

### `org.xpde.Shell1`

| Method | Args | Returns |
|--------|------|---------|
| `Ping` | — | `s` |
| `ListApps` | — | `s` JSON `[AppEntry]` |
| `LaunchDesktop` | `s` desktop id, `s` action | — |

### Examples (`busctl`)

```bash
# After logging into an XPDE session with xpde-shelld running:
busctl --user call org.xpde.Shell /org/xpde/Web1 org.xpde.Web1 ProbeUrl s 'https://www.debian.org'

busctl --user call org.xpde.Shell /org/xpde/Web1 org.xpde.Web1 ListPinned

busctl --user call org.xpde.Shell /org/xpde/Web1 org.xpde.Web1 Search s 'wiki rust'
```

## Search providers

Each `*.toml` file:

```toml
id = "wikipedia"
name = "Wikipedia"
keyword = "wiki"
template = "https://en.wikipedia.org/w/index.php?search={query}"
```

Federated search recognizes `keyword remainder...` (e.g. `wiki rust`) and emits an `open-url:` hit.

## `xpde-webview`

Stub launcher: reads `~/.config/xpde/web-objects/<id>.toml` and opens the URL with the OS default handler (`xdg-open` equivalent via the `open` crate). Optional `--url` overrides the stored URL for deep links.
