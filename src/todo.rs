use std::{
    io::{prelude::*, BufReader, Error, Write},
    net::TcpStream,
    path::PathBuf,
    str,
};

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

        for (_, task) in self.tasks.iter().enumerate() {
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

        send_tasks_to_server(&self).expect("Failed to send todo to server");

        match std::fs::write(path, data) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn load() -> Result<Todo, std::io::Error> {
        let path = get_database_path();

        let data = std::fs::read_to_string(path)?;

        let mut tasks: Vec<Task> = serde_json::from_str(&data)?;

        tasks = read_tasks_from_server().expect("Failed to read tasks from server");

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

fn send_tasks_to_server(todo: &Todo) -> Result<(), Error> {
    let mut input = String::from("write\n");

    input.push_str(
        serde_json::to_string(&todo.tasks)
            .expect("Failed to serialize tasks")
            .as_str(),
    );

    let mut stream = TcpStream::connect("127.0.0.1:7878")?;

    stream.write(input.as_bytes()).expect("Failed to write");

    let mut reader = BufReader::new(&stream);
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_until(b'\n', &mut buffer)?;

    Ok(())
}

fn read_tasks_from_server() -> Result<Vec<Task>, Error> {
    let input = String::from("read\n");

    let mut stream = TcpStream::connect("127.0.0.1:7878")?;

    stream.write(input.as_bytes()).expect("Failed to write");

    let mut reader = BufReader::new(&stream);
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_until(b'\n', &mut buffer)?;

    let response = str::from_utf8(&buffer).unwrap();

    serde_json::from_str(&response).map_err(|e| e.into())
}
