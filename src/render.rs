use crossterm::{cursor, queue};
use crossterm::style::{Color, SetForegroundColor, ResetColor};
use crossterm::terminal::{self, ClearType};
use syntect::easy::HighlightLines;
use std::io::{self, Write};

use crate::editor::App;
use crate::mode::Mode;
use crate::style;

impl App {
    pub fn draw(&self, out: &mut impl Write) -> io::Result<()> {
        queue!(out, cursor::Hide, cursor::MoveTo(0, 0))?;

        // Tab bar
        self.draw_tabs(out)?;

        let tab = self.tab();
        let buf = self.buf();
        let rows = self.rows();
        let cols = self.cols();

        if buf.crashed {
            // Show crash message
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

            for i in 0..rows {
                let file_row = i + tab.offset;
                let is_cursorline = tab.cursorline && file_row == tab.cy;
                queue!(out, cursor::MoveTo(0, (i + 1) as u16))?;

                if is_cursorline {
                    queue!(out, crossterm::style::SetBackgroundColor(style::CURSORLINE_BG))?;
                    write!(out, "{}", " ".repeat(cols))?;
                    queue!(out, cursor::MoveTo(0, (i + 1) as u16))?;
                }

                if file_row < buf.rope.len_lines() {
                    let line_str = buf.rope.line(file_row).to_string();
                    if let Ok(ranges) = h.highlight_line(&line_str, &self.ss) {
                        let mut col = 0usize;
                        for (st, segment) in ranges {
                            for ch in segment.chars() {
                                if ch == '\n' { continue; }
                                if col >= cols { break; }
                                let in_sel = self.is_in_visual_selection(file_row, col);
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
                                col += 1;
                            }
                        }
                        queue!(out, ResetColor)?;
                    }
                } else {
                    queue!(out, SetForegroundColor(style::TILDE_FG))?;
                    write!(out, "~")?;
                }
                queue!(out, ResetColor, terminal::Clear(ClearType::UntilNewLine))?;
            }
        }

        // Status bar
        let status_row = (self.rows() + 1) as u16;
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
            write!(out, ":{}", tab.command_buf)?;
        } else if !tab.status_msg.is_empty() {
            write!(out, "{}", tab.status_msg)?;
        }
        queue!(out, terminal::Clear(ClearType::UntilNewLine))?;

        // Position cursor
        if !buf.crashed {
            queue!(out, cursor::MoveTo(
                tab.cx.min(cols.saturating_sub(1)) as u16,
                (tab.cy - tab.offset + 1) as u16,
            ))?;
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
