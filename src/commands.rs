use std::collections::BTreeSet;

pub fn all_commands() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "q", "q!", "w", "wq", "x",
        "help",
        "tabnew", "tabnext", "tabprev", "tabclose",
        "gt", "gT",
        "e", "bnew", "buffer", "kb", "ls",
        "set cursorline", "set nocursorline",
    ])
}

pub fn description(cmd: &str) -> &'static str {
    match cmd {
        "q" => "Close tab/quit",
        "q!" => "Force close",
        "w" => "Save file",
        "wq" => "Save and close",
        "x" => "Save and close",
        "help" => "Show help",
        "tabnew" => "New tab",
        "tabnext" => "Next tab",
        "tabprev" => "Previous tab",
        "tabclose" => "Close tab",
        "gt" => "Next tab",
        "gT" => "Previous tab",
        "e" => "Open file",
        "bnew" => "New buffer",
        "buffer" => "Switch buffer",
        "kb" => "Kill buffer",
        "ls" => "List buffers",
        "set cursorline" => "Highlight cursor line",
        "set nocursorline" => "Disable cursor highlight",
        _ => "",
    }
}

pub fn filter_commands(input: &str) -> Vec<&'static str> {
    all_commands()
        .into_iter()
        .filter(|cmd| cmd.starts_with(input))
        .collect()
}
