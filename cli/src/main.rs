use chrono::Local;
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

mod bindings;

use bindings::{Action, Category, CATEGORIES};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

/// Apply Windows-style keyboard shortcuts to Zed on macOS.
///
/// Replaces Cmd-based macOS defaults with Ctrl-based Windows-style shortcuts
/// so muscle memory from Windows / Linux carries over.
#[derive(Parser)]
#[command(name = "zed-win-keys", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to Zed's keymap.json (auto-detected if omitted)
    #[arg(long, global = true)]
    keymap: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show all available binding categories and their shortcuts
    List {
        /// Show only a specific category
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Apply Windows-style keybindings to Zed's keymap
    Apply {
        /// Apply only specific categories (comma-separated).
        /// Use `list` to see available categories.
        #[arg(short, long, value_delimiter = ',')]
        categories: Option<Vec<String>>,

        /// Skip creating a backup of the current keymap
        #[arg(long)]
        no_backup: bool,

        /// Print the resulting keymap to stdout instead of writing it
        #[arg(long)]
        dry_run: bool,
    },
    /// Revert to the most recent backup
    Revert,
    /// List available backups
    Backups,
}

// ---------------------------------------------------------------------------
// Keymap path resolution
// ---------------------------------------------------------------------------

fn default_keymap_path() -> PathBuf {
    // macOS: ~/Library/Application Support/Zed/keymap.json  (dirs::config_dir → ~/Library/Application Support)
    // Linux: ~/.config/zed/keymap.json
    // Windows: %APPDATA%\Zed\keymap.json
    if let Some(config) = dirs::config_dir() {
        // On macOS dirs::config_dir() returns ~/Library/Application Support
        let zed_dir = config.join("Zed");
        if zed_dir.exists() {
            return zed_dir.join("keymap.json");
        }
        // Linux style
        let zed_lower = config.join("zed");
        if zed_lower.exists() {
            return zed_lower.join("keymap.json");
        }
        // Fallback — use capitalized
        return zed_dir.join("keymap.json");
    }
    // Last resort
    PathBuf::from("keymap.json")
}

fn backup_dir(keymap_path: &PathBuf) -> PathBuf {
    keymap_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("keymap-backups")
}

// ---------------------------------------------------------------------------
// Keymap manipulation
// ---------------------------------------------------------------------------

/// The marker we embed so we can identify our blocks later.
const MARKER_KEY: &str = "__managed_by";
const MARKER_VALUE: &str = "zed-win-keys";

/// An action value ready for JSON serialization — either a string or [name, args].
enum ActionValue {
    Simple(String),
    WithArg(String, Value),
}

impl ActionValue {
    fn to_json(&self) -> Value {
        match self {
            ActionValue::Simple(s) => Value::String(s.clone()),
            ActionValue::WithArg(name, args) => {
                Value::Array(vec![Value::String(name.clone()), args.clone()])
            }
        }
    }
}

fn action_to_value(action: &Action) -> ActionValue {
    match action {
        Action::Simple(s) => ActionValue::Simple(s.to_string()),
        Action::WithArg(name, args_json) => {
            let args: Value = serde_json::from_str(args_json)
                .unwrap_or_else(|_| Value::String(args_json.to_string()));
            ActionValue::WithArg(name.to_string(), args)
        }
    }
}

/// Build a single keymap block for a set of bindings sharing the same context.
fn make_block(context: &str, bindings: &[(&str, ActionValue)]) -> Value {
    let mut map = serde_json::Map::new();
    if !context.is_empty() {
        map.insert("context".into(), Value::String(context.into()));
    }
    // Marker so we can find/remove our blocks
    map.insert(
        MARKER_KEY.into(),
        Value::String(MARKER_VALUE.into()),
    );
    let mut b = serde_json::Map::new();
    for (key, action) in bindings {
        b.insert((*key).into(), action.to_json());
    }
    map.insert("bindings".into(), Value::Object(b));
    Value::Object(map)
}

/// Group raw bindings by context, then produce keymap blocks.
fn build_blocks(categories: &[&Category]) -> Vec<Value> {
    // context → vec of (key, action_value)
    let mut grouped: BTreeMap<&str, Vec<(&str, ActionValue)>> = BTreeMap::new();
    for cat in categories {
        for binding in cat.bindings {
            grouped
                .entry(binding.context)
                .or_default()
                .push((binding.key, action_to_value(&binding.action)));
        }
    }
    grouped
        .into_iter()
        .map(|(ctx, bindings)| make_block(ctx, &bindings))
        .collect()
}

/// Read existing keymap (or return empty array).
fn read_keymap(path: &PathBuf) -> Value {
    match fs::read_to_string(path) {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                Value::Array(vec![])
            } else {
                // Zed's keymap supports // comments via JSONC — strip them
                let stripped = strip_jsonc_comments(trimmed);
                serde_json::from_str(&stripped).unwrap_or_else(|e| {
                    eprintln!(
                        "Warning: couldn't parse existing keymap ({}). Starting fresh.",
                        e
                    );
                    Value::Array(vec![])
                })
            }
        }
        Err(_) => Value::Array(vec![]),
    }
}

/// Minimal JSONC comment stripper (handles // line comments and /* block comments */).
fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;

    while i < len {
        if in_string {
            out.push(chars[i]);
            if chars[i] == '\\' && i + 1 < len {
                i += 1;
                out.push(chars[i]);
            } else if chars[i] == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if chars[i] == '"' {
            in_string = true;
            out.push(chars[i]);
            i += 1;
            continue;
        }

        if chars[i] == '/' && i + 1 < len {
            if chars[i + 1] == '/' {
                // Skip to end of line
                i += 2;
                while i < len && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            } else if chars[i + 1] == '*' {
                // Skip to */
                i += 2;
                while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2; // skip */
                continue;
            }
        }

        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Remove all blocks we previously inserted (identified by marker).
fn remove_managed_blocks(keymap: &mut Vec<Value>) {
    keymap.retain(|block| {
        if let Some(obj) = block.as_object() {
            if let Some(Value::String(v)) = obj.get(MARKER_KEY) {
                return v != MARKER_VALUE;
            }
        }
        true
    });
}

/// Merge our blocks into the keymap: remove old managed blocks, then append new ones.
fn merge_blocks(existing: &Value, new_blocks: Vec<Value>) -> Value {
    let mut arr = match existing {
        Value::Array(a) => a.clone(),
        _ => vec![],
    };
    remove_managed_blocks(&mut arr);
    arr.extend(new_blocks);
    Value::Array(arr)
}

/// Pretty-print keymap JSON with 2-space indent.
fn format_keymap(value: &Value) -> String {
    // serde_json::to_string_pretty uses 2-space indent by default
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "[]".to_string())
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

fn cmd_list(category_filter: Option<String>) {
    let filter = category_filter.as_deref().map(|s| s.to_lowercase());

    for cat in CATEGORIES {
        if let Some(ref f) = filter {
            if !cat.slug.contains(f.as_str()) && !cat.name.to_lowercase().contains(f.as_str()) {
                continue;
            }
        }
        println!("\n  {} [{}]", cat.name, cat.slug);
        println!("  {}", cat.description);
        println!("  {}", "-".repeat(cat.name.len() + cat.slug.len() + 3));
        for b in cat.bindings {
            let ctx = if b.context.is_empty() {
                "".to_string()
            } else {
                format!("  ({})", b.context)
            };
            println!("    {:<24} → {}{}", b.key, b.action_display(), ctx);
        }
    }
    println!();
    println!("  Use `apply -c <slug>,<slug>` to apply specific categories.");
    println!("  Use `apply` with no flags to apply all categories.");
    println!();
}

fn cmd_apply(
    keymap_path: &PathBuf,
    category_slugs: Option<Vec<String>>,
    no_backup: bool,
    dry_run: bool,
) {
    let selected: Vec<&Category> = match category_slugs {
        Some(ref slugs) => {
            let slugs_lower: Vec<String> = slugs.iter().map(|s| s.to_lowercase()).collect();
            let mut matched: Vec<&Category> = Vec::new();
            for slug in &slugs_lower {
                if let Some(cat) = CATEGORIES.iter().find(|c| c.slug == slug.as_str()) {
                    matched.push(cat);
                } else {
                    eprintln!("Unknown category: '{}'. Use `list` to see available categories.", slug);
                    std::process::exit(1);
                }
            }
            matched
        }
        None => CATEGORIES.iter().collect(),
    };

    let blocks = build_blocks(&selected);
    let existing = read_keymap(keymap_path);
    let merged = merge_blocks(&existing, blocks);
    let output = format_keymap(&merged);

    if dry_run {
        println!("{}", output);
        return;
    }

    // Ensure parent directory exists
    if let Some(parent) = keymap_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // Backup
    if !no_backup {
        if keymap_path.exists() {
            let bak_dir = backup_dir(keymap_path);
            fs::create_dir_all(&bak_dir).ok();
            let stamp = Local::now().format("%Y%m%d_%H%M%S");
            let bak_path = bak_dir.join(format!("keymap_{}.json", stamp));
            if let Err(e) = fs::copy(keymap_path, &bak_path) {
                eprintln!("Warning: couldn't create backup: {}", e);
            } else {
                println!("Backup saved to {}", bak_path.display());
            }
        }
    }

    if let Err(e) = fs::write(keymap_path, &output) {
        eprintln!("Error writing keymap: {}", e);
        std::process::exit(1);
    }

    let binding_count: usize = selected.iter().map(|c| c.bindings.len()).sum();
    let cat_names: Vec<&str> = selected.iter().map(|c| c.name).collect();
    println!(
        "Applied {} bindings from {} categories: {}",
        binding_count,
        selected.len(),
        cat_names.join(", ")
    );
    println!("Keymap written to {}", keymap_path.display());
    println!("\nRestart Zed or run `zed: reload keymap` for changes to take effect.");
}

fn cmd_revert(keymap_path: &PathBuf) {
    let bak_dir = backup_dir(keymap_path);
    if !bak_dir.exists() {
        eprintln!("No backups found at {}", bak_dir.display());
        std::process::exit(1);
    }

    let mut backups: Vec<_> = fs::read_dir(&bak_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();

    if backups.is_empty() {
        eprintln!("No backup files found in {}", bak_dir.display());
        std::process::exit(1);
    }

    backups.sort_by_key(|e| e.file_name());
    let latest = backups.last().unwrap();

    if let Err(e) = fs::copy(latest.path(), keymap_path) {
        eprintln!("Error restoring backup: {}", e);
        std::process::exit(1);
    }

    println!("Restored keymap from {}", latest.path().display());
    println!("Restart Zed or run `zed: reload keymap` for changes to take effect.");
}

fn cmd_backups(keymap_path: &PathBuf) {
    let bak_dir = backup_dir(keymap_path);
    if !bak_dir.exists() {
        println!("No backups directory found at {}", bak_dir.display());
        return;
    }

    let mut backups: Vec<_> = fs::read_dir(&bak_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();

    if backups.is_empty() {
        println!("No backup files in {}", bak_dir.display());
        return;
    }

    backups.sort_by_key(|e| e.file_name());
    println!("Backups in {}:", bak_dir.display());
    for entry in &backups {
        let meta = entry.metadata().ok();
        let size = meta.map(|m| m.len()).unwrap_or(0);
        println!("  {} ({} bytes)", entry.file_name().to_string_lossy(), size);
    }
    println!("\nUse `revert` to restore the most recent backup.");
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();
    let keymap_path = cli.keymap.unwrap_or_else(default_keymap_path);

    match cli.command {
        Commands::List { category } => cmd_list(category),
        Commands::Apply {
            categories,
            no_backup,
            dry_run,
        } => cmd_apply(&keymap_path, categories, no_backup, dry_run),
        Commands::Revert => cmd_revert(&keymap_path),
        Commands::Backups => cmd_backups(&keymap_path),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_comments_basic() {
        let input = r#"[
  // this is a comment
  { "key": "value" /* inline */ }
]"#;
        let stripped = strip_jsonc_comments(input);
        let parsed: Value = serde_json::from_str(&stripped).unwrap();
        assert!(parsed.is_array());
    }

    #[test]
    fn strip_comments_preserves_strings() {
        let input = r#"{ "url": "https://example.com" }"#;
        let stripped = strip_jsonc_comments(input);
        let parsed: Value = serde_json::from_str(&stripped).unwrap();
        assert_eq!(
            parsed["url"].as_str().unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn managed_block_roundtrip() {
        let block = make_block("Editor", &[("ctrl-c", ActionValue::Simple("editor::Copy".into()))]);
        let obj = block.as_object().unwrap();
        assert_eq!(obj[MARKER_KEY].as_str().unwrap(), MARKER_VALUE);
    }

    #[test]
    fn action_with_arg_produces_array() {
        let block = make_block("Pane", &[
            ("ctrl-1", ActionValue::WithArg("pane::ActivateItem".into(), serde_json::json!({"index": 0}))),
        ]);
        let bindings = block["bindings"].as_object().unwrap();
        let action = &bindings["ctrl-1"];
        assert!(action.is_array());
        let arr = action.as_array().unwrap();
        assert_eq!(arr[0].as_str().unwrap(), "pane::ActivateItem");
        assert_eq!(arr[1]["index"].as_u64().unwrap(), 0);
    }

    #[test]
    fn remove_managed_preserves_user_blocks() {
        let user_block: Value =
            serde_json::from_str(r#"{"context": "Editor", "bindings": {"cmd-k": "custom"}}"#)
                .unwrap();
        let managed_block = make_block("Editor", &[("ctrl-c", ActionValue::Simple("editor::Copy".into()))]);
        let mut arr = vec![user_block.clone(), managed_block];
        remove_managed_blocks(&mut arr);
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0], user_block);
    }

    #[test]
    fn merge_replaces_old_managed_blocks() {
        let managed_v1 = make_block("Editor", &[("ctrl-c", ActionValue::Simple("editor::Copy".into()))]);
        let existing = Value::Array(vec![managed_v1]);
        let new_blocks = vec![make_block("Editor", &[("ctrl-v", ActionValue::Simple("editor::Paste".into()))])];
        let merged = merge_blocks(&existing, new_blocks);
        let arr = merged.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        let bindings = arr[0]["bindings"].as_object().unwrap();
        assert!(bindings.contains_key("ctrl-v"));
        assert!(!bindings.contains_key("ctrl-c"));
    }

    #[test]
    fn empty_keymap_applies_cleanly() {
        let existing = Value::Array(vec![]);
        let blocks = vec![make_block("", &[("ctrl-s", ActionValue::Simple("workspace::Save".into()))])];
        let merged = merge_blocks(&existing, blocks);
        assert_eq!(merged.as_array().unwrap().len(), 1);
    }
}
