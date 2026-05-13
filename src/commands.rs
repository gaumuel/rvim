use std::collections::BTreeSet;

pub fn all_commands() -> BTreeSet<&'static str> {
    BTreeSet::from([
        // File
        "fw", "fq", "fq!", "fx",
        // Buffer
        "bnew", "buffer", "bk", "bl",
        // Tab
        "tnew", "tnext", "tprev", "tclose",
        // Settings
        "set cursorline", "set nocursorline",
        // Help
        "help",
        // Aliases (old commands)
        "w", "q", "q!", "wq", "x",
        "e",
        "kb", "ls",
        "tabnew", "tabnext", "tabprev", "tabclose",
        "gt", "gT",
    ])
}

pub fn description(cmd: &str) -> &'static str {
    match cmd {
        // File group
        "fw" => "Save file",
        "fq" => "Close tab/quit",
        "fq!" => "Force close",
        "fx" => "Save and close",
        // Buffer group
        "bnew" => "New buffer",
        "buffer" => "Switch buffer by ID",
        "bk" => "Kill buffer",
        "bl" => "List buffers",
        // Tab group
        "tnew" => "New tab",
        "tnext" => "Next tab",
        "tprev" => "Previous tab",
        "tclose" => "Close tab",
        // Settings
        "set cursorline" => "Highlight cursor line",
        "set nocursorline" => "Disable cursor highlight",
        // Help
        "help" => "Show help",
        // Aliases
        "w" => "Save (alias: fw)",
        "q" => "Close (alias: fq)",
        "q!" => "Force close (alias: fq!)",
        "wq" | "x" => "Save+close (alias: fx)",
        "e" => "Open file in buffer",
        "kb" => "Kill buffer (alias: bk)",
        "ls" => "List buffers (alias: bl)",
        "tabnew" => "New tab (alias: tnew)",
        "tabnext" | "gt" => "Next tab (alias: tnext)",
        "tabprev" | "gT" => "Prev tab (alias: tprev)",
        "tabclose" => "Close tab (alias: tclose)",
        _ => "",
    }
}

pub fn filter_commands(input: &str) -> Vec<&'static str> {
    all_commands()
        .into_iter()
        .filter(|cmd| cmd.starts_with(input))
        .collect()
}
