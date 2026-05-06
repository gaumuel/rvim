mod editor;
mod help;
mod mode;
mod render;
mod style;

use crossterm::{
    cursor, execute,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};
use std::io::{self, stdout};
use std::env;

fn main() -> io::Result<()> {
    let filename = env::args().nth(1);
    let mut app = editor::App::new(filename);
    let mut out = stdout();

    terminal::enable_raw_mode()?;
    execute!(out, terminal::EnterAlternateScreen, cursor::Show)?;

    loop {
        app.clamp_cursor();
        app.scroll();
        app.draw(&mut out)?;

        if let Ok(ev) = event::read() {
            if let Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. }) = ev {
                break;
            }
            app.handle_event(ev);
        }

        if app.quit { break; }
    }

    terminal::disable_raw_mode()?;
    execute!(out, terminal::LeaveAlternateScreen)?;
    Ok(())
}
