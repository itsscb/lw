use std::sync::OnceLock;
use std::{fs, path::PathBuf};

use color_eyre::{Result, eyre::eyre};

use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::Flex;
use ratatui::prelude::*;

use ratatui::widgets::{Cell, Clear, HighlightSpacing, Paragraph, Row, Table, TableState};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    style::Color,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};
use serde::{Deserialize, Serialize};

use crate::log::Item;

pub mod log;

pub static APP_NAME: &str = "lw";
static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    logs: Vec<Item>,
    #[serde(skip)]
    delete: Option<usize>,
    #[serde(skip)]
    input: String,
    #[serde(skip)]
    state: TableState,
}
impl Default for App {
    #[allow(clippy::expect_used)]
    fn default() -> Self {
        let config = Self::config_path();
        Self::new(config.to_owned()).expect("could not create app struct with default config")
    }
}
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
impl App {
    pub fn new(config: PathBuf) -> Result<Self> {
        if config.exists()
            && let Ok(v) = fs::read_to_string(config)
            && let Ok(app) = serde_json::from_str(&v)
        {
            return Ok(app);
        }
        Err(eyre!("failed to read config"))
    }
    #[allow(clippy::expect_used)]
    pub fn config_path() -> &'static PathBuf {
        CONFIG_PATH.get_or_init(|| {
            if cfg!(windows) {
                let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
                let dir = PathBuf::from(base).join(APP_NAME);

                if !dir.exists() {
                    fs::create_dir_all(&dir).expect("failed to create app directory");
                }
                dir.join("config.json")
            } else {
                let base = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                let dir = PathBuf::from(base).join(".config").join(APP_NAME);
                if !dir.exists() {
                    fs::create_dir_all(&dir).expect("failed to create app directory");
                }
                dir.join("config.json")
            }
        })
    }

    fn draw(&mut self, frame: &mut Frame) {
        if let Some(item) = self.logs.last()
            && item.content().is_empty()
        {
            let block = Block::bordered().title("New");
            let area = popup_area(frame.area(), 60, 20);
            frame.render_widget(Clear, area);
            frame.render_widget(
                Paragraph::new(Text::from(self.input.clone())).block(block),
                area,
            );
        }
        self.render(frame.area(), frame.buffer_mut());
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if let Ok(event) = event::read()
                && let Event::Key(key_event) = event
                && key_event.kind == event::KeyEventKind::Press
            {
                if let Some(item) = self.logs.last()
                    && item.content().is_empty()
                {
                    match key_event.code {
                        KeyCode::Char('o')
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            self.update(item.id(), self.input.clone());
                            self.input = String::new();
                            self.save()?;
                            continue;
                        }
                        KeyCode::Char(key) => {
                            self.input.push(key);
                            continue;
                        }
                        KeyCode::Backspace => {
                            self.input.truncate(self.input.len() - 1);
                            continue;
                        }
                        KeyCode::Esc => {
                            self.remove(item.id());
                            self.input = String::new();
                            self.save()?;
                            continue;
                        }
                        _ => {}
                    }
                }
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => self.state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.state.select_previous(),
                    KeyCode::Char('g') | KeyCode::Home => self.state.select_first(),
                    KeyCode::Char('G') | KeyCode::End => self.state.select_last(),
                    KeyCode::Char('o') => self.add(Item::new()),
                    KeyCode::Char('d') => {
                        let curr = self.state.selected();
                        match curr {
                            None => {
                                self.delete = None;
                                continue;
                            }
                            Some(c) => {
                                if self.delete == curr {
                                    let id = self.logs[c].id();
                                    self.remove(id);
                                    self.save()?;
                                } else {
                                    self.delete = curr;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn add(&mut self, item: Item) {
        self.logs.push(item);
    }

    pub fn update<T: AsRef<str>>(&mut self, id: T, content: T) {
        if let Some(item) = self.logs.iter_mut().find(|i| i.id() == id.as_ref()) {
            item.update(content.as_ref().to_owned());
        }
    }

    pub fn remove<T: AsRef<str>>(&mut self, id: T) {
        self.logs.retain(|i| i.id() != id.as_ref());
    }

    pub fn save(&self) -> Result<()> {
        let output = serde_json::to_string_pretty(&self)?;
        fs::write(Self::config_path(), output)?;
        Ok(())
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" Log Your Work ".bold());

        let instructions = Line::from(vec![
            Span::raw(" New "),
            Span::styled(
                "<O>",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Select "),
            Span::styled(
                "<Space>",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Down "),
            Span::styled(
                "<J>",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Up "),
            Span::styled(
                "<K>",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Quit "),
            Span::styled(
                "<Q> | <ESC>",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .title_style(Style::new().blue())
            .border_set(border::THICK)
            .border_style(Style::new().dark_gray());

        let header = ["Log", "Modified", "Created"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::default().fg(Color::DarkGray).bold())
            .height(1);

        let items: Vec<Row> = self
            .logs
            .iter()
            .map(|item| {
                [
                    item.content(),
                    item.modified().format("%Y-%m-%d %H:%M:%S").to_string(),
                    item.created().format("%Y-%m-%d %H:%M:%S").to_string(),
                ]
                .into_iter()
                .map(|c| Cell::from(Text::from(c)))
                .collect::<Row>()
                .style(Style::new().fg(Color::White))
                .height(4)
            })
            .collect();

        let table = Table::new(
            items,
            [
                Constraint::Min(200),
                Constraint::Min(10),
                Constraint::Min(10),
            ],
        )
        .block(block)
        .header(header)
        .highlight_symbol(">")
        .row_highlight_style(Style::new().bold().fg(Color::Green))
        .highlight_spacing(HighlightSpacing::Always);

        // let list = List::new(items)
        //     .block(block)
        //     .highlight_symbol(">")
        //     .highlight_spacing(HighlightSpacing::Always);
        //
        StatefulWidget::render(table, area, buf, &mut self.state);
    }
}
