use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
};

pub fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn Error>> {
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

pub fn reset_terminal() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let l = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((r.height - height) / 2),
                Constraint::Length(height),
                Constraint::Min(0),
            ]
        ).split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width - width) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ]).
        split(l[1])[1]
}

pub mod app;
pub mod devmem;
