use std::path::PathBuf;

use directories::ProjectDirs;
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::ListItem,
};

use crate::{ensure_dir_exists, task::Task, Mode};

const DB_FILE: &str = "db.json";

pub struct Todo {
    pub tasks: Vec<Task>,
    pub new_task_text: String,
    pub mode: Mode,
    pub current_task: usize,
}
impl Todo {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            new_task_text: String::new(),
            mode: Mode::Normal,
            current_task: 0,
        }
    }

    pub fn insert(&mut self, text: String) {
        self.tasks.insert(self.tasks.len(), Task::new(text));
        self.current_task = self.tasks.len() - 1;

        self.save().unwrap();
    }

    pub fn toggle(&mut self) {
        let found_task = self.tasks.get_mut(self.current_task);

        match found_task {
            Some(task) => {
                task.completed = !task.completed;
            }
            None => {}
        }

        self.save().unwrap();
    }

    pub fn list(&self) -> Vec<ListItem> {
        let mut items = Vec::new();

        for (index, task) in self.tasks.iter().enumerate() {
            let formated_status = if task.completed { "[x]" } else { "[ ]" };

            let list_item = ListItem::new(format!("{} {}", formated_status, task.text));

            let style = match self.current_task == items.len() {
                true => Style::default().add_modifier(Modifier::BOLD),
                false => Style::default(),
            };

            let style = match task.completed {
                true => style.fg(Color::Green),
                false => style.fg(Color::Yellow),
            };

            items.push(list_item.style(style));
        }

        items
    }

    pub fn delete(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        self.tasks.remove(self.current_task);

        self.current_task = if self.current_task > 0 {
            self.current_task - 1
        } else {
            0
        };

        self.save().unwrap();
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let data = serde_json::to_string(&self.tasks)?;

        let path = get_database_path();

        match std::fs::write(path, data) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn load() -> Result<Todo, std::io::Error> {
        let path = get_database_path();

        let data = std::fs::read_to_string(path)?;

        let tasks: Vec<Task> = serde_json::from_str(&data)?;

        Ok(Todo {
            tasks,
            new_task_text: String::new(),
            mode: Mode::Normal,
            current_task: 0,
        })
    }

    pub fn prev(&mut self) {
        if self.current_task > 0 {
            self.current_task -= 1;
        }
    }

    pub fn next(&mut self) {
        if self.current_task < self.tasks.len() - 1 {
            self.current_task += 1;
        }
    }
}

fn get_database_path() -> PathBuf {
    let path = ProjectDirs::from("eu", "tortitas", "todot")
        .unwrap()
        .data_dir()
        .to_path_buf();

    ensure_dir_exists(&path).unwrap();

    path.join(DB_FILE)
}
