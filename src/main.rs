use std::{
    collections::HashMap,
    error::Error,
    io::{self, Stdout},
    thread,
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use crossterm::{execute, terminal::EnterAlternateScreen};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

const DB_PATH: &str = "db.json";

#[derive(PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
}

struct Todo {
    tasks: HashMap<String, bool>,
    new_task_text: String,
    mode: Mode,
}
impl Todo {
    fn insert(&mut self, task: String) {
        self.tasks.insert(task.clone(), false);
    }

    fn toggle(&mut self, task: &String) {
        let found_task = self.tasks.get_mut(task);

        match found_task {
            Some(value) => {
                *value = !*value;
            }
            None => {}
        }
    }

    fn list(&self) -> Vec<ListItem> {
        let mut items = Vec::new();

        for (task, status) in &self.tasks {
            let formated_status = if *status { "[x]" } else { "[ ]" };

            items.push(ListItem::new(format!("{} - {}", formated_status, task)));
        }

        items
    }

    fn delete(&mut self, task: &String) {
        self.tasks.remove(task);
    }

    fn save(&self) -> Result<(), std::io::Error> {
        let data = serde_json::to_string(&self.tasks)?;

        std::fs::write(DB_PATH, data)?;

        Ok(())
    }

    fn load() -> Result<Todo, std::io::Error> {
        let data = std::fs::read_to_string(DB_PATH)?;

        let tasks: HashMap<String, bool> = serde_json::from_str(&data)?;

        Ok(Todo {
            tasks,
            new_task_text: String::new(),
            mode: Mode::Normal,
        })
    }
}

fn main() -> Result<(), io::Error> {
    let mut todo = match Todo::load() {
        Ok(todo) => todo,
        Err(_) => Todo {
            tasks: HashMap::new(),
            new_task_text: String::new(),
            mode: Mode::Normal,
        },
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

        let new_task_text = if todo.mode == Mode::Normal {
            "Press i to add a task"
        } else {
            &todo.new_task_text
        };

        let new_task = Paragraph::new(Text::raw(new_task_text))
            .block(Block::default().title("Add tasks").borders(Borders::ALL));

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
                KeyCode::Char('i') => {
                    todo.new_task_text = String::new();
                    todo.mode = Mode::Insert;
                }
                KeyCode::Char('q') => {
                    return Err("Quitting".into());
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
