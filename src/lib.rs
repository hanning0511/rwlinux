use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hex;
use std::error::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
};

pub fn init_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn Error>> {
    execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

pub fn reset_terminal() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let l = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height - height) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width - width) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(l[1])[1]
}

#[derive(Debug)]
pub struct Cell {
    inner: Option<u8>,
}

impl Cell {
    pub fn hex_str(&self) -> String {
        if self.inner.is_none() {
            String::from("xx")
        } else {
            format!("{:02x}", self.inner.unwrap())
        }
    }
}

#[derive(Debug)]
enum OpMode {
    Normal,
    Jump,
    Edit,
}

struct JumpAddress {
    inner: String,
}

impl JumpAddress {
    fn new(s: &str) -> Self {
        Self {
            inner: String::from(s),
        }
    }

    fn parse(&self, base: u64) -> Option<u64> {
        if self.inner.starts_with("+") {
            let offset = self.inner.strip_prefix("+").unwrap();
            let offset = u64::from_str_radix(offset, 16);
            if let Ok(offset) = offset {
                return Some(base + offset);
            } else {
                return None;
            }
        } else if self.inner.starts_with("-") {
            let offset = self.inner.strip_prefix("-").unwrap();
            let offset = u64::from_str_radix(offset, 16);
            if let Ok(offset) = offset {
                if base >= offset {
                    return Some(base - offset);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            let offset = u64::from_str_radix(&self.inner, 16);
            if let Ok(offset) = offset {
                return Some(offset);
            } else {
                return None;
            }
        }
    }
}

struct WriteValue {
    inner: String,
}

impl WriteValue {
    fn new(s: &str) -> Self {
        Self {
            inner: String::from(s),
        }
    }

    fn parse(&self) -> Option<Vec<u8>> {
        if !self.inner.contains(":") {
            if let Ok(bytes) = hex::decode(&self.inner) {
                return Some(bytes);
            } else {
                return None;
            }
        } else {
            let split_at = self.inner.find(":").unwrap();
            let prefix = self.inner[0..split_at].to_string();
            let value = self.inner[split_at + 1..].to_string();

            match prefix.as_str() {
                "B" => {
                    if let Ok(byte) = u8::from_str_radix(&value, 16) {
                        return Some(vec![byte]);
                    } else {
                        return None;
                    }
                }
                "W" => {
                    if let Ok(data) = u16::from_str_radix(&value, 16) {
                        return Some(data.to_ne_bytes().to_vec());
                    } else {
                        return None;
                    }
                }
                "DW" => {
                    if let Ok(data) = u32::from_str_radix(&value, 16) {
                        return Some(data.to_ne_bytes().to_vec());
                    } else {
                        return None;
                    }
                }
                "QW" => {
                    if let Ok(data) = u64::from_str_radix(&value, 16) {
                        return Some(data.to_ne_bytes().to_vec());
                    } else {
                        return None;
                    }
                }
                "DQW" => {
                    if let Ok(data) = u128::from_str_radix(&value, 16) {
                        return Some(data.to_ne_bytes().to_vec());
                    } else {
                        return None;
                    }
                }
                _ => {}
            }
        }
        None
    }
}

pub mod app;
pub mod devmem;
pub mod io;
