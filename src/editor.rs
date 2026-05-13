use crossterm::terminal;
use ropey::Rope;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use std::collections::HashMap;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::mode::Mode;

// --- Buffer: shared content ---
pub type BufferId = usize;

pub struct Buffer {
    pub id: BufferId,
    pub rope: Rope,
    pub filename: Option<String>,
    pub dirty: bool,
    pub crashed: bool,
}

impl Buffer {
    pub fn new(id: BufferId, filename: Option<String>) -> Self {
        let rope = match &filename {
            Some(f) => fs::read_to_string(f)
                .map(|s| Rope::from_str(&s))
                .unwrap_or_else(|_| Rope::from_str("")),
            None => Rope::from_str(""),
        };
        Buffer { id, rope, filename, dirty: false, crashed: false }
    }

    pub fn name(&self) -> &str {
        self.filename.as_deref().unwrap_or("[No Name]")
    }

    pub fn save(&mut self) -> String {
        if let Some(ref f) = self.filename {
            match fs::write(f, self.rope.to_string()) {
                Ok(_) => { self.dirty = false; format!("\"{}\" written", f) }
                Err(e) => format!("Error: {}", e),
            }
        } else {
            String::from("No filename")
        }
    }
}

// --- Tab: viewport into a buffer ---
pub struct Tab {
    pub buffer_id: BufferId,
    pub cx: usize,
    pub cy: usize,
    pub offset: usize,
    pub mode: Mode,
    pub command_buf: String,
    pub status_msg: String,
    pub last_command: String,
    pub cursorline: bool,
    pub visual_anchor: (usize, usize),
    pub pending_op: Option<char>,
    pub prev_view: Option<(BufferId, usize, usize, usize)>, // for :help restore
}

impl Tab {
    pub fn new(buffer_id: BufferId) -> Self {
        Tab {
            buffer_id,
            cx: 0,
            cy: 0,
            offset: 0,
            mode: Mode::Normal,
            command_buf: String::new(),
            status_msg: String::new(),
            last_command: String::new(),
            cursorline: false,
            visual_anchor: (0, 0),
            pending_op: None,
            prev_view: None,
        }
    }
}

// --- App: manages everything ---
pub struct App {
    pub buffers: HashMap<BufferId, Buffer>,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub next_buf_id: BufferId,
    pub ss: SyntaxSet,
    pub ts: ThemeSet,
    pub quit: bool,
}

impl App {
    pub fn new(filename: Option<String>) -> Self {
        let mut app = App {
            buffers: HashMap::new(),
            tabs: Vec::new(),
            active_tab: 0,
            next_buf_id: 0,
            ss: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
            quit: false,
        };
        let bid = app.create_buffer(filename);
        app.tabs.push(Tab::new(bid));
        app
    }

    pub fn create_buffer(&mut self, filename: Option<String>) -> BufferId {
        let id = self.next_buf_id;
        self.next_buf_id += 1;
        self.buffers.insert(id, Buffer::new(id, filename));
        id
    }

    pub fn tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub fn tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub fn buf(&self) -> &Buffer {
        &self.buffers[&self.tabs[self.active_tab].buffer_id]
    }

    pub fn buf_mut(&mut self) -> &mut Buffer {
        let bid = self.tabs[self.active_tab].buffer_id;
        self.buffers.get_mut(&bid).unwrap()
    }

    pub fn palette_rows(&self) -> usize {
        // Fixed reservation: last_command + header + up to 5 suggestion rows
        7
    }

    pub fn rows(&self) -> usize {
        let (_, h) = terminal::size().unwrap_or((80, 24));
        let base = h as usize - 3; // tab bar + status + command
        if self.tab().mode == Mode::Command {
            base.saturating_sub(self.palette_rows())
        } else {
            base
        }
    }

    pub fn cols(&self) -> usize {
        let (w, _) = terminal::size().unwrap_or((80, 24));
        w as usize
    }

    pub fn scroll(&mut self) {
        let rows = self.rows();
        let tab = self.tab();
        let cols = self.cols();

        // Compute the cursor's absolute screen row
        let mut cursor_screen_row: usize = 0;
        for i in 0..tab.cy {
            cursor_screen_row += self.wrapped_line_height(i);
        }
        // Add wrap offset within current line
        if cols > 0 {
            cursor_screen_row += tab.cx / cols;
        }

        let offset = self.tab().offset;
        if cursor_screen_row < offset {
            self.tab_mut().offset = cursor_screen_row;
        }
        if cursor_screen_row >= offset + rows {
            self.tab_mut().offset = cursor_screen_row - rows + 1;
        }
    }

    pub fn clamp_cursor(&mut self) {
        let bid = self.tabs[self.active_tab].buffer_id;
        let buf = &self.buffers[&bid];
        let line_count = buf.rope.len_lines().max(1);
        let tab = &mut self.tabs[self.active_tab];

        let max_y = line_count.saturating_sub(1);
        if tab.cy > max_y { tab.cy = max_y; }

        let line_len = {
            if tab.cy < buf.rope.len_lines() {
                let line = buf.rope.line(tab.cy);
                let len = line.len_chars();
                if len > 0 && line.char(len - 1) == '\n' { len - 1 } else { len }
            } else { 0 }
        };
        let max_x = if tab.mode == Mode::Insert { line_len } else { line_len.saturating_sub(1) };
        if tab.cx > max_x { tab.cx = max_x; }
    }

    // Execute a buffer operation with crash isolation
    pub fn with_buffer<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Buffer, &mut Tab) -> R,
    {
        let bid = self.tabs[self.active_tab].buffer_id;
        let buf = self.buffers.get_mut(&bid).unwrap();
        if buf.crashed {
            self.tabs[self.active_tab].status_msg = String::from("Buffer crashed! Switch with :buffer or :ls");
            return None;
        }
        let tab = &mut self.tabs[self.active_tab];
        let result = catch_unwind(AssertUnwindSafe(|| f(buf, tab)));
        match result {
            Ok(r) => Some(r),
            Err(_) => {
                self.buffers.get_mut(&bid).unwrap().crashed = true;
                self.tabs[self.active_tab].status_msg = String::from("Buffer CRASHED! Switch with :buffer <id>");
                None
            }
        }
    }

    // Tab management
    pub fn new_tab(&mut self, filename: Option<String>) {
        let bid = self.create_buffer(filename);
        self.tabs.push(Tab::new(bid));
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() <= 1 {
            self.quit = true;
            return;
        }
        self.tabs.remove(self.active_tab);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = if self.active_tab == 0 { self.tabs.len() - 1 } else { self.active_tab - 1 };
    }

    pub fn kill_buffer(&mut self, bid: BufferId) {
        if !self.buffers.contains_key(&bid) {
            self.tabs[self.active_tab].status_msg = format!("No buffer {}", bid);
            return;
        }
        // Don't kill the last buffer
        if self.buffers.len() <= 1 {
            self.tabs[self.active_tab].status_msg = String::from("Can't kill last buffer");
            return;
        }
        self.buffers.remove(&bid);
        // Find a fallback buffer for any tab that was viewing the killed buffer
        let fallback = *self.buffers.keys().next().unwrap();
        for tab in &mut self.tabs {
            if tab.buffer_id == bid {
                tab.buffer_id = fallback;
                tab.cx = 0;
                tab.cy = 0;
                tab.offset = 0;
                tab.prev_view = None;
                tab.status_msg = format!("Buffer {} killed", bid);
            }
        }
    }

    // Buffer management
    pub fn switch_buffer(&mut self, bid: BufferId) {
        if self.buffers.contains_key(&bid) {
            self.tabs[self.active_tab].buffer_id = bid;
            self.tabs[self.active_tab].cx = 0;
            self.tabs[self.active_tab].cy = 0;
            self.tabs[self.active_tab].offset = 0;
        } else {
            self.tabs[self.active_tab].status_msg = format!("No buffer {}", bid);
        }
    }

    pub fn buffer_list(&self) -> String {
        let current_bid = self.tabs[self.active_tab].buffer_id;
        let mut lines: Vec<String> = vec![String::from("  Buffers:")];
        lines.push(String::from("  ─────────────────────────────────────"));
        let mut ids: Vec<_> = self.buffers.keys().collect();
        ids.sort();
        for &id in &ids {
            let b = &self.buffers[id];
            let active = if *id == current_bid { "%" } else { " " };
            let dirty = if b.dirty { "+" } else { " " };
            let crash = if b.crashed { " CRASHED" } else { "" };
            let lines_count = b.rope.len_lines();
            lines.push(format!("  {}{} {:>3}: {:<30} {:>5} lines{}",
                active, dirty, b.id, b.name(), lines_count, crash));
        }
        lines.push(String::new());
        lines.push(String::from("  % = active in current tab"));
        lines.push(String::from("  + = unsaved changes"));
        lines.push(String::from("  Use :buffer <id> to switch"));
        lines.join("\n")
    }

    // Helper: get current line length for the active tab's buffer
    pub fn current_line_len(&self) -> usize {
        let buf = self.buf();
        let tab = self.tab();
        if tab.cy < buf.rope.len_lines() {
            let line = buf.rope.line(tab.cy);
            let len = line.len_chars();
            if len > 0 && line.char(len - 1) == '\n' { len - 1 } else { len }
        } else { 0 }
    }

    pub fn line_count(&self) -> usize {
        self.buf().rope.len_lines().max(1)
    }

    pub fn is_in_visual_selection(&self, row: usize, col: usize) -> bool {
        let tab = self.tab();
        if tab.mode != Mode::Visual { return false; }
        let buf = self.buf();
        let anchor = buf.rope.line_to_char(tab.visual_anchor.0) + tab.visual_anchor.1;
        let cursor = buf.rope.line_to_char(tab.cy) + tab.cx;
        let (start, end) = if anchor <= cursor {
            (anchor, (cursor + 1).min(buf.rope.len_chars()))
        } else {
            (cursor, (anchor + 1).min(buf.rope.len_chars()))
        };
        let pos = buf.rope.line_to_char(row) + col;
        pos >= start && pos < end
    }
}
