use std::{
    collections::HashMap,
    io::{self, Stdout},
    thread,
    time::Duration,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame, Terminal,
};

const DB_PATH: &str = "db.json";

struct Todo {
    tasks: HashMap<String, bool>,
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

        Ok(Todo { tasks })
    }
}

fn main() -> Result<(), io::Error> {
    let mut todo = match Todo::load() {
        Ok(todo) => todo,
        Err(_) => Todo {
            tasks: HashMap::new(),
        },
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run(&mut terminal, &mut todo)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    todo: &mut Todo,
) -> Result<(), io::Error> {
    terminal.draw(|f| ui(f, &todo))?;

    thread::sleep(Duration::from_millis(2000));

    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, todo: &Todo) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());

    let block = Block::default().title("New task").borders(Borders::ALL);
    f.render_widget(block, chunks[0]);

    let block = Block::default().title("Tasks").borders(Borders::ALL);
    let tasks = todo.list();
    let tasks = List::new(tasks).block(block).highlight_style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    f.render_widget(tasks, chunks[1]);
}
