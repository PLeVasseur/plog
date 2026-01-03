# Jekyll Minima Migration Plan

**Status**: Ready for Testing  
**Created**: 2026-01-03  
**Last Updated**: 2026-01-03

## Overview

Migrate the plog GitHub Pages site from bare markdown rendering to Jekyll with the Minima theme, adding:
- Dark mode support
- Persistent navigation (Home | Articles | Media)
- Article dates displayed
- GitHub-style syntax highlighting
- Streamlined homepage with recent items
- Dedicated listing pages for Articles and Media

## Constraints

- **No breaking URLs** - all existing links must continue to work
- **Keep markdown workflow** - continue writing articles in plain markdown
- **Keep README.md** - it serves the GitHub repo page separately from the website

## URL Preservation

| URL | After Migration | Notes |
|-----|-----------------|-------|
| `/` | Served by `index.md` | Works |
| `/articles/022-srs-for-fls-glossary.html` | Same | Jekyll preserves `.html` |
| `/media/uProto-what - GOSIM Beijing 2024.pdf` | Same | Static passthrough |
| `/Pete-LeVasseur-Resume.pdf` | Same | Static passthrough |
| `/pete.png` | Same | Static passthrough |
| **NEW** `/articles/` | Article listing page | New URL |
| **NEW** `/media/` | Media listing page | New URL |

---

## Task Tracking

### Phase 1: Core Jekyll Setup

- [x] **1.1** Create `_config.yml` with Minima theme, dark mode, navigation
- [x] **1.2** Create `assets/css/style.scss` with GitHub-style syntax highlighting

### Phase 2: Layouts

- [x] **2.1** Create `_layouts/post.html` for articles (date, back link)

### Phase 3: Homepage

- [x] **3.1** Create `index.md` with:
  - Intro/bio section
  - Safety-Critical Rust Consortium section
  - Rust Project section
  - Contact section
  - 5 most recent articles (link to full listing)
  - 3 most recent media items (link to full listing)

### Phase 4: Listing Pages

- [x] **4.1** Create `articles/index.md` with full article listing (newest first)
- [x] **4.2** Create `media/index.md` with media organized by type:
  - Talks & Presentations (table with date, title, event, links)
  - Podcasts & Interviews (list format)

### Phase 5: Article Front Matter

Add front matter to all 16 articles with `layout: post`, `title`, and `date`.

Article dates from git history:

| File | Date | Title |
|------|------|-------|
| [x] `002-interop-cpp-rust.md` | 2024-08-21 | Rust and C++ (vsomeip) Interop |
| [x] `003-interop-java-rust.md` | 2024-08-21 | Rust and Java (Java UPClient) Interop |
| [x] `004-writing-zenoh-plugin.md` | 2024-08-20 | Writing a Zenoh Plugin |
| [x] `010-writing-async-rust.md` | 2024-08-23 | Writing Async Rust for the uProtocol uStreamer |
| [x] `011-rust-android-binder-aosp.md` | 2024-08-22 | Using Rust with Java Binder |
| [x] `012-rust-atomics-generate-uuids.md` | 2024-08-23 | Using Rust Atomics for Consistency under High Contention |
| [x] `013-rust-simplify-api-design.md` | 2024-08-23 | Simplifying Rust API Design |
| [x] `014-rust-protobuf-api-guardrails.md` | 2024-08-27 | Guardrails around Valid UUri Protobuf Serialization |
| [x] `015-rust-async-resource-protection.md` | 2024-08-28 | Rust Async Pattern For Resource Protection |
| [x] `016-rust-cpp-proc-macro.md` | 2024-08-30 | Creating a Pool of Callbacks from Rust to C++ with Proc Macros |
| [x] `017-manage-extern-c-fn-pool.md` | 2024-08-30 | Managing a Pool of extern "C" fns |
| [x] `018-gracefully-cpp-interop-drop-impl.md` | 2024-08-30 | Gracefully Winding Down a C++ Library's Resources From Rust |
| [x] `019-trash-git-interactive-rebase.md` | 2024-09-04 | Trashy Commits + Interactive Rebase = Great Git History |
| [x] `020-benefits-of-committing.md` | 2024-10-28 | The Benefits of Committing |
| [x] `021-power-of-spaced-repetition.md` | 2025-08-11 | The Power of Spaced-Repetition Systems |
| [x] `022-srs-for-fls-glossary.md` | 2025-11-13 | Using Spaced-Repetition Systems for Learning the FLS Glossary |

### Phase 6: Verification

- [ ] **6.1** Test locally with Docker
- [ ] **6.2** Verify all existing URLs still work
- [ ] **6.3** Verify dark mode toggle works
- [ ] **6.4** Verify syntax highlighting on code blocks

---

## File Summary

### Files to Create

| File | Purpose |
|------|---------|
| `_config.yml` | Jekyll configuration |
| `index.md` | Homepage |
| `articles/index.md` | Full article listing |
| `media/index.md` | Full media listing |
| `_layouts/post.html` | Article layout |
| `assets/css/style.scss` | Custom CSS overrides |

### Files to Modify

All 16 files in `articles/*.md` - add front matter only (content unchanged)

### Files Unchanged

| File | Reason |
|------|--------|
| `README.md` | Serves GitHub repo page, kept separate |
| `CNAME` | Domain config, unchanged |
| `LICENSE` | Unchanged |
| `pete.png` | Static asset |
| `Pete-LeVasseur-Resume.pdf` | Static asset |
| `media/*.pdf` | Static assets |
| `articles/*-assets/` | Static assets for articles |

---

## Design Decisions

### Homepage Structure

```
[plog]                              [Home] [Articles] [Media]
──────────────────────────────────────────────────────────────

# Pete LeVasseur's Blog [plog]

[photo]

Hey there, I'm Pete...
[bio content]

## Safety-Critical Rust Consortium
[content]

## Rust Project
[content]

## Contact
[content]

## Recent Articles
- [Article 1](link) - Nov 2025
- [Article 2](link) - Aug 2025
- ...
[View all articles →](/articles/)

## Recent Media
- [Talk 1](link) - Dec 2025
- [Talk 2](link) - Jul 2025
- ...
[View all media →](/media/)
```

### Media Page Organization

**Talks & Presentations** (table format):
| Date | Title | Event | Links |
|------|-------|-------|-------|
| Dec 2025 | Rust for Automotive | HAL4SDV | Slides |
| Jul 2025 | Rust in Automotive: Shifting Left | Detroit Rust | Slides, Video |
| Feb 2025 | Self-Referential Structs in Rust | Eclipse SDV | Slides |
| Feb 2025 | My Open Source Story | SEA:ME / 42lisboa | Slides |
| Oct 2024 | uProto-what? | GOSIM Beijing | Slides, Video |

**Podcasts & Interviews** (list format):
- Rustacean Station - Eclipse uProtocol
- Eclipse SDV Blog - Future of open source
- YOUは何しに日本へ? - Side quest

---

## Rollback Plan

If something goes wrong:
1. Delete: `_config.yml`, `index.md`, `articles/index.md`, `media/index.md`, `_layouts/`, `assets/`
2. Revert front matter changes in articles via `git checkout`
3. Site returns to previous bare-markdown rendering

---

## Local Development with Docker

No Ruby installation required. Uses the official Jekyll Docker image.

### First-Time Setup

```bash
# Pull the Jekyll Docker image (one-time)
docker pull jekyll/jekyll:4.0
```

### Running the Local Server

```bash
# From the plog directory root:
docker run --rm -it \
  -v "$PWD":/srv/jekyll \
  -p 4000:4000 \
  jekyll/jekyll:4.0 \
  jekyll serve --host 0.0.0.0 --force_polling
```

Then open `http://localhost:4000` in your browser.

### Notes

- `--rm` removes the container when you stop it (clean)
- `-v "$PWD":/srv/jekyll` mounts your current directory into the container
- `-p 4000:4000` exposes the Jekyll dev server port
- `--force_polling` ensures file changes are detected on all platforms
- Press `Ctrl+C` to stop the server

### Troubleshooting

If you see permission errors:
```bash
docker run --rm -it \
  -v "$PWD":/srv/jekyll \
  -p 4000:4000 \
  -e JEKYLL_UID=$(id -u) \
  -e JEKYLL_GID=$(id -g) \
  jekyll/jekyll:4.0 \
  jekyll serve --host 0.0.0.0 --force_polling
```

---

## Notes

- Minima theme docs: https://github.com/jekyll/minima
- GitHub Pages supported themes: https://pages.github.com/themes/
- Jekyll docs: https://jekyllrb.com/docs/
- Jekyll Docker image: https://hub.docker.com/r/jekyll/jekyll
