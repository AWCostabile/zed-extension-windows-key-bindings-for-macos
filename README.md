# Windows Key Bindings for Zed

Apply Windows/Linux-style keyboard shortcuts (Ctrl-based) to [Zed](https://zed.dev) on macOS, replacing the default Cmd-based bindings. Perfect for developers who switch between Windows/Linux and macOS and want consistent muscle memory.

## What it does

On macOS, Zed uses `Cmd` as the primary modifier (like all Mac apps). This remaps those shortcuts to use `Ctrl` instead, matching the Windows/Linux experience:

- `Ctrl+C` / `Ctrl+V` / `Ctrl+X` for clipboard
- `Ctrl+S` to save, `Ctrl+Z` to undo
- `Ctrl+F` for find, `Ctrl+Shift+F` for project search
- `Ctrl+P` for file finder, `Ctrl+Shift+P` for command palette
- `F12` for go-to-definition, `Ctrl+.` for code actions
- ...and 100 bindings total across 14 categories

## Option 1: Zed Extension (recommended)

The easiest way — install directly from Zed, no external tools needed.

### Install

1. Open Zed
2. Open the extensions panel (`Cmd+Shift+X`)
3. Search for **"Windows Key Bindings"**
4. Click **Install**

### Use

1. Open the Agent panel (`Cmd+?`)
2. Type `/win-keys` and press Enter
3. The Agent will read the generated keybinding data and apply it to your `keymap.json`
4. Restart Zed or run `zed: reload keymap` from the command palette

You can also be selective:

- `/win-keys apply clipboard` — apply only clipboard bindings
- `/win-keys apply navigation find-replace` — apply specific categories
- `/win-keys list` — see all available categories and bindings
- `/win-keys revert` — get instructions to remove the bindings

## Option 2: Standalone CLI

A command-line tool that directly modifies your keymap — no Agent required.

### Install

**Pre-built binaries (no Rust needed):**

```bash
# macOS (Apple Silicon)
curl -L https://github.com/AWCostabile/windows-key-bindings-for-macos/releases/latest/download/zed-win-keys-aarch64-apple-darwin -o zed-win-keys
chmod +x zed-win-keys
sudo mv zed-win-keys /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/AWCostabile/windows-key-bindings-for-macos/releases/latest/download/zed-win-keys-x86_64-apple-darwin -o zed-win-keys
chmod +x zed-win-keys
sudo mv zed-win-keys /usr/local/bin/
```

**From source (any platform):**

```bash
git clone https://github.com/AWCostabile/windows-key-bindings-for-macos.git
cd windows-key-bindings-for-macos/cli
cargo install --path .
```

### Use

```bash
zed-win-keys list                              # Browse all bindings
zed-win-keys apply                             # Apply all bindings
zed-win-keys apply -c clipboard,navigation     # Apply specific categories
zed-win-keys apply --dry-run                   # Preview without writing
zed-win-keys revert                            # Restore from backup
zed-win-keys backups                           # List available backups
```

## Categories

| Slug | Name | Bindings | Description |
|------|------|----------|-------------|
| `clipboard` | Clipboard | 4 | Copy, cut, paste |
| `undo-redo` | Undo / Redo | 3 | Undo and redo |
| `selection` | Selection | 10 | Select all, select next match, multi-cursor, duplicate line |
| `cursor` | Cursor Movement | 14 | Word-level movement, Home/End, delete word |
| `find-replace` | Find & Replace | 7 | Buffer search, project search, find next/previous |
| `files` | File Operations | 8 | Save, save as, open, new file, close tab, reopen tab |
| `navigation` | Navigation | 17 | File finder, command palette, go to line, tab switching |
| `code-editing` | Code Editing | 13 | Comments, indent/outdent, move lines, fold, format, join |
| `code-intel` | Code Intelligence | 8 | Go to definition, references, rename, code actions |
| `panels` | Panels | 7 | Toggle sidebar, explorer, terminal, diagnostics |
| `zoom-layout` | Zoom & Layout | 6 | Font size, split panes, fullscreen |
| `terminal` | Terminal | 4 | Terminal-specific copy, paste, select all |
| `completions` | Completions | 3 | Accept or dismiss autocomplete suggestions |
| `git` | Git | 1 | Git panel toggle |

## How it works

Both the extension and CLI produce the same keybinding blocks. Each block is tagged with `"__managed_by": "zed-win-keys"` so it can be cleanly updated or removed without touching your custom keybindings.

Your existing custom bindings are always preserved.

## Architecture

```
windows-key-bindings-for-macos/
├── extension.toml          # Zed extension manifest (slash command)
├── Cargo.toml              # WASM extension crate (zed_extension_api 0.7)
├── src/
│   ├── lib.rs              # /win-keys slash command implementation
│   └── bindings.rs         # 100 keybinding definitions across 14 categories
├── cli/                    # Standalone CLI (alternative to the extension)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # CLI with apply/revert/list/backups commands
│       └── bindings.rs     # Same binding definitions (shared source)
├── .github/workflows/
│   └── release.yml         # CI for cross-platform CLI binaries
├── README.md
└── LICENSE
```

## License

MIT
