use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::editor::App;
use crate::help;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl App {
    pub fn handle_event(&mut self, ev: Event) {
        let mode = self.tab().mode;
        match mode {
            Mode::Normal => self.handle_normal(ev),
            Mode::Insert => self.handle_insert(ev),
            Mode::Visual => self.handle_visual(ev),
            Mode::Command => self.handle_command(ev),
        }
    }

    fn handle_normal(&mut self, ev: Event) {
        if let Event::Key(KeyEvent { code, .. }) = ev {
            match code {
                KeyCode::Char('h') | KeyCode::Left => {
                    self.tab_mut().cx = self.tab().cx.saturating_sub(1);
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.tab().cy < self.line_count() - 1 {
                        self.tab_mut().cy += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.tab_mut().cy = self.tab().cy.saturating_sub(1);
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if self.tab().cx < self.current_line_len().saturating_sub(1) {
                        self.tab_mut().cx += 1;
                    }
                }
                KeyCode::Char('i') => self.tab_mut().mode = Mode::Insert,
                KeyCode::Char('a') => {
                    let ll = self.current_line_len();
                    let t = self.tab_mut();
                    t.cx = (t.cx + 1).min(ll);
                    t.mode = Mode::Insert;
                }
                KeyCode::Char('o') => {
                    self.with_buffer(|buf, tab| {
                        let idx = buf.rope.line_to_char(tab.cy) + buf.rope.line(tab.cy).len_chars();
                        buf.rope.insert(idx, "\n");
                        buf.dirty = true;
                        tab.cy += 1;
                        tab.cx = 0;
                        tab.mode = Mode::Insert;
                    });
                }
                KeyCode::Char('O') => {
                    self.with_buffer(|buf, tab| {
                        let idx = buf.rope.line_to_char(tab.cy);
                        buf.rope.insert(idx, "\n");
                        buf.dirty = true;
                        tab.cx = 0;
                        tab.mode = Mode::Insert;
                    });
                }
                KeyCode::Char('x') => {
                    self.with_buffer(|buf, tab| {
                        let line_len = {
                            let line = buf.rope.line(tab.cy);
                            let len = line.len_chars();
                            if len > 0 && line.char(len - 1) == '\n' { len - 1 } else { len }
                        };
                        if line_len > 0 {
                            let idx = buf.rope.line_to_char(tab.cy) + tab.cx;
                            buf.rope.remove(idx..idx + 1);
                            buf.dirty = true;
                        }
                    });
                }
                KeyCode::Char('d') => {
                    self.with_buffer(|buf, tab| {
                        let lc = buf.rope.len_lines().max(1);
                        if lc > 1 {
                            let start = buf.rope.line_to_char(tab.cy);
                            let end = if tab.cy + 1 < buf.rope.len_lines() {
                                buf.rope.line_to_char(tab.cy + 1)
                            } else {
                                buf.rope.len_chars()
                            };
                            buf.rope.remove(start..end);
                        } else {
                            let len = buf.rope.len_chars();
                            if len > 0 { buf.rope.remove(0..len); }
                        }
                        buf.dirty = true;
                    });
                }
                KeyCode::Char('0') => self.tab_mut().cx = 0,
                KeyCode::Char('$') => {
                    let ll = self.current_line_len().saturating_sub(1);
                    self.tab_mut().cx = ll;
                }
                KeyCode::Char('G') => {
                    let lc = self.line_count().saturating_sub(1);
                    self.tab_mut().cy = lc;
                }
                KeyCode::Char('g') => self.tab_mut().cy = 0,
                KeyCode::Char('w') => {
                    self.with_buffer(|buf, tab| {
                        let line = buf.rope.line(tab.cy).to_string();
                        let chars: Vec<char> = line.chars().collect();
                        let mut i = tab.cx;
                        while i < chars.len() && !chars[i].is_whitespace() { i += 1; }
                        while i < chars.len() && chars[i].is_whitespace() { i += 1; }
                        let line_len = {
                            let l = buf.rope.line(tab.cy);
                            let len = l.len_chars();
                            if len > 0 && l.char(len - 1) == '\n' { len - 1 } else { len }
                        };
                        tab.cx = i.min(line_len.saturating_sub(1));
                    });
                }
                KeyCode::Char('b') => {
                    self.with_buffer(|buf, tab| {
                        let line = buf.rope.line(tab.cy).to_string();
                        let chars: Vec<char> = line.chars().collect();
                        let mut i = tab.cx.saturating_sub(1);
                        while i > 0 && chars[i].is_whitespace() { i -= 1; }
                        while i > 0 && !chars[i - 1].is_whitespace() { i -= 1; }
                        tab.cx = i;
                    });
                }
                KeyCode::Char(':') => {
                    let t = self.tab_mut();
                    t.mode = Mode::Command;
                    t.command_buf.clear();
                }
                KeyCode::Char('v') => {
                    let t = self.tab_mut();
                    t.visual_anchor = (t.cy, t.cx);
                    t.mode = Mode::Visual;
                }
                _ => {}
            }
        }
    }

    fn handle_insert(&mut self, ev: Event) {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = ev {
            match code {
                KeyCode::Esc => {
                    let t = self.tab_mut();
                    t.mode = Mode::Normal;
                    t.cx = t.cx.saturating_sub(1);
                }
                KeyCode::Char(c) => {
                    if modifiers.contains(KeyModifiers::CONTROL) { return; }
                    self.with_buffer(|buf, tab| {
                        let idx = buf.rope.line_to_char(tab.cy) + tab.cx;
                        buf.rope.insert_char(idx, c);
                        buf.dirty = true;
                        tab.cx += 1;
                    });
                }
                KeyCode::Enter => {
                    self.with_buffer(|buf, tab| {
                        let idx = buf.rope.line_to_char(tab.cy) + tab.cx;
                        buf.rope.insert_char(idx, '\n');
                        buf.dirty = true;
                        tab.cy += 1;
                        tab.cx = 0;
                    });
                }
                KeyCode::Backspace => {
                    self.with_buffer(|buf, tab| {
                        let idx = buf.rope.line_to_char(tab.cy) + tab.cx;
                        if idx > 0 {
                            buf.rope.remove(idx - 1..idx);
                            buf.dirty = true;
                            if tab.cx > 0 {
                                tab.cx -= 1;
                            } else if tab.cy > 0 {
                                tab.cy -= 1;
                                let line = buf.rope.line(tab.cy);
                                let len = line.len_chars();
                                tab.cx = if len > 0 && line.char(len - 1) == '\n' { len - 1 } else { len };
                            }
                        }
                    });
                }
                KeyCode::Left => self.tab_mut().cx = self.tab().cx.saturating_sub(1),
                KeyCode::Right => {
                    if self.tab().cx < self.current_line_len() {
                        self.tab_mut().cx += 1;
                    }
                }
                KeyCode::Up => self.tab_mut().cy = self.tab().cy.saturating_sub(1),
                KeyCode::Down => {
                    if self.tab().cy < self.line_count() - 1 {
                        self.tab_mut().cy += 1;
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_visual(&mut self, ev: Event) {
        if let Event::Key(KeyEvent { code, .. }) = ev {
            match code {
                KeyCode::Esc => self.tab_mut().mode = Mode::Normal,
                KeyCode::Char('h') | KeyCode::Left => {
                    self.tab_mut().cx = self.tab().cx.saturating_sub(1);
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.tab().cy < self.line_count() - 1 { self.tab_mut().cy += 1; }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.tab_mut().cy = self.tab().cy.saturating_sub(1);
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if self.tab().cx < self.current_line_len().saturating_sub(1) {
                        self.tab_mut().cx += 1;
                    }
                }
                KeyCode::Char('0') => self.tab_mut().cx = 0,
                KeyCode::Char('$') => {
                    let ll = self.current_line_len().saturating_sub(1);
                    self.tab_mut().cx = ll;
                }
                KeyCode::Char('G') => {
                    let lc = self.line_count().saturating_sub(1);
                    self.tab_mut().cy = lc;
                }
                KeyCode::Char('g') => self.tab_mut().cy = 0,
                KeyCode::Char('d') | KeyCode::Char('x') => {
                    self.with_buffer(|buf, tab| {
                        let anchor = buf.rope.line_to_char(tab.visual_anchor.0) + tab.visual_anchor.1;
                        let cursor = buf.rope.line_to_char(tab.cy) + tab.cx;
                        let (start, end) = if anchor <= cursor {
                            (anchor, (cursor + 1).min(buf.rope.len_chars()))
                        } else {
                            (cursor, (anchor + 1).min(buf.rope.len_chars()))
                        };
                        buf.rope.remove(start..end);
                        buf.dirty = true;
                        let ci = start.min(buf.rope.len_chars().saturating_sub(1));
                        tab.cy = buf.rope.char_to_line(ci);
                        tab.cx = ci - buf.rope.line_to_char(tab.cy);
                        tab.mode = Mode::Normal;
                    });
                }
                KeyCode::Char('y') => {
                    let t = self.tab_mut();
                    t.status_msg = String::from("yanked");
                    t.mode = Mode::Normal;
                }
                _ => {}
            }
        }
    }

    fn handle_command(&mut self, ev: Event) {
        if let Event::Key(KeyEvent { code, .. }) = ev {
            match code {
                KeyCode::Esc => {
                    let t = self.tab_mut();
                    t.mode = Mode::Normal;
                    t.command_buf.clear();
                }
                KeyCode::Enter => {
                    self.exec_command();
                    self.tab_mut().mode = Mode::Normal;
                }
                KeyCode::Backspace => { self.tab_mut().command_buf.pop(); }
                KeyCode::Char(c) => self.tab_mut().command_buf.push(c),
                _ => {}
            }
        }
    }

    fn exec_command(&mut self) {
        let cmd = self.tab().command_buf.trim().to_string();
        self.tab_mut().last_command = cmd.clone();
        match cmd.as_str() {
            "q" | "q!" => {
                // If viewing a temporary buffer, restore previous view and remove temp buffer
                if let Some((bid, cx, cy, offset)) = self.tab().prev_view {
                    let temp_bid = self.tab().buffer_id;
                    let t = self.tab_mut();
                    t.buffer_id = bid;
                    t.cx = cx;
                    t.cy = cy;
                    t.offset = offset;
                    t.prev_view = None;
                    t.status_msg.clear();
                    // Remove the temporary buffer
                    self.buffers.remove(&temp_bid);
                } else {
                    self.close_tab();
                }
            }
            "w" => {
                let msg = self.buf_mut().save();
                self.tab_mut().status_msg = msg;
            }
            "wq" | "x" => {
                let msg = self.buf_mut().save();
                self.tab_mut().status_msg = msg;
                self.close_tab();
            }
            "help" => self.open_help(),
            // Tab commands
            "tabnew" => self.new_tab(None),
            "tabnext" | "gt" => self.next_tab(),
            "tabprev" | "gT" => self.prev_tab(),
            "tabclose" => self.close_tab(),
            // Buffer commands
            "bnew" => {
                let bid = self.create_buffer(None);
                self.tab_mut().buffer_id = bid;
                self.tab_mut().cx = 0;
                self.tab_mut().cy = 0;
                self.tab_mut().offset = 0;
            }
            "kb" => {
                self.kill_buffer(self.tab().buffer_id);
            }
            "ls" => {
                let list = self.buffer_list();
                // If already in a temp view, just replace content in-place
                if self.tab().prev_view.is_some() {
                    let bid = self.tab().buffer_id;
                    let b = self.buffers.get_mut(&bid).unwrap();
                    b.rope = ropey::Rope::from_str(&list);
                    b.filename = Some("[Buffers]".to_string());
                    let t = self.tab_mut();
                    t.cx = 0;
                    t.cy = 0;
                    t.offset = 0;
                } else {
                    let bid = self.create_buffer(Some("[Buffers]".to_string()));
                    self.buffers.get_mut(&bid).unwrap().rope = ropey::Rope::from_str(&list);
                    let t = self.tab_mut();
                    let prev = (t.buffer_id, t.cx, t.cy, t.offset);
                    t.prev_view = Some(prev);
                    t.buffer_id = bid;
                    t.cx = 0;
                    t.cy = 0;
                    t.offset = 0;
                }
                self.tab_mut().status_msg = String::from("Press :q to close | :buffer <id> to switch");
            }
            "set cursorline" => self.tab_mut().cursorline = true,
            "set nocursorline" => self.tab_mut().cursorline = false,
            _ => {
                if let Some(name) = cmd.strip_prefix("w ") {
                    self.buf_mut().filename = Some(name.to_string());
                    let msg = self.buf_mut().save();
                    self.tab_mut().status_msg = msg;
                } else if let Some(name) = cmd.strip_prefix("tabnew ") {
                    self.new_tab(Some(name.to_string()));
                } else if let Some(name) = cmd.strip_prefix("e ") {
                    // Open file in new buffer, switch current tab to it
                    let bid = self.create_buffer(Some(name.to_string()));
                    let t = self.tab_mut();
                    t.buffer_id = bid;
                    t.cx = 0;
                    t.cy = 0;
                    t.offset = 0;
                } else if let Some(id_str) = cmd.strip_prefix("buffer ") {
                    if let Ok(bid) = id_str.trim().parse::<usize>() {
                        self.switch_buffer(bid);
                    } else {
                        self.tab_mut().status_msg = format!("Invalid buffer id: {}", id_str);
                    }
                } else if let Some(id_str) = cmd.strip_prefix("kb ") {
                    if let Ok(bid) = id_str.trim().parse::<usize>() {
                        self.kill_buffer(bid);
                    } else {
                        self.tab_mut().status_msg = format!("Invalid buffer id: {}", id_str);
                    }
                } else {
                    self.tab_mut().status_msg = format!("Unknown command: {}", cmd);
                }
            }
        }
        self.tab_mut().command_buf.clear();
    }

    fn open_help(&mut self) {
        if self.tab().prev_view.is_some() {
            // Already in a temp view, replace content
            let bid = self.tab().buffer_id;
            let b = self.buffers.get_mut(&bid).unwrap();
            b.rope = ropey::Rope::from_str(help::HELP_TEXT);
            b.filename = Some("[Help]".to_string());
            let t = self.tab_mut();
            t.cx = 0;
            t.cy = 0;
            t.offset = 0;
        } else {
            let bid = self.create_buffer(Some("[Help]".to_string()));
            self.buffers.get_mut(&bid).unwrap().rope = ropey::Rope::from_str(help::HELP_TEXT);
            let t = self.tab_mut();
            let prev = (t.buffer_id, t.cx, t.cy, t.offset);
            t.prev_view = Some(prev);
            t.buffer_id = bid;
            t.cx = 0;
            t.cy = 0;
            t.offset = 0;
        }
        self.tab_mut().status_msg = String::from("Press :q to close help");
    }
}
