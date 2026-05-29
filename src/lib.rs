use zed_extension_api::{
    self as zed, Result, SlashCommand, SlashCommandOutput, SlashCommandOutputSection,
};

mod bindings;

use bindings::CATEGORIES;

struct WindowsKeyBindingsExtension;

impl zed::Extension for WindowsKeyBindingsExtension {
    fn new() -> Self {
        Self
    }

    fn complete_slash_command_argument(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<zed::SlashCommandArgumentCompletion>, String> {
        if command.name != "win-keys" {
            return Ok(vec![]);
        }

        let mut completions = vec![
            zed::SlashCommandArgumentCompletion {
                label: "apply".into(),
                new_text: "apply".into(),
                run_command: true,
            },
            zed::SlashCommandArgumentCompletion {
                label: "revert".into(),
                new_text: "revert".into(),
                run_command: true,
            },
            zed::SlashCommandArgumentCompletion {
                label: "list".into(),
                new_text: "list".into(),
                run_command: true,
            },
        ];

        // Add each category as a completion
        for cat in CATEGORIES {
            completions.push(zed::SlashCommandArgumentCompletion {
                label: format!("apply {}", cat.slug),
                new_text: format!("apply {}", cat.slug),
                run_command: true,
            });
        }

        Ok(completions)
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        if command.name != "win-keys" {
            return Err("Unknown command".into());
        }

        let subcommand = args.first().map(|s| s.as_str()).unwrap_or("apply");

        let text = match subcommand {
            "list" => generate_list_output(),
            "revert" => generate_revert_instructions(),
            "apply" => {
                let category_filter: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();
                generate_apply_output(&category_filter)
            }
            // Treat unknown args as category names for apply
            _ => {
                let category_filter: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                generate_apply_output(&category_filter)
            }
        };

        let len = text.len();
        Ok(SlashCommandOutput {
            text,
            sections: vec![SlashCommandOutputSection {
                range: (0..len).into(),
                label: "Windows Key Bindings".into(),
            }],
        })
    }
}

// ---------------------------------------------------------------------------
// Output generators
// ---------------------------------------------------------------------------

fn generate_list_output() -> String {
    let mut out = String::from("# Windows Key Bindings — Available Categories\n\n");
    out.push_str("The following binding categories are available. ");
    out.push_str("Use `/win-keys apply <category>` to apply a specific category, ");
    out.push_str("or `/win-keys apply` to apply all.\n\n");

    for cat in CATEGORIES {
        out.push_str(&format!("## {} [{}]\n", cat.name, cat.slug));
        out.push_str(&format!("{}\n\n", cat.description));
        out.push_str("| Shortcut | Action | Context |\n");
        out.push_str("|----------|--------|---------|\n");
        for b in cat.bindings {
            let ctx = if b.context.is_empty() {
                "global"
            } else {
                b.context
            };
            out.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                b.key,
                b.action_display(),
                ctx
            ));
        }
        out.push('\n');
    }

    let total: usize = CATEGORIES.iter().map(|c| c.bindings.len()).sum();
    out.push_str(&format!(
        "**Total: {} bindings across {} categories.**\n",
        total,
        CATEGORIES.len()
    ));
    out
}

fn generate_revert_instructions() -> String {
    let mut out = String::from("# Windows Key Bindings — Revert Instructions\n\n");
    out.push_str("To revert Windows-style keybindings, you need to remove all JSON blocks \
                  that contain `\"__managed_by\": \"zed-win-keys\"` from the user's keymap.json.\n\n");
    out.push_str("## Keymap locations by platform\n\n");
    out.push_str("- **macOS:** `~/.config/zed/keymap.json`\n");
    out.push_str("- **Linux:** `~/.config/zed/keymap.json`\n");
    out.push_str("- **Windows:** `%APPDATA%\\Zed\\keymap.json`\n\n");
    out.push_str("## Steps\n\n");
    out.push_str("1. Open the keymap file\n");
    out.push_str(
        "2. Find and remove every object in the top-level array that has \
                  `\"__managed_by\": \"zed-win-keys\"`\n",
    );
    out.push_str("3. Preserve all other objects — those are the user's custom bindings\n");
    out.push_str("4. Save the file\n");
    out.push_str("5. The user should restart Zed or run `zed: reload keymap`\n\n");
    out.push_str(
        "**Important:** Do NOT remove objects that don't have the `__managed_by` marker.\n",
    );
    out
}

fn generate_apply_output(category_filter: &[&str]) -> String {
    let selected_categories: Vec<&bindings::Category> = if category_filter.is_empty() {
        CATEGORIES.iter().collect()
    } else {
        let mut matched = Vec::new();
        for slug in category_filter {
            let slug_lower = slug.to_lowercase();
            if let Some(cat) = CATEGORIES.iter().find(|c| c.slug == slug_lower) {
                matched.push(cat);
            }
        }
        if matched.is_empty() {
            CATEGORIES.iter().collect()
        } else {
            matched
        }
    };

    let binding_count: usize = selected_categories.iter().map(|c| c.bindings.len()).sum();
    let cat_names: Vec<&str> = selected_categories.iter().map(|c| c.name).collect();

    let mut out = String::from("# Windows Key Bindings for Zed\n\n");
    out.push_str(&format!(
        "Apply {} Windows-style keybindings ({} categories: {}) to Zed's keymap.\n\n",
        binding_count,
        selected_categories.len(),
        cat_names.join(", ")
    ));

    // Instructions for the Agent
    out.push_str("## Instructions for applying\n\n");
    out.push_str(
        "You are helping the user apply Windows/Linux-style keyboard shortcuts to Zed on macOS. ",
    );
    out.push_str("These replace the default Cmd-based macOS shortcuts with Ctrl-based Windows-style ones.\n\n");
    out.push_str("### Keymap file location\n\n");
    out.push_str("- **macOS:** `~/.config/zed/keymap.json`\n");
    out.push_str("- **Linux:** `~/.config/zed/keymap.json`\n");
    out.push_str("- **Windows:** `%APPDATA%\\Zed\\keymap.json`\n\n");
    out.push_str("### How to apply\n\n");
    out.push_str("1. Read the user's current `keymap.json` (create it if it doesn't exist)\n");
    out.push_str("2. The file should contain a JSON array `[...]` at the top level\n");
    out.push_str("3. If there are existing blocks with `\"__managed_by\": \"zed-win-keys\"`, remove them first (these are from a previous apply)\n");
    out.push_str("4. Append ALL of the JSON blocks listed below to the array\n");
    out.push_str(
        "5. Preserve all existing user blocks that do NOT have the `__managed_by` marker\n",
    );
    out.push_str("6. Write the file back\n");
    out.push_str(
        "7. Tell the user to restart Zed or run `zed: reload keymap` from the command palette\n\n",
    );
    out.push_str("### Important notes\n\n");
    out.push_str("- The `__managed_by` field is a marker so we can identify and update/remove these blocks later\n");
    out.push_str("- NEVER remove blocks that don't have this marker — those are the user's own customizations\n");
    out.push_str("- If the file doesn't exist or is empty, create it with just these blocks inside a `[...]` array\n");
    out.push_str("- If the file has JSONC comments (`//` or `/* */`), preserve them\n\n");

    // The actual JSON blocks
    out.push_str("## Keybinding blocks to add\n\n");
    out.push_str("Add the following JSON blocks to the keymap array:\n\n");
    out.push_str("```json\n");

    // Generate the JSON blocks grouped by context
    let blocks = generate_json_blocks(&selected_categories);
    out.push_str(&blocks);

    out.push_str("\n```\n\n");

    // Category summary
    out.push_str("## Categories included\n\n");
    for cat in &selected_categories {
        out.push_str(&format!(
            "- **{}** ({}): {} ({} bindings)\n",
            cat.name,
            cat.slug,
            cat.description,
            cat.bindings.len()
        ));
    }

    out
}

/// Generate the raw JSON blocks for the keymap, grouped by context.
fn generate_json_blocks(categories: &[&bindings::Category]) -> String {
    use std::collections::BTreeMap;

    // Group all bindings by context
    let mut grouped: BTreeMap<&str, Vec<&bindings::Binding>> = BTreeMap::new();
    for cat in categories {
        for binding in cat.bindings {
            grouped.entry(binding.context).or_default().push(binding);
        }
    }

    let mut blocks: Vec<String> = Vec::new();

    for (context, bindings) in &grouped {
        let mut block = String::from("  {\n");

        if !context.is_empty() {
            block.push_str(&format!("    \"context\": \"{}\",\n", context));
        }

        block.push_str("    \"__managed_by\": \"zed-win-keys\",\n");
        block.push_str("    \"bindings\": {\n");

        let binding_strs: Vec<String> = bindings
            .iter()
            .map(|b| {
                let action_str = match &b.action {
                    bindings::Action::Simple(s) => format!("\"{}\"", s),
                    bindings::Action::WithArg(name, args) => {
                        format!("[\"{}\", {}]", name, args)
                    }
                };
                format!("      \"{}\": {}", b.key, action_str)
            })
            .collect();

        block.push_str(&binding_strs.join(",\n"));
        block.push('\n');
        block.push_str("    }\n");
        block.push_str("  }");

        blocks.push(block);
    }

    format!("[\n{}\n]", blocks.join(",\n"))
}

zed::register_extension!(WindowsKeyBindingsExtension);
