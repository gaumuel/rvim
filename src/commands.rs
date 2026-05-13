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

pub fn filter_commands(input: &str) -> Vec<&'static str> {
    all_commands()
        .into_iter()
        .filter(|cmd| cmd.starts_with(input))
        .collect()
}
