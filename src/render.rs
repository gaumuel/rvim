use crossterm::{cursor, queue};
use crossterm::style::{Color, SetForegroundColor, ResetColor};
use crossterm::terminal::{self, ClearType};
use syntect::easy::HighlightLines;
use std::io::{self, Write};

use crate::editor::App;
use crate::mode::Mode;
use crate::style;

impl App {
    /// How many screen rows a buffer line occupies with wrapping.
    pub fn wrapped_line_height(&self, line_idx: usize) -> usize {
        let buf = self.buf();
        if line_idx >= buf.rope.len_lines() { return 1; }
        let line = buf.rope.line(line_idx);
        let len = line.len_chars();
        // Exclude trailing newline
        let char_count = if len > 0 && line.char(len - 1) == '\n' { len - 1 } else { len };
        let cols = self.cols();
        if cols == 0 { return 1; }
        if char_count == 0 { return 1; }
        (char_count + cols - 1) / cols
    }

    /// Compute the screen row of the cursor relative to tab.offset (screen row offset).
    /// Returns (screen_row, screen_col) relative to the viewport.
    pub fn cursor_screen_pos(&self) -> (usize, usize) {
        let tab = self.tab();
        let cols = self.cols();
        let screen_col = if cols == 0 { 0 } else { tab.cx % cols };
        let cursor_wrap_row = if cols == 0 { 0 } else { tab.cx / cols };

        // Count screen rows from line 0 to cursor line
        let mut screen_row: usize = 0;
        for line_idx in 0..tab.cy {
            screen_row += self.wrapped_line_height(line_idx);
        }
        screen_row += cursor_wrap_row;

        // Subtract the scroll offset (which is in screen rows)
        let row_in_viewport = screen_row.saturating_sub(tab.offset);
        (row_in_viewport, screen_col)
    }

    pub fn draw(&self, out: &mut impl Write) -> io::Result<()> {
        queue!(out, cursor::Hide, cursor::MoveTo(0, 0))?;

        // Tab bar
        self.draw_tabs(out)?;

        let tab = self.tab();
        let buf = self.buf();
        let rows = self.rows();
        let cols = self.cols();

        if buf.crashed {
            queue!(out, cursor::MoveTo(0, 1))?;
            queue!(out, SetForegroundColor(Color::Red))?;
            write!(out, "  BUFFER CRASHED")?;
            queue!(out, cursor::MoveTo(0, 3))?;
            queue!(out, SetForegroundColor(Color::White))?;
            write!(out, "  This buffer is in a bad state.")?;
            queue!(out, cursor::MoveTo(0, 4))?;
            write!(out, "  Use :buffer <id> to switch or :ls to list buffers.")?;
            queue!(out, ResetColor)?;
        } else {
            let ext = buf.filename.as_deref().unwrap_or("txt");
            let syntax = self.ss.find_syntax_for_file(ext)
                .ok().flatten()
                .unwrap_or_else(|| self.ss.find_syntax_plain_text());
            let theme = &self.ts.themes[style::THEME_NAME];
            let mut h = HighlightLines::new(syntax, theme);

            // Find which buffer line corresponds to tab.offset screen rows
            let mut screen_row_acc: usize = 0;
            let mut start_line: usize = 0;
            let mut skip_wrap_rows: usize = 0; // how many wrap rows to skip in the first visible line

            for i in 0..buf.rope.len_lines() {
                let lh = self.wrapped_line_height(i);
                if screen_row_acc + lh > tab.offset {
                    start_line = i;
                    skip_wrap_rows = tab.offset - screen_row_acc;
                    break;
                }
                screen_row_acc += lh;
                start_line = i + 1;
            }

            // We need to call highlight_line for lines before start_line to keep highlighter state correct
            for i in 0..start_line {
                let line_str = buf.rope.line(i).to_string();
                let _ = h.highlight_line(&line_str, &self.ss);
            }

            let mut screen_y: usize = 0; // current screen row being drawn

            'outer: for line_idx in start_line..buf.rope.len_lines() {
                if screen_y >= rows { break; }

                let is_cursorline = tab.cursorline && line_idx == tab.cy;
                let line_str = buf.rope.line(line_idx).to_string();

                // Get highlighted segments
                let ranges = match h.highlight_line(&line_str, &self.ss) {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                // Flatten into chars with styles
                let mut chars_with_style: Vec<(char, syntect::highlighting::Style)> = Vec::new();
                for (st, segment) in &ranges {
                    for ch in segment.chars() {
                        if ch == '\n' { continue; }
                        chars_with_style.push((ch, *st));
                    }
                }

                // Split into wrapped rows
                let wrap_rows: Vec<&[(char, syntect::highlighting::Style)]> = if chars_with_style.is_empty() {
                    vec![&[]]
                } else {
                    chars_with_style.chunks(cols).collect()
                };

                for (wrap_idx, wrap_row) in wrap_rows.iter().enumerate() {
                    // Skip rows before the viewport for the first visible line
                    if line_idx == start_line && wrap_idx < skip_wrap_rows {
                        continue;
                    }
                    if screen_y >= rows { break 'outer; }

                    queue!(out, cursor::MoveTo(0, (screen_y + 1) as u16))?;

                    if is_cursorline {
                        queue!(out, crossterm::style::SetBackgroundColor(style::CURSORLINE_BG))?;
                        write!(out, "{}", " ".repeat(cols))?;
                        queue!(out, cursor::MoveTo(0, (screen_y + 1) as u16))?;
                    }

                    for (col_in_wrap, &(ch, st)) in wrap_row.iter().enumerate() {
                        let char_idx_in_line = wrap_idx * cols + col_in_wrap;
                        let in_sel = self.is_in_visual_selection(line_idx, char_idx_in_line);
                        let fg = style::syntect_to_crossterm(st);
                        queue!(out, SetForegroundColor(fg))?;
                        if in_sel {
                            queue!(out,
                                SetForegroundColor(style::VISUAL_FG),
                                crossterm::style::SetBackgroundColor(style::VISUAL_BG),
                            )?;
                        }
                        queue!(out, crossterm::style::Print(ch))?;
                        if in_sel {
                            queue!(out, ResetColor)?;
                        }
                    }

                    queue!(out, ResetColor, terminal::Clear(ClearType::UntilNewLine))?;
                    screen_y += 1;
                }
            }

            // Fill remaining rows with tildes
            while screen_y < rows {
                queue!(out, cursor::MoveTo(0, (screen_y + 1) as u16))?;
                queue!(out, SetForegroundColor(style::TILDE_FG))?;
                write!(out, "~")?;
                queue!(out, ResetColor, terminal::Clear(ClearType::UntilNewLine))?;
                screen_y += 1;
            }
        }

        // Status bar
        let status_row = (rows + 1) as u16;
        queue!(out, cursor::MoveTo(0, status_row))?;
        queue!(out, SetForegroundColor(style::STATUS_FG))?;
        queue!(out, crossterm::style::SetBackgroundColor(style::STATUS_BG))?;
        let mode_str = match tab.mode {
            Mode::Normal => " NORMAL ",
            Mode::Insert => " INSERT ",
            Mode::Visual => " VISUAL ",
            Mode::Command => " COMMAND ",
        };
        let fname = buf.name();
        let buf_info = format!("[buf:{}]", tab.buffer_id);
        let pos = format!(" {}:{} ", tab.cy + 1, tab.cx + 1);
        let pad = cols.saturating_sub(mode_str.len() + fname.len() + buf_info.len() + 2 + pos.len());
        let status = format!("{}{} {} {}{}", mode_str, fname, buf_info, " ".repeat(pad), pos);
        write!(out, "{}", &status[..status.len().min(cols)])?;
        queue!(out, ResetColor)?;

        // Command/message line
        queue!(out, cursor::MoveTo(0, status_row + 1))?;
        if tab.mode == Mode::Command {
            // Draw command palette
            let suggestions = crate::commands::filter_commands(&tab.command_buf);
            let max_suggestions = 10usize;
            let shown: Vec<&str> = suggestions.iter().take(max_suggestions).copied().collect();

            // Row 1: input line
            write!(out, ":{}", tab.command_buf)?;
            queue!(out, terminal::Clear(ClearType::UntilNewLine))?;

            // Row 2: last command
            queue!(out, cursor::MoveTo(0, status_row + 2))?;
            queue!(out, SetForegroundColor(Color::DarkGrey))?;
            if !tab.last_command.is_empty() {
                write!(out, " last: :{}", tab.last_command)?;
            }
            queue!(out, ResetColor, terminal::Clear(ClearType::UntilNewLine))?;

            // Row 3+: suggestions in columns with descriptions
            if !shown.is_empty() {
                let col_width = 34usize;
                let num_cols = (cols / col_width).max(1);
                let cmd_name_width = 16usize;
                let desc_max = col_width - cmd_name_width - 2;

                let mut row_offset = 0u16;

                // Show group index when input is empty
                if tab.command_buf.is_empty() {
                    let groups = [
                        ("f…", "File commands"),
                        ("b…", "Buffer commands"),
                        ("t…", "Tab commands"),
                        ("set…", "Settings"),
                        ("help", "Show help"),
                    ];
                    queue!(out, cursor::MoveTo(0, status_row + 3))?;
                    queue!(out, SetForegroundColor(Color::DarkYellow))?;
                    write!(out, " Groups:")?;
                    queue!(out, ResetColor, terminal::Clear(ClearType::UntilNewLine))?;
                    row_offset = 1;
                    for chunk in groups.chunks(num_cols) {
                        queue!(out, cursor::MoveTo(0, status_row + 3 + row_offset))?;
                        for (prefix, desc) in chunk {
                            queue!(out, SetForegroundColor(Color::DarkGreen))?;
                            write!(out, " :{:<w$}", prefix, w = cmd_name_width)?;
                            let truncated = if desc.len() > desc_max { &desc[..desc_max] } else { *desc };
                            queue!(out, SetForegroundColor(Color::DarkGrey))?;
                            write!(out, "{:<w$}", truncated, w = desc_max)?;
                        }
                        queue!(out, ResetColor, terminal::Clear(ClearType::UntilNewLine))?;
                        row_offset += 1;
                    }
                } else {
                    // Show group header if input matches a group prefix
                    let group_label = match tab.command_buf.as_str() {
                        s if s.starts_with("f") && !s.contains(' ') => Some("── File ──"),
                        s if s.starts_with("b") && !s.contains(' ') => Some("── Buffer ──"),
                        s if s.starts_with("t") && !s.contains(' ') => Some("── Tab ──"),
                        s if s.starts_with("set") => Some("── Settings ──"),
                        _ => None,
                    };

                    if let Some(label) = group_label {
                        queue!(out, cursor::MoveTo(0, status_row + 3))?;
                        queue!(out, SetForegroundColor(Color::DarkYellow))?;
                        write!(out, " {}", label)?;
                        queue!(out, ResetColor, terminal::Clear(ClearType::UntilNewLine))?;
                        row_offset = 1;
                    }

                    for chunk in shown.chunks(num_cols) {
                        queue!(out, cursor::MoveTo(0, status_row + 3 + row_offset))?;
                        for cmd in chunk {
                            queue!(out, SetForegroundColor(Color::DarkGreen))?;
                            write!(out, " :{:<w$}", cmd, w = cmd_name_width)?;
                            let desc = crate::commands::description(cmd);
                            let truncated = if desc.len() > desc_max { &desc[..desc_max] } else { desc };
                            queue!(out, SetForegroundColor(Color::DarkGrey))?;
                            write!(out, "{:<w$}", truncated, w = desc_max)?;
                        }
                        queue!(out, ResetColor, terminal::Clear(ClearType::UntilNewLine))?;
                        row_offset += 1;
                    }
                }
                // Clear any leftover lines below
                let max_palette_rows = self.palette_rows() as u16;
                for extra in row_offset as u16..max_palette_rows {
                    queue!(out, cursor::MoveTo(0, status_row + 3 + extra))?;
                    queue!(out, terminal::Clear(ClearType::UntilNewLine))?;
                }
            } else {
                queue!(out, cursor::MoveTo(0, status_row + 3))?;
                queue!(out, SetForegroundColor(Color::DarkGrey))?;
                write!(out, " (no matches)")?;
                queue!(out, ResetColor, terminal::Clear(ClearType::UntilNewLine))?;
            }
        } else if !tab.status_msg.is_empty() {
            write!(out, "{}", tab.status_msg)?;
            queue!(out, terminal::Clear(ClearType::UntilNewLine))?;
        } else {
            queue!(out, terminal::Clear(ClearType::UntilNewLine))?;
        }

        // Position cursor with wrapping
        if !buf.crashed {
            if tab.mode == Mode::Command {
                queue!(out, cursor::MoveTo(
                    (1 + tab.command_buf.len()) as u16,
                    status_row + 1,
                ))?;
            } else {
                let (cursor_row, cursor_col) = self.cursor_screen_pos();
                queue!(out, cursor::MoveTo(
                    cursor_col as u16,
                    (cursor_row + 1) as u16, // +1 for tab bar
                ))?;
            }
        }

        queue!(out, cursor::Show)?;
        out.flush()
    }

    fn draw_tabs(&self, out: &mut impl Write) -> io::Result<()> {
        queue!(out, cursor::MoveTo(0, 0))?;
        queue!(out, crossterm::style::SetBackgroundColor(Color::DarkGrey))?;
        write!(out, "{}", " ".repeat(self.cols()))?;
        queue!(out, cursor::MoveTo(0, 0))?;

        for (i, tab) in self.tabs.iter().enumerate() {
            let buf = &self.buffers[&tab.buffer_id];
            let name = buf.name();
            let label = if name.len() > 14 { &name[name.len()-14..] } else { name };
            let dirty = if buf.dirty { "+" } else { "" };
            if i == self.active_tab {
                queue!(out, crossterm::style::SetBackgroundColor(Color::Black))?;
                queue!(out, SetForegroundColor(Color::White))?;
            } else {
                queue!(out, crossterm::style::SetBackgroundColor(Color::DarkGrey))?;
                queue!(out, SetForegroundColor(Color::Grey))?;
            }
            write!(out, " {}{} ", label, dirty)?;
        }
        queue!(out, ResetColor)?;
        Ok(())
    }
}
