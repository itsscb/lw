use std::sync::OnceLock;
use std::{fs, path::PathBuf};

use color_eyre::{Result, eyre::eyre};

use ratatui::crossterm::event::{KeyEvent, KeyModifiers};
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
static COLOR_PRIMARY: Color = Color::Rgb(51, 217, 178);
static COLOR_PRIMARY_DARK: Color = Color::Rgb(33, 140, 116);
static COLOR_SECONDARY: Color = Color::Rgb(52, 172, 224);
static COLOR_TERTIARY: Color = Color::Rgb(247, 241, 227);
static COLOR_TERTIARY_DARK: Color = Color::Rgb(132, 129, 122);

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    logs: Vec<Item>,
    #[serde(skip)]
    exit: bool,
    #[serde(skip)]
    edit: Option<Item>,
    #[serde(skip)]
    delete: Option<usize>,
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
impl App {
    pub fn new(config: PathBuf) -> Result<Self> {
        if config.exists()
            && let Ok(v) = fs::read_to_string(config)
        {
            let mut app: Self = serde_json::from_str(&v)?;

            app.logs.sort_by_key(|l| std::cmp::Reverse(l.created()));
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
        self.render(frame.area(), frame.buffer_mut());
        if let Some(ref item) = self.edit {
            let block = Block::bordered()
                .title(Span::styled(
                    "Details",
                    Style::default().bold().fg(COLOR_SECONDARY),
                ))
                .title_bottom(Line::from(vec![
                    Span::raw(" Save "),
                    Span::styled(
                        "<CTRL-Enter> | <CTRL-o>",
                        Style::default()
                            .fg(COLOR_PRIMARY)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Cancel "),
                    Span::styled(
                        "<CTRL-c> | <ESC>",
                        Style::default()
                            .fg(COLOR_PRIMARY)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))
                .title_style(Style::default().bold().fg(Color::White));

            let area = popup_area(frame.area(), 90, 90);

            frame.render_widget(Clear, area);

            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            let inner = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(outer[1]);

            frame.render_widget(
                Paragraph::new(
                    Text::from(format!(
                        "modified at {}",
                        item.modified().format("%Y-%m-%d %H:%M:%S")
                    ))
                    .style(Style::default().fg(COLOR_SECONDARY).bold())
                    .right_aligned(),
                )
                .right_aligned(),
                inner[1],
            );

            frame.render_widget(
                Paragraph::new(
                    Text::from(format!(
                        "created at {}",
                        item.created().format("%Y-%m-%d %H:%M:%S")
                    ))
                    .style(Style::default().fg(COLOR_SECONDARY).bold()),
                ),
                inner[0],
            );

            let content: String = item.content().to_owned();
            let v: Vec<Line> = content
                .split("\n")
                .enumerate()
                .map(|(i, c)| {
                    let mut t = vec![Span::from(c)];
                    if i >= content.split("\n").count() - 1 {
                        t.push(
                            Span::from("_")
                                .patch_style(Style::new().add_modifier(Modifier::RAPID_BLINK)),
                        );
                    }
                    Line::from(t)
                })
                .collect();

            frame.render_widget(
                Paragraph::new(v)
                    .block(block)
                    .wrap(ratatui::widgets::Wrap { trim: false }),
                outer[0],
            );
        }
    }

    pub fn handle_edit_keys(&mut self, key_event: KeyEvent, item: Item) -> Result<()> {
        match key_event.code {
            KeyCode::Backspace => {
                if let Some(item) = handle_backspace(item.clone(), key_event) {
                    self.edit = Some(item);
                }
            }
            KeyCode::Esc => {
                self.edit = None;
            }
            KeyCode::Char('o') | KeyCode::Enter
                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                if !item
                    .content()
                    .replace("\n", "")
                    .replace("\t", "")
                    .is_empty()
                {
                    if self.logs.iter().any(|l| l.id() == item.id()) {
                        self.update(item.id(), item.content());
                    } else {
                        self.logs.push(item.clone());
                        self.logs.sort_by_key(|l| std::cmp::Reverse(l.created()));
                    }
                    self.edit = None;
                    self.save()?;
                }
            }
            KeyCode::Enter => {
                let mut tmp = item.clone();
                let mut s = tmp.content();
                s.push('\n');
                tmp.update(s);
                self.edit = Some(tmp);
            }
            KeyCode::Char(key) => {
                if key == 'h' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    if let Some(item) = handle_backspace(item.clone(), key_event) {
                        self.edit = Some(item);
                    }
                } else if key == 'c' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.edit = None;
                } else {
                    let mut tmp = item.clone();
                    let mut s = tmp.content();
                    s.push(key);
                    tmp.update(s);
                    self.edit = Some(tmp);
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn handle_main_keys(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            KeyCode::Char('e') | KeyCode::Char(' ') | KeyCode::Enter => match self.state.selected()
            {
                Some(selected) => {
                    if let Some(v) = self.logs.get(selected) {
                        self.edit = Some(v.clone());
                    } else {
                        self.edit = Some(Item::new());
                    }
                }
                None => self.edit = Some(Item::new()),
            },
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.select_next();
                self.delete = None;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.select_previous();
                self.delete = None;
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.state.select_first();
                self.delete = None;
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.state.select_last();
                self.delete = None;
            }
            KeyCode::Char('o') => {
                self.edit = Some(Item::new());
                self.delete = None;
            }
            KeyCode::Char('d') => {
                let curr = self.state.selected();
                match curr {
                    None => {
                        self.delete = None;
                    }
                    Some(c) => {
                        if self.delete == curr {
                            let id = self.logs[c].id();
                            self.delete = None;
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
        Ok(())
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.state.select_next();
        loop {
            if self.exit {
                break Ok(());
            }
            terminal.draw(|frame| self.draw(frame))?;
            if let Ok(event) = event::read()
                && let Event::Key(key_event) = event
                && key_event.kind == event::KeyEventKind::Press
            {
                if let Some(ref item) = self.edit {
                    let item = item.clone();
                    self.handle_edit_keys(key_event, item)?;
                    continue;
                }
                self.handle_main_keys(key_event)?;
            }
        }
    }

    pub fn add(&mut self, item: Item) {
        self.logs.push(item);
        self.logs.sort_by_key(|l| std::cmp::Reverse(l.created()));
    }

    pub fn update<T: AsRef<str>>(&mut self, id: T, content: T) {
        if let Some(item) = self.logs.iter_mut().find(|i| i.id() == id.as_ref()) {
            item.update(content.as_ref().to_owned());
        }
        self.logs.sort_by_key(|l| std::cmp::Reverse(l.created()));
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
        let primary_color = if self.edit.is_none() {
            COLOR_PRIMARY
        } else {
            COLOR_PRIMARY_DARK
        };

        let teritary_color = if self.edit.is_none() {
            COLOR_TERTIARY
        } else {
            COLOR_TERTIARY_DARK
        };

        let title = Line::from(Span::styled(
            " Log Your Work ",
            Style::default().fg(primary_color).bold(),
        ));

        let instructions = Line::from(vec![
            Span::raw(" New "),
            Span::styled(
                "<o>",
                Style::default()
                    .fg(primary_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Select "),
            Span::styled(
                "<e> | <Enter> | <Space>",
                Style::default()
                    .fg(primary_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Down "),
            Span::styled(
                "<j>",
                Style::default()
                    .fg(primary_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Up "),
            Span::styled(
                "<k>",
                Style::default()
                    .fg(primary_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Bottom "),
            Span::styled(
                "<G>",
                Style::default()
                    .fg(primary_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Top "),
            Span::styled(
                "<g>",
                Style::default()
                    .fg(primary_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Quit "),
            Span::styled(
                "<q> | <ESC>",
                Style::default()
                    .fg(primary_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .title_style(Color::White)
            .border_set(border::THICK)
            .border_style(Color::White);

        let header = [
            "Log", // "Modified",
            "Created",
        ]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(Style::default().fg(teritary_color).bold())
        .height(1);

        let mut highlight_style = Style::new().fg(primary_color).bold();

        let row_text_color = if self.edit.is_some() {
            COLOR_TERTIARY_DARK
        } else {
            Color::White
        };

        let items: Vec<Row> = self
            .logs
            .iter()
            .enumerate()
            .map(|(i, item)| {
                [
                    item.content().replace("\n", " "),
                    // item.modified().format("%Y-%m-%d %H:%M:%S").to_string(),
                    item.created().format("%Y-%m-%d %H:%M:%S").to_string(),
                ]
                .into_iter()
                .map(|c| {
                    Cell::from(Text::from(c).style({
                        let s = Style::new();

                        if let Some(index) = self.delete
                            && i == index
                        {
                            highlight_style = Style::new().fg(Color::LightRed).bold();
                            s.fg(Color::LightRed).bold()
                        } else {
                            s
                        }
                    }))
                })
                .collect::<Row>()
                .style(Style::new().fg(row_text_color))
                .height(2)
            })
            .collect();

        let empty = items.is_empty();

        let mut table = Table::new(items, [Constraint::Min(200), Constraint::Min(20)])
            .block(block)
            .header(header)
            .highlight_symbol("> ")
            .row_highlight_style(highlight_style)
            .highlight_spacing(HighlightSpacing::Always);

        if empty {
            table = table.footer(
                Row::new(vec!["Nothing here yet"]).style(Style::default().fg(COLOR_SECONDARY)),
            );
        }
        StatefulWidget::render(table, area, buf, &mut self.state);
    }
}

fn ctrl_backspace_remaining<T: AsRef<str>>(s: T) -> String {
    let s = String::from(s.as_ref());
    let cut_pos = {
        let mut chars = s.char_indices().rev();

        let mut pos_after_trailing_ws = None;
        for (idx, ch) in &mut chars {
            if !ch.is_whitespace() {
                pos_after_trailing_ws = Some(idx + ch.len_utf8());
                break;
            }
        }

        if pos_after_trailing_ws.is_some() {
            let mut o = 0;
            for (idx, ch) in chars {
                if ch.is_whitespace() {
                    o = idx + ch.len_utf8();
                    o = o.saturating_sub(1);
                    break;
                }
            }
            o
        } else {
            0
        }
    };

    String::from(&s[..cut_pos])
}

fn handle_backspace(item: Item, key_event: KeyEvent) -> Option<Item> {
    let mut tmp = item;
    let mut s: String = tmp.content();
    if !s.is_empty() {
        tmp.update(if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            ctrl_backspace_remaining(s)
        } else {
            s.truncate(s.len() - 1);
            s
        });
        return Some(tmp);
    }
    None
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
