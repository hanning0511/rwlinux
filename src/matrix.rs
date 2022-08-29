use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use hex;
use std::error::Error;
use std::io;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Clear, Paragraph};
use tui::{Frame, Terminal};
use unicode_width::UnicodeWidthStr;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

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

/// Initializes the terminal.
pub fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    Ok(terminal)
}

/// Resets the terminal.
pub fn reset_terminal() -> Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

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

pub struct Cell {
    pub inner: Option<u8>,
}

impl Cell {
    pub fn hex(&self) -> String {
        match self.inner {
            Some(byte) => format!("{:02x}", byte),
            None => String::from("xx"),
        }
    }

    pub fn ascii(&self) -> String {
        match self.inner {
            Some(byte) => {
                if byte.is_ascii() {
                    return (byte as char).to_string();
                } else {
                    return String::from(".");
                }
            }
            None => String::from("."),
        }
    }
}

pub trait MatrixData {
    fn new(size: u16) -> Self;
    fn write(&self, offset: u64, bytes: Vec<u8>);
    fn update(&mut self, start: u64);
    fn get(&self, index: usize) -> Option<Cell>;
}
pub enum OpMode {
    Normal,
    Jump,
    Write,
}
pub struct Matrix<T> {
    pub name: String,
    pub cols: u16,
    pub rows: u16,
    pub offset: u64,
    pub data: T,
    pub op_mode: OpMode,
    pub input: String,
}

impl<T: MatrixData> Matrix<T> {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            cols: 16,
            rows: 16,
            offset: 0,
            data: T::new(16 * 16),
            op_mode: OpMode::Normal,
            input: String::new(),
        }
    }

    pub fn page_size(&self) -> u64 {
        (self.cols * self.rows) as u64
    }

    pub fn page_offset(&self) -> u64 {
        self.offset % self.page_size()
    }

    pub fn page_start(&self) -> u64 {
        self.offset - self.page_offset()
    }

    pub fn next_byte(&mut self) {
        self.offset += 1;
        if self.page_offset() == 0 {
            self.data.update(self.page_start());
        }
    }

    pub fn prev_byte(&mut self) {
        if self.offset >= 1 {
            self.offset -= 1;
        }

        if self.page_offset() + 1 == self.page_size() {
            self.data.update(self.page_start())
        }
    }

    pub fn next_line(&mut self) {
        self.offset += self.cols as u64;
        if self.page_offset() <= self.cols as u64 {
            self.data.update(self.page_start());
        }
    }

    pub fn prev_line(&mut self) {
        if self.offset >= self.cols as u64 {
            self.offset -= self.cols as u64;
        }
        if self.page_offset() >= self.page_size() - self.cols as u64 {
            self.data.update(self.page_start());
        }
    }

    pub fn next_page(&mut self) {
        self.offset += self.page_size();
        self.data.update(self.page_start());
    }

    pub fn prev_page(&mut self) {
        if self.offset >= self.page_size() {
            self.offset -= self.page_size();
            self.data.update(self.page_start());
        }
    }

    fn jump(&mut self) {
        if let Some(addr) = JumpAddress::new(&self.input).parse(self.offset) {
            self.offset = addr;
            self.data.update(self.page_start());
            self.op_mode = OpMode::Normal;
        }
        self.input.clear();
    }

    fn write(&mut self) {
        if let Some(bytes) = WriteValue::new(&self.input).parse() {
            self.data.write(self.offset, bytes);
            self.data.update(self.page_start());
            self.op_mode = OpMode::Normal;
        }
        self.input.clear();
    }
}

pub fn start<B: Backend, T: MatrixData>(
    terminal: &mut Terminal<B>,
    m: &mut Matrix<T>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, m))?;

        if let Event::Key(key) = event::read()? {
            match m.op_mode {
                OpMode::Normal => match key.code {
                    // Quit application
                    KeyCode::Char('q') => return Ok(()),
                    // Hex matrix navigations
                    KeyCode::Char('h') | KeyCode::Left => m.prev_byte(),
                    KeyCode::Char('l') | KeyCode::Right => m.next_byte(),
                    KeyCode::Char('k') | KeyCode::Up => m.prev_line(),
                    KeyCode::Char('j') | KeyCode::Down => m.next_line(),
                    KeyCode::Char('p') | KeyCode::PageUp => m.prev_page(),
                    KeyCode::Char('n') | KeyCode::PageDown => m.next_page(),
                    // Interactions
                    KeyCode::Char('J') => {
                        m.input.clear();
                        m.op_mode = OpMode::Jump;
                    }
                    KeyCode::Char('e') => {
                        m.input.clear();
                        m.op_mode = OpMode::Write;
                    }
                    _ => {}
                },
                OpMode::Jump => match key.code {
                    KeyCode::Char(c) => m.input.push(c),
                    KeyCode::Backspace => {
                        m.input.pop();
                    }
                    KeyCode::Enter => m.jump(),
                    KeyCode::Esc => m.op_mode = OpMode::Normal,
                    _ => (),
                },
                OpMode::Write => match key.code {
                    KeyCode::Char(c) => m.input.push(c),
                    KeyCode::Backspace => {
                        m.input.pop();
                    }
                    KeyCode::Enter => m.write(),
                    KeyCode::Esc => m.op_mode = OpMode::Normal,

                    _ => (),
                },
            }
        }
    }
}

fn header<B: Backend, T: MatrixData>(f: &mut Frame<B>, app: &Matrix<T>, area: Rect) {
    let header = Paragraph::new(app.name.to_owned()).alignment(Alignment::Center);
    f.render_widget(header, area);
}

fn hex_matrix<B: Backend, T: MatrixData>(f: &mut Frame<B>, m: &Matrix<T>, area: Rect) {
    // Here, cell length is 2, height is 1
    let row_cons = std::iter::repeat(Constraint::Length(1))
        .take(m.rows as usize)
        .collect::<Vec<_>>();
    let col_cons = std::iter::repeat(Constraint::Length(3))
        .take(m.cols as usize)
        .collect::<Vec<_>>();

    // draw hex matrix
    let row_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_cons)
        .split(area);
    for (r, row_rect) in row_rects.into_iter().enumerate() {
        let col_rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_cons.to_owned())
            .split(row_rect);
        for (c, col_rect) in col_rects.into_iter().enumerate() {
            let index = r * (m.cols as usize) + c;
            let cell_str = match m.data.get(index) {
                Some(cell) => cell.hex(),
                None => String::from("xx"),
            };
            if m.page_offset() as usize == index {
                let cb = Paragraph::new(cell_str)
                    .block(Block::default())
                    .style(Style::default().fg(Color::Red))
                    .alignment(Alignment::Left);
                f.render_widget(cb, col_rect);
            } else {
                let cb = Paragraph::new(cell_str)
                    .block(Block::default())
                    .alignment(Alignment::Left);
                f.render_widget(cb, col_rect);
            }
        }
    }
}

fn status<B: Backend, T: MatrixData>(f: &mut Frame<B>, m: &Matrix<T>, area: Rect) {
    let mut content = String::new();
    content.push_str(format!("Page Offset:   0x{:x}\n", m.page_offset()).as_str());
    content.push_str(format!("Global Offset: 0x{:x}\n", m.offset).as_str());

    let block = Paragraph::new(content)
        .block(Block::default())
        .alignment(Alignment::Left);
    f.render_widget(block, area);
}

fn draw_jump<B: Backend, T: MatrixData>(f: &mut Frame<B>, m: &Matrix<T>, area: Rect) {
    let input = Paragraph::new(m.input.as_ref())
        .style(match m.op_mode {
            OpMode::Jump => Style::default().fg(Color::Green),
            _ => Style::default(),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center),
        );
    let area = centered_rect(40, 3, area);
    f.render_widget(Clear, area);
    f.render_widget(input, area);
    match m.op_mode {
        OpMode::Jump => f.set_cursor(area.x + 1 + m.input.width() as u16, area.y + 1),
        _ => {}
    }
}

fn draw_edit<B: Backend, T: MatrixData>(f: &mut Frame<B>, m: &Matrix<T>, area: Rect) {
    let edit = Paragraph::new(m.input.as_ref())
        .style(match m.op_mode {
            OpMode::Write => Style::default().fg(Color::Green),
            _ => Style::default(),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center),
        );
    let area = centered_rect(40, 3, area);

    f.render_widget(Clear, area);
    f.render_widget(edit, area);

    match m.op_mode {
        OpMode::Write => f.set_cursor(area.x + 1 + m.input.width() as u16, area.y + 1),
        _ => {}
    }
}

fn ui<B: Backend, T: MatrixData>(f: &mut Frame<B>, m: &Matrix<T>) {
    let size = f.size();
    let matrix_width = 3 * m.cols + 3;
    let matrix_height = m.rows;
    let padding_left = (size.width - matrix_width - 2) / 2;
    let padding_top = 1;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(padding_left),
            Constraint::Length(matrix_width),
            Constraint::Min(0),
        ])
        .split(size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(padding_top),
            Constraint::Length(2),
            Constraint::Length(matrix_height),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(chunks[1]);
    let header_chunk = chunks[1];
    let hex_chunk = chunks[2];
    let status_chunk = chunks[4];

    header(f, m, header_chunk);
    hex_matrix(f, m, hex_chunk);
    status(f, m, status_chunk);

    match m.op_mode {
        OpMode::Jump => {
            draw_jump(f, m, hex_chunk);
        }
        OpMode::Write => {
            draw_edit(f, m, hex_chunk);
        }
        _ => {}
    }
}
