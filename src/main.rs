use std::{
    error::Error,
    fs,
    io::{self, Stdout},
    path::PathBuf,
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use crossterm::{execute, terminal::EnterAlternateScreen};
use directories::ProjectDirs;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use serde::{Deserialize, Serialize};

const DB_FILE: &str = "db.json";
const DEBUG: bool = false;

#[derive(PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
    Delete,
}

#[derive(Clone, Serialize, Deserialize)]
struct Task {
    text: String,
    completed: bool,
}
impl Task {
    fn new(text: String) -> Self {
        Self {
            text,
            completed: false,
        }
    }
}

struct Todo {
    tasks: Vec<Task>,
    new_task_text: String,
    mode: Mode,
    current_task: usize,
}
impl Todo {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            new_task_text: String::new(),
            mode: Mode::Normal,
            current_task: 0,
        }
    }

    fn insert(&mut self, text: String) {
        self.tasks.insert(self.tasks.len(), Task::new(text));
        self.current_task = self.tasks.len() - 1;

        self.save().unwrap();
    }

    fn toggle(&mut self) {
        let found_task = self.tasks.get_mut(self.current_task);

        match found_task {
            Some(task) => {
                task.completed = !task.completed;
            }
            None => {}
        }

        self.save().unwrap();
    }

    fn list(&self) -> Vec<ListItem> {
        let mut items = Vec::new();

        for (index, task) in self.tasks.iter().enumerate() {
            let formated_status = if task.completed { "[x]" } else { "[ ]" };

            let formatted_text = if DEBUG {
                format!(
                    "{} {} => {} - {}",
                    &self.current_task, index, formated_status, task.text
                )
            } else {
                format!("{} {}", formated_status, task.text)
            };

            let list_item = ListItem::new(formatted_text);

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

    fn delete(&mut self) {
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

    fn save(&self) -> Result<(), std::io::Error> {
        let data = serde_json::to_string(&self.tasks)?;

        let path = ProjectDirs::from("eu", "tortitas", "todot")
            .unwrap()
            .data_dir()
            .to_path_buf();

        ensure_dir_exists(&path).unwrap();

        let path = path.join(DB_FILE);

        match std::fs::write(path, data) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn load() -> Result<Todo, std::io::Error> {
        let path = ProjectDirs::from("eu", "tortitas", "todot")
            .unwrap()
            .data_dir()
            .join(DB_FILE);

        let data = std::fs::read_to_string(path)?;

        let tasks: Vec<Task> = serde_json::from_str(&data)?;

        Ok(Todo {
            tasks,
            new_task_text: String::new(),
            mode: Mode::Normal,
            current_task: 0,
        })
    }

    fn prev(&mut self) {
        if self.current_task > 0 {
            self.current_task -= 1;
        }
    }

    fn next(&mut self) {
        if self.current_task < self.tasks.len() - 1 {
            self.current_task += 1;
        }
    }
}

fn main() -> Result<(), io::Error> {
    let mut todo = match Todo::load() {
        Ok(todo) => todo,
        Err(_) => Todo::new(),
    };

    let mut terminal = setup_terminal().unwrap();

    run(&mut terminal, &mut todo)?;

    restore_terminal(&mut terminal).unwrap();

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    Ok(terminal.show_cursor()?)
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    todo: &mut Todo,
) -> Result<(), io::Error> {
    loop {
        draw(terminal, todo).unwrap();

        match handle_input(todo) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }

    Ok(())
}

fn draw(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    todo: &mut Todo,
) -> Result<(), io::Error> {
    terminal.draw(|f| {
        let size = f.size();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(size);

        let tasks =
            List::new(todo.list()).block(Block::default().title("Tasks").borders(Borders::ALL));

        let new_task_text = match todo.mode {
            Mode::Normal => "Add a task (Press 'i' to insert)",
            Mode::Delete => "Press 'd' again to delete the selected task",
            Mode::Insert => &todo.new_task_text,
        };

        let new_task = Paragraph::new(Text::raw(new_task_text))
            .block(Block::default().title("Add a task").borders(Borders::ALL));

        f.render_widget(tasks, layout[0]);
        f.render_widget(new_task, layout[1]);
    })?;

    Ok(())
}

fn handle_input(todo: &mut Todo) -> Result<(), Box<dyn Error>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            match key.code {
                _ if todo.mode == Mode::Insert => handle_insert_mode(key, todo),
                _ if todo.mode == Mode::Delete => match key.code {
                    KeyCode::Char('d') => {
                        todo.delete();
                        todo.mode = Mode::Normal;
                    }
                    KeyCode::Esc => {
                        todo.mode = Mode::Normal;
                    }
                    _ => {}
                },
                KeyCode::Char('i') | KeyCode::Char('o') | KeyCode::Char('a') => {
                    todo.new_task_text = String::new();
                    todo.mode = Mode::Insert;
                }
                KeyCode::Char('q') => {
                    return Err("Quitting".into());
                }
                KeyCode::Up | KeyCode::Char('k') => todo.prev(),
                KeyCode::Down | KeyCode::Char('j') => todo.next(),
                KeyCode::Char(' ') | KeyCode::Enter => {
                    todo.toggle();
                }
                KeyCode::Char('d') => {
                    if todo.mode == Mode::Normal {
                        todo.mode = Mode::Delete;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn handle_insert_mode(key: KeyEvent, todo: &mut Todo) {
    match key.code {
        KeyCode::Char(c) => {
            todo.new_task_text.push(c);
        }
        KeyCode::Backspace => {
            todo.new_task_text.pop();
        }
        KeyCode::Enter => {
            todo.mode = Mode::Normal;
            todo.insert(todo.new_task_text.clone());
            todo.new_task_text = String::new();
        }
        KeyCode::Esc => {
            todo.mode = Mode::Normal;
            todo.new_task_text = String::new();
        }
        _ => {}
    }
}

fn ensure_dir_exists(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}
