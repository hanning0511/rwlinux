use super::{Cell, OpMode};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use hex;
use libc::{O_RDWR, O_SYNC};
use memmap::MmapOptions;
use std::fs::OpenOptions;
use std::{error::Error, os::unix::prelude::OpenOptionsExt};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

const MEMDEV: &str = "/dev/mem";

pub fn read_byte(offset: u64) -> Option<u8> {
    let file = OpenOptions::new().read(true).write(true).open(MEMDEV);
    if file.is_err() {
        return None;
    }
    let mmap = unsafe { MmapOptions::new().offset(offset).len(1).map(&file.unwrap()) };
    if mmap.is_err() {
        return None;
    }
    let mmap = mmap.unwrap();
    Some(mmap[0])
}

pub fn write(offset: u64, bytes: Vec<u8>) {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(O_RDWR | O_SYNC)
        .open(MEMDEV);
    if file.is_err() {
        println!("fail to open /dev/mem");
        return;
    }

    let mmap = unsafe {
        MmapOptions::new()
            .offset(offset)
            .len(bytes.len())
            .map_mut(&file.unwrap())
    };
    if mmap.is_err() {
        println!("fail to map /dev/mem");
        return;
    }
    let mut mmap = mmap.unwrap();

    mmap.copy_from_slice(&bytes)
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

#[derive(Debug)]
struct Devmem {
    columns: u64,
    rows: u64,
    offset: u64,
    data: Vec<Cell>,
    mode: OpMode,
    jump_addr: String,
    write_value: String,
}

impl Devmem {
    fn new() -> Self {
        let mut app = Self {
            columns: 16,
            rows: 16,
            offset: 0,
            data: vec![],
            mode: OpMode::Normal,
            jump_addr: String::new(),
            write_value: String::new(),
        };
        app.get_data();
        app
    }

    fn get_data(&mut self) {
        let mut data: Vec<Cell> = vec![];
        let page_start = self.offset - (self.offset % self.page_size());
        let page_stop = page_start + self.page_size();

        for i in page_start..page_stop {
            data.push(Cell {
                inner: read_byte(i as u64),
            });
        }

        self.data = data;
    }

    fn page_size(&self) -> u64 {
        (self.rows * self.columns) as u64
    }

    fn page_offset(&self) -> u64 {
        self.offset % self.page_size()
    }

    fn next_page(&mut self) {
        self.offset += self.page_size();
        self.get_data();
    }

    fn prev_page(&mut self) {
        if self.offset >= self.page_size() {
            self.offset -= self.page_size();
            self.get_data();
        }
    }

    fn next_byte(&mut self) {
        self.offset += 1;
        if self.page_offset() == 0 {
            self.get_data();
        }
    }

    fn prev_byte(&mut self) {
        if self.offset == 0 {
            return;
        }

        self.offset -= 1;
        if self.page_offset() == self.page_size() - 1 {
            self.get_data();
        }
    }

    fn next_line(&mut self) {
        self.offset += self.columns;
        if self.page_offset() < self.columns {
            self.get_data();
        }
    }

    fn prev_line(&mut self) {
        if self.offset < self.columns {
            return;
        }
        self.offset -= self.columns;
        if self.page_offset() >= self.page_size() - self.columns {
            self.get_data();
        }
    }

    fn jump_to(&mut self) {
        if let Some(addr) = JumpAddress::new(&self.jump_addr).parse(self.offset) {
            self.offset = addr;
            self.get_data();
            self.mode = OpMode::Normal;
        }
        self.jump_addr.clear();
    }

    fn edit(&mut self) {
        // TODO: implement edit logic here
        if let Some(bytes) = WriteValue::new(&self.write_value).parse() {
            write(self.offset, bytes);
            self.get_data();
            self.mode = OpMode::Normal;
        }

        // reset edit value
        self.write_value.clear();
    }

    fn handle_events(&mut self, key: KeyEvent) -> std::io::Result<()> {
        match self.mode {
            OpMode::Normal => match key.code {
                KeyCode::PageDown | KeyCode::Char('n') => self.next_page(),
                KeyCode::PageUp | KeyCode::Char('p') => self.prev_page(),
                KeyCode::Right | KeyCode::Char('l') => self.next_byte(),
                KeyCode::Left | KeyCode::Char('h') => self.prev_byte(),
                KeyCode::Up | KeyCode::Char('k') => self.prev_line(),
                KeyCode::Down | KeyCode::Char('j') => self.next_line(),
                KeyCode::Char('J') => self.mode = OpMode::Jump,
                KeyCode::Char('e') => self.mode = OpMode::Edit,
                _ => (),
            },
            OpMode::Jump => match key.code {
                KeyCode::Char(c) => self.jump_addr.push(c),
                KeyCode::Backspace => {
                    self.jump_addr.pop();
                }
                KeyCode::Enter => self.jump_to(),
                KeyCode::Esc => self.mode = OpMode::Normal,
                _ => (),
            },
            OpMode::Edit => match key.code {
                KeyCode::Char(c) => self.write_value.push(c),
                KeyCode::Backspace => {
                    self.write_value.pop();
                }
                KeyCode::Enter => self.edit(),
                KeyCode::Esc => self.mode = OpMode::Normal,
                _ => (),
            },
        }
        Ok(())
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut terminal = super::init_terminal()?;
    let dm = Devmem::new();
    let result = run_app(&mut terminal, dm);
    if let Err(err) = result {
        println!("{:?}", err);
    }
    super::reset_terminal()?;
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut dm: Devmem) -> std::io::Result<()> {
    loop {
        terminal.draw(|f| draw(f, &mut dm))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                _ => {
                    dm.handle_events(key)?;
                }
            }
        }
    }
}

const CELL_WIDTH: u16 = 3;
const CELL_HEIGHT: u16 = 1;

fn draw_header<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let header = Paragraph::new("/dev/mem")
        .block(Block::default())
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(header, area);
}

fn draw_hex_matrix<B: Backend>(f: &mut Frame<B>, area: Rect, app: &Devmem) {
    let row_constraints = std::iter::repeat(Constraint::Length(CELL_HEIGHT))
        .take(app.rows as usize)
        .collect::<Vec<_>>();
    let col_constraints = std::iter::repeat(Constraint::Length(CELL_WIDTH))
        .take(app.columns as usize)
        .collect::<Vec<_>>();

    // draw config matrix
    let row_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    for (r, row_rect) in row_rects.into_iter().enumerate() {
        let col_rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints.clone())
            .split(row_rect);

        for (c, cell_rect) in col_rects.into_iter().enumerate() {
            let index = r * (app.columns as usize) + c;
            let cell_str = match app.data.get(index) {
                Some(cell) => cell.hex_str(),
                None => String::from("xx"),
            };

            if app.page_offset() as usize == index {
                let cb = Paragraph::new(cell_str)
                    .block(Block::default())
                    .style(Style::default().fg(Color::Red))
                    .alignment(Alignment::Left);
                f.render_widget(cb, cell_rect);
            } else {
                let cb = Paragraph::new(cell_str)
                    .block(Block::default())
                    .alignment(Alignment::Left);
                f.render_widget(cb, cell_rect);
            }
        }
    }
}

fn draw_status_bar<B: Backend>(f: &mut Frame<B>, area: Rect, app: &Devmem) {
    let mut content = String::new();
    content.push_str(format!("Page Offset:   0x{:x}\n", app.page_offset()).as_str());
    content.push_str(format!("Global Offset: 0x{:x}\n", app.offset).as_str());
    let status = Paragraph::new(content)
        .block(Block::default())
        .alignment(Alignment::Left);
    f.render_widget(status, area);
}

fn draw_jump<B: Backend>(f: &mut Frame<B>, app: &Devmem) {
    let input = Paragraph::new(app.jump_addr.as_ref())
        .style(match app.mode {
            OpMode::Jump => Style::default().fg(Color::Green),
            _ => Style::default(),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center),
        );
    let area = super::centered_rect(40, 3, f.size());
    f.render_widget(Clear, area);
    f.render_widget(input, area);
    match app.mode {
        OpMode::Jump => f.set_cursor(area.x + 1 + app.jump_addr.width() as u16, area.y + 1),
        _ => {}
    }
}

fn draw_edit<B: Backend>(f: &mut Frame<B>, app: &Devmem) {
    let edit = Paragraph::new(app.write_value.as_ref())
        .style(match app.mode {
            OpMode::Edit => Style::default().fg(Color::Green),
            _ => Style::default(),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center),
        );
    let area = super::centered_rect(40, 3, f.size());

    f.render_widget(Clear, area);
    f.render_widget(edit, area);

    match app.mode {
        OpMode::Edit => f.set_cursor(area.x + 1 + app.write_value.width() as u16, area.y + 1),
        _ => {}
    }
}

fn draw<B: Backend>(f: &mut Frame<B>, app: &mut Devmem) {
    let area = f.size();
    let hex_matrix_width = app.columns * CELL_WIDTH as u64 + 1;
    let hex_matrix_height = app.rows * CELL_HEIGHT as u64 + 1;
    let margin_left = (area.width - (hex_matrix_width as u16 + 1)) / 2;
    let margin_top = (area.height - (hex_matrix_height as u16 + 4)) / 2;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(margin_left),             // left margin
            Constraint::Length(hex_matrix_width as u16), // main chunk
            Constraint::Min(0),                          // right margin
        ])
        .split(area);
    let chunks = Layout::default()
        .margin(1)
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(margin_top),
            Constraint::Length(2),
            Constraint::Length(hex_matrix_height as u16),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(chunks[1]);
    let header_chunk = chunks[1];
    let hex_chunk = chunks[2];
    let status_chunk = chunks[3];

    f.render_widget(Clear, f.size());

    draw_header(f, header_chunk);
    draw_hex_matrix(f, hex_chunk, &app);
    draw_status_bar(f, status_chunk, &app);

    match app.mode {
        OpMode::Jump => draw_jump(f, app),
        OpMode::Edit => draw_edit(f, app),
        _ => {}
    }
}
