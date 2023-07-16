mod task;
mod todo;

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
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, Paragraph},
    Terminal,
};
use todo::Todo;

#[derive(PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Delete,
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
            .style(match todo.mode {
                Mode::Normal => Style::default(),
                Mode::Delete => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                Mode::Insert => Style::default().add_modifier(Modifier::BOLD),
            })
            .block(Block::default().title("Add a task").borders(Borders::ALL));

        if todo.mode == Mode::Insert {
            f.set_cursor(
                layout[1].x + new_task_text.len() as u16 + 1,
                layout[1].y + 1,
            );
        }

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
