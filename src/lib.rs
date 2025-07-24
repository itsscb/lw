use std::{fs, path::PathBuf};

use color_eyre::Result;

use ratatui::prelude::*;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    style::Color,
    symbols::border,
    text::Line,
    widgets::{Block, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget},
};
use serde::{Deserialize, Serialize};

use crate::log::Item;

pub mod log;

pub static APP_NAME: &str = "lw";

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    logs: Vec<Item>,
    config: PathBuf,
    #[serde(skip)]
    state: ListState,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
    fn draw(&mut self, frame: &mut Frame) {
        self.render(frame.area(), frame.buffer_mut());
    }
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            // terminal.draw(render)?;
            terminal.draw(|frame| self.draw(frame))?;
            if let Ok(event) = event::read()
                && let Event::Key(key_event) = event
                && key_event.kind == event::KeyEventKind::Press
            {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                    _ => {}
                }
            }
            // match event::read() {
            //     Ok(event) => match event {
            //         Event::Key(key_event) => match key_event.kind {
            //             event::KeyEventKind::Press => {
            //                 if key_event.code == KeyCode::Char('q') {
            //                     break Ok(());
            //                 }
            //             }
            //             _ => {}
            //         },
            //         _ => {}
            //     },
            //     _ => {}
            // }
        }
    }

    pub fn add(&mut self, item: Item) -> Result<()> {
        self.logs.push(item);
        self.save()
    }

    pub fn update<T: AsRef<str>>(&mut self, id: T, content: T) -> Result<()> {
        if let Some(item) = self.logs.iter_mut().find(|i| i.id() == id.as_ref()) {
            item.update(content.as_ref().to_owned());
        }
        self.save()
    }

    pub fn remove<T: AsRef<str>>(&mut self, id: T) -> Result<()> {
        self.logs.retain(|i| i.id() != id.as_ref());
        self.save()
    }

    pub fn save(&self) -> Result<()> {
        let output = serde_json::to_string_pretty(&self)?;
        fs::write(self.config.clone(), output)?;
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        #[cfg(unix)]
        let mut dir = PathBuf::from(std::env::var("HOME").expect("No HOME directory"));
        #[cfg(windows)]
        let mut dir = PathBuf::from(std::env::var("APPDATA").expect("No APPDATA directory"));

        dir.push(APP_NAME);

        if !dir.exists() {
            fs::create_dir_all(dir.clone()).expect("Failed to create App directory");
        }

        dir.push("config.json");

        if dir.exists()
            && let Ok(v) = fs::read_to_string(&dir)
            && let Ok(app) = serde_json::from_str(&v)
        {
            return app;
        }

        Self {
            logs: vec![],
            config: dir,
            ..Default::default()
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" Log Your Work ".bold());

        let instructions = Line::from(vec![
            Span::raw(" Add "),
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
            .border_set(border::THICK);

        let items: Vec<ListItem> = self
            .logs
            .iter()
            .enumerate()
            .map(|(i, item)| {
                ListItem::new(Line::styled(format!("{}", item.content()), Color::White)).bg(
                    if i % 2 == 0 {
                        Color::Black
                    } else {
                        Color::DarkGray
                    },
                )
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}
