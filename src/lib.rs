use std::{fs, io::Result, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::log::Item;

pub mod log;

pub static APP_NAME: &str = "lw";

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    logs: Vec<Item>,
    config: PathBuf,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, item: Item) -> Result<()> {
        self.logs.push(item);
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
        }
    }
}
