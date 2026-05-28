/// A Zed action — either a plain string or a [name, args] pair.
pub enum Action {
    /// Simple action: `"editor::Copy"`
    Simple(&'static str),
    /// Action with arguments: `["pane::ActivateItem", {"index": 0}]`
    WithArg(&'static str, &'static str),
}

/// A single keybinding: the shortcut, the Zed action, and the context it applies in.
pub struct Binding {
    /// Zed-format key chord, e.g. "ctrl-c", "ctrl-shift-f"
    pub key: &'static str,
    /// Zed action
    pub action: Action,
    /// Zed context filter, e.g. "Editor && mode == full". Empty = global.
    pub context: &'static str,
}

impl Binding {
    /// Human-readable action name for display.
    pub fn action_display(&self) -> String {
        match &self.action {
            Action::Simple(s) => s.to_string(),
            Action::WithArg(name, args) => format!("{} {}", name, args),
        }
    }
}

/// A named group of related bindings.
pub struct Category {
    /// Human-readable name
    pub name: &'static str,
    /// CLI slug for selective apply
    pub slug: &'static str,
    /// Description
    pub description: &'static str,
    /// The bindings in this category
    pub bindings: &'static [Binding],
}

// ===========================================================================
// Binding definitions
//
// These map Windows-style Ctrl-based shortcuts to Zed actions, replacing the
// macOS Cmd-based defaults. The goal: a Windows/Linux user sits down at a Mac
// running Zed and everything works as expected.
//
// Contexts:
//   ""                                   → global (any view)
//   "Editor"                             → any editor view
//   "Editor && mode == full"             → full editor (not inline rename etc.)
//   "Editor && showing_completions"      → autocomplete menu open
//   "Pane"                               → tab/pane level
//   "Workspace"                          → workspace level
//   "Terminal"                           → integrated terminal
//   "ProjectSearchBar > Editor"          → project search input
//   "BufferSearchBar > Editor"           → buffer search input
// ===========================================================================

// Shorthand macros for readability
macro_rules! b {
    ($key:expr, $action:expr, $ctx:expr) => {
        Binding { key: $key, action: Action::Simple($action), context: $ctx }
    };
}

macro_rules! ba {
    ($key:expr, $action:expr, $args:expr, $ctx:expr) => {
        Binding { key: $key, action: Action::WithArg($action, $args), context: $ctx }
    };
}

const CLIPBOARD: &[Binding] = &[
    b!("ctrl-c",        "editor::Copy",                       "Editor"),
    b!("ctrl-x",        "editor::Cut",                        "Editor"),
    b!("ctrl-v",        "editor::Paste",                      "Editor"),
    // Paste without formatting — maps to the standard Ctrl+Shift+V expectation
    b!("ctrl-shift-v",  "editor::Paste",                      "Editor"),
];

const UNDO_REDO: &[Binding] = &[
    b!("ctrl-z",        "editor::Undo",                       "Editor"),
    b!("ctrl-y",        "editor::Redo",                       "Editor"),
    b!("ctrl-shift-z",  "editor::Redo",                       "Editor"),
];

const SELECTION: &[Binding] = &[
    b!("ctrl-a",           "editor::SelectAll",               "Editor"),
    b!("ctrl-d",           "editor::SelectNext",              "Editor && mode == full"),
    b!("ctrl-shift-l",     "editor::SelectAllMatches",        "Editor && mode == full"),
    b!("ctrl-l",           "editor::SelectLine",              "Editor && mode == full"),
    b!("ctrl-shift-k",     "editor::DeleteLine",              "Editor && mode == full"),
    b!("ctrl-alt-up",      "editor::AddSelectionAbove",       "Editor && mode == full"),
    b!("ctrl-alt-down",    "editor::AddSelectionBelow",       "Editor && mode == full"),
    b!("shift-alt-down",   "editor::DuplicateLineDown",       "Editor && mode == full"),
    b!("shift-alt-up",     "editor::DuplicateLineUp",         "Editor && mode == full"),
    b!("ctrl-shift-d",     "editor::DuplicateLineDown",       "Editor && mode == full"),
];

const CURSOR_MOVEMENT: &[Binding] = &[
    b!("ctrl-right",       "editor::MoveToNextWordEnd",           "Editor"),
    b!("ctrl-left",        "editor::MoveToPreviousWordStart",     "Editor"),
    b!("ctrl-shift-right", "editor::SelectToNextWordEnd",         "Editor"),
    b!("ctrl-shift-left",  "editor::SelectToPreviousWordStart",   "Editor"),
    b!("home",             "editor::MoveToBeginningOfLine",       "Editor"),
    b!("end",              "editor::MoveToEndOfLine",             "Editor"),
    b!("shift-home",       "editor::SelectToBeginningOfLine",     "Editor"),
    b!("shift-end",        "editor::SelectToEndOfLine",           "Editor"),
    b!("ctrl-home",        "editor::MoveToBeginning",             "Editor"),
    b!("ctrl-end",         "editor::MoveToEnd",                   "Editor"),
    b!("ctrl-shift-home",  "editor::SelectToBeginning",           "Editor"),
    b!("ctrl-shift-end",   "editor::SelectToEnd",                 "Editor"),
    b!("ctrl-backspace",   "editor::DeleteToPreviousWordStart",   "Editor"),
    b!("ctrl-delete",      "editor::DeleteToNextWordEnd",         "Editor"),
];

const FIND_REPLACE: &[Binding] = &[
    b!("ctrl-f",        "buffer_search::Deploy",              "Editor && mode == full"),
    b!("ctrl-h",        "buffer_search::DeployReplace",       "Editor && mode == full"),
    b!("ctrl-shift-f",  "pane::DeploySearch",                  "Workspace"),
    b!("ctrl-shift-h",  "search::ToggleReplace",              "ProjectSearchBar > Editor"),
    b!("f3",            "search::SelectNextMatch",            "Editor"),
    b!("shift-f3",      "search::SelectPreviousMatch",        "Editor"),
    b!("escape",        "buffer_search::Dismiss",             "BufferSearchBar > Editor"),
];

const FILE_OPERATIONS: &[Binding] = &[
    b!("ctrl-s",        "workspace::Save",                    ""),
    b!("ctrl-shift-s",  "workspace::SaveAs",                  ""),
    b!("ctrl-k s",      "workspace::SaveAll",                 ""),
    b!("ctrl-n",        "workspace::NewFile",                 ""),
    b!("ctrl-o",        "workspace::Open",                    ""),
    b!("ctrl-shift-n",  "workspace::NewWindow",               ""),
    b!("ctrl-w",        "pane::CloseActiveItem",              "Pane"),
    b!("ctrl-shift-t",  "pane::ReopenClosedItem",             "Pane"),
];

const NAVIGATION: &[Binding] = &[
    b!("ctrl-p",           "file_finder::Toggle",             ""),
    b!("ctrl-shift-p",     "command_palette::Toggle",         ""),
    b!("ctrl-g",           "go_to_line::Toggle",              "Editor && mode == full"),
    b!("ctrl-shift-o",     "outline::Toggle",                 "Editor && mode == full"),
    b!("ctrl-tab",         "pane::ActivateNextItem",          "Pane"),
    b!("ctrl-shift-tab",   "pane::ActivatePreviousItem",      "Pane"),
    b!("alt-left",         "pane::GoBack",                    "Pane"),
    b!("alt-right",        "pane::GoForward",                 "Pane"),
    // Tab switching by number — Zed uses ActivateItem with 0-based index arg
    ba!("ctrl-1", "pane::ActivateItem", "{\"index\": 0}", "Pane"),
    ba!("ctrl-2", "pane::ActivateItem", "{\"index\": 1}", "Pane"),
    ba!("ctrl-3", "pane::ActivateItem", "{\"index\": 2}", "Pane"),
    ba!("ctrl-4", "pane::ActivateItem", "{\"index\": 3}", "Pane"),
    ba!("ctrl-5", "pane::ActivateItem", "{\"index\": 4}", "Pane"),
    ba!("ctrl-6", "pane::ActivateItem", "{\"index\": 5}", "Pane"),
    ba!("ctrl-7", "pane::ActivateItem", "{\"index\": 6}", "Pane"),
    ba!("ctrl-8", "pane::ActivateItem", "{\"index\": 7}", "Pane"),
    ba!("ctrl-9", "pane::ActivateItem", "{\"index\": 8}", "Pane"),
];

const CODE_EDITING: &[Binding] = &[
    b!("ctrl-/",           "editor::ToggleComments",          "Editor && mode == full"),
    // Ctrl+Shift+A = block comment toggle (VS Code Windows: Shift+Alt+A)
    b!("shift-alt-a",      "editor::ToggleBlockComments",     "Editor && mode == full"),
    b!("ctrl-]",           "editor::Indent",                  "Editor && mode == full"),
    b!("ctrl-[",           "editor::Outdent",                 "Editor && mode == full"),
    b!("alt-up",           "editor::MoveLineUp",              "Editor && mode == full"),
    b!("alt-down",         "editor::MoveLineDown",            "Editor && mode == full"),
    b!("ctrl-shift-[",     "editor::Fold",                    "Editor && mode == full"),
    b!("ctrl-shift-]",     "editor::UnfoldLines",             "Editor && mode == full"),
    b!("ctrl-enter",       "editor::NewlineBelow",            "Editor && mode == full"),
    b!("ctrl-shift-enter", "editor::NewlineAbove",            "Editor && mode == full"),
    // Windows VS Code uses Shift+Alt+F for format (not Ctrl+Shift+F which is project search)
    b!("shift-alt-f",      "editor::Format",                  "Editor && mode == full"),
    b!("shift-alt-o",      "editor::OrganizeImports",         "Editor && mode == full"),
    b!("ctrl-j",           "editor::JoinLines",               "Editor && mode == full"),
];

const CODE_INTELLIGENCE: &[Binding] = &[
    b!("f12",              "editor::GoToDefinition",          "Editor && mode == full"),
    b!("alt-f12",          "editor::GoToDefinitionSplit",     "Editor && mode == full"),
    b!("shift-f12",        "editor::FindAllReferences",       "Editor && mode == full"),
    b!("f2",               "editor::Rename",                  "Editor && mode == full"),
    b!("ctrl-.",           "editor::ToggleCodeActions",       "Editor && mode == full"),
    b!("ctrl-shift-space", "editor::ShowSignatureHelp",       "Editor && mode == full"),
    b!("ctrl-space",       "editor::ShowCompletions",         "Editor && mode == full"),
    b!("ctrl-k ctrl-i",    "editor::Hover",                  "Editor && mode == full"),
];

const PANELS: &[Binding] = &[
    b!("ctrl-b",           "workspace::ToggleLeftDock",       ""),
    b!("ctrl-shift-e",     "project_panel::ToggleFocus",      ""),
    b!("ctrl-shift-m",     "diagnostics::Deploy",             ""),
    b!("ctrl-`",           "terminal_panel::ToggleFocus",     ""),
    b!("ctrl-shift-`",     "workspace::NewTerminal",          ""),
    b!("ctrl-shift-y",     "workspace::ToggleBottomDock",     ""),
    b!("ctrl-shift-j",     "workspace::ToggleRightDock",      ""),
];

const ZOOM_LAYOUT: &[Binding] = &[
    b!("ctrl-=",           "zed::IncreaseBufferFontSize",     ""),
    b!("ctrl--",           "zed::DecreaseBufferFontSize",     ""),
    b!("ctrl-0",           "zed::ResetBufferFontSize",        ""),
    b!("ctrl-\\",          "pane::SplitRight",                "Pane"),
    b!("ctrl-shift-\\",    "pane::SplitDown",                 "Pane"),
    b!("f11",              "zed::ToggleFullScreen",            ""),
];

const TERMINAL: &[Binding] = &[
    b!("ctrl-c",           "terminal::Copy",                  "Terminal"),
    b!("ctrl-v",           "terminal::Paste",                 "Terminal"),
    b!("ctrl-shift-`",     "workspace::NewTerminal",          "Terminal"),
    b!("ctrl-a",           "terminal::SelectAll",             "Terminal"),
];

const COMPLETIONS: &[Binding] = &[
    b!("enter",            "editor::ConfirmCompletion",       "Editor && showing_completions"),
    b!("tab",              "editor::ConfirmCompletion",       "Editor && showing_completions"),
    b!("escape",           "editor::Cancel",                  "Editor && showing_completions"),
];

const GIT: &[Binding] = &[
    b!("ctrl-shift-g",     "git_panel::ToggleFocus",          ""),
];

// ===========================================================================
// Category registry
// ===========================================================================

pub static CATEGORIES: &[Category] = &[
    Category {
        name: "Clipboard",
        slug: "clipboard",
        description: "Copy, cut, paste",
        bindings: CLIPBOARD,
    },
    Category {
        name: "Undo / Redo",
        slug: "undo-redo",
        description: "Undo and redo",
        bindings: UNDO_REDO,
    },
    Category {
        name: "Selection",
        slug: "selection",
        description: "Select all, select next match, multi-cursor, duplicate line",
        bindings: SELECTION,
    },
    Category {
        name: "Cursor Movement",
        slug: "cursor",
        description: "Word-level movement, Home/End, Ctrl+Home/End, delete word",
        bindings: CURSOR_MOVEMENT,
    },
    Category {
        name: "Find & Replace",
        slug: "find-replace",
        description: "Buffer search, project search, find next/previous",
        bindings: FIND_REPLACE,
    },
    Category {
        name: "File Operations",
        slug: "files",
        description: "Save, open, new file, close tab, reopen tab",
        bindings: FILE_OPERATIONS,
    },
    Category {
        name: "Navigation",
        slug: "navigation",
        description: "File finder, command palette, go to line, tab switching",
        bindings: NAVIGATION,
    },
    Category {
        name: "Code Editing",
        slug: "code-editing",
        description: "Comments, indent/outdent, move lines, fold, newline above/below",
        bindings: CODE_EDITING,
    },
    Category {
        name: "Code Intelligence",
        slug: "code-intel",
        description: "Go to definition, references, rename, code actions, completions",
        bindings: CODE_INTELLIGENCE,
    },
    Category {
        name: "Panels",
        slug: "panels",
        description: "Toggle sidebar, explorer, terminal, diagnostics",
        bindings: PANELS,
    },
    Category {
        name: "Zoom & Layout",
        slug: "zoom-layout",
        description: "Font size, split panes, fullscreen",
        bindings: ZOOM_LAYOUT,
    },
    Category {
        name: "Terminal",
        slug: "terminal",
        description: "Terminal-specific copy, paste, select all",
        bindings: TERMINAL,
    },
    Category {
        name: "Completions",
        slug: "completions",
        description: "Accept or dismiss autocomplete suggestions",
        bindings: COMPLETIONS,
    },
    Category {
        name: "Git",
        slug: "git",
        description: "Git panel toggle",
        bindings: GIT,
    },
];
