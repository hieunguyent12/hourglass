use chrono::{DateTime, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::TableState, Terminal};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::{
    env, io,
    time::{Duration, Instant},
};

mod action;
mod cache;
mod issues;
mod scheduler;
mod ui;

use action::Action;
use issues::{get_issues, GitUser, RepoIssue};

enum View {
    Task(Action),
    Issues(Action),
}

pub const HOURGLASS_EXTENSION: &str = "hourglass";
pub const HOURGLASS_FILE_STORAGE_NAME: &str = "tasks.hourglass";
pub const TIME_FORMAT: &'static str = "%b %d, %Y %I:%M %p";

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Task {
    id: i32,
    description: String,
    completed: bool,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
}

pub struct Hourglass {
    should_quit: bool,
    input: String,
    next_id: i32,

    view: View,

    table_state: TableState,

    tabs: Vec<String>,
    tab_index: usize,

    tasks: Vec<Task>,
    issues: Vec<RepoIssue>,
}

impl Hourglass {
    pub fn new() -> Self {
        let mut table_state = TableState::default();

        table_state.select(Some(0));

        Self {
            should_quit: false,
            input: String::new(),
            view: View::Task(Action::View),
            next_id: 1,
            tasks: vec![],
            issues: vec![],
            table_state,

            tabs: vec![String::from("tasks"), String::from("issues")],
            tab_index: 0,
        }
    }

    pub fn start_tui() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(terminal)
    }

    pub fn pause_tui() -> io::Result<()> {
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        terminal.show_cursor()?;
        Ok(())
    }

    pub fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> io::Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(250);
        // how is rust able to run an infinite loop without crashing?

        loop {
            // terminal.draw(|f| {
            //     ui::build_ui(f, self);
            // })?;

            // wtf is the point of this?
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            // the poll method will halt the loop to wait a certain amount of time (based on timeout) for an event to occur before moving on
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_input(key);
                }
            }

            if self.should_quit {
                return Ok(());
            }

            // why?
            // without this line, the program will consume very high CPU
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
    }

    fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                let len = match self.view {
                    View::Task(_) => self.tasks.len(),
                    View::Issues(_) => self.issues.len(),
                };

                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.table_state.select(Some(i))
    }

    fn previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                let len = match self.view {
                    View::Task(_) => self.tasks.len(),
                    View::Issues(_) => self.issues.len(),
                };

                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.tabs.len();

        self.update_view();
    }

    fn previous_tab(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = self.tabs.len() - 1;
        }

        self.update_view();
    }

    fn update_view(&mut self) {
        match self.tabs[self.tab_index].as_str() {
            "tasks" => self.view = View::Task(Action::View),
            "issues" => {
                self.view = View::Issues(Action::View);

                // TODO: cache the issues
                // self.issues = vec![
                //     RepoIssue {
                //         id: 1,
                //         node_id: "Test".to_string(),
                //         html_url: "Test URL".to_string(),
                //         number: 1234,
                //         title: "Dummy issue".to_string(),
                //         body: "Fix stuff".to_string(),
                //         created_at: "2023-06-04T21:19:30.546904Z"
                //             .parse::<DateTime<Utc>>()
                //             .unwrap(),
                //         updated_at: "2023-06-04T21:19:30.546904Z"
                //             .parse::<DateTime<Utc>>()
                //             .unwrap(),
                //         user: GitUser {
                //             login: "hieu".to_string(),
                //             id: 1,
                //             node_id: "Test".to_string(),
                //         },
                //     },
                //     RepoIssue {
                //         id: 1,
                //         node_id: "Test".to_string(),
                //         html_url: "Test URL".to_string(),
                //         number: 2,
                //         title: "Dummy issue".to_string(),
                //         body: "Fix stuff".to_string(),
                //         created_at: "2023-06-04T21:19:30.546904Z"
                //             .parse::<DateTime<Utc>>()
                //             .unwrap(),
                //         updated_at: "2023-06-04T21:19:30.546904Z"
                //             .parse::<DateTime<Utc>>()
                //             .unwrap(),
                //         user: GitUser {
                //             login: "hieu".to_string(),
                //             id: 1,
                //             node_id: "Test".to_string(),
                //         },
                //     },
                //     RepoIssue {
                //         id: 1,
                //         node_id: "Test".to_string(),
                //         html_url: "Test URL".to_string(),
                //         number: 3,
                //         title: "Dummy issue".to_string(),
                //         body: "Fix stuff".to_string(),
                //         created_at: "2023-06-04T21:19:30.546904Z"
                //             .parse::<DateTime<Utc>>()
                //             .unwrap(),
                //         updated_at: "2023-06-04T21:19:30.546904Z"
                //             .parse::<DateTime<Utc>>()
                //             .unwrap(),
                //         user: GitUser {
                //             login: "hieu".to_string(),
                //             id: 1,
                //             node_id: "Test".to_string(),
                //         },
                //     },
                // ]
                let issues = match get_issues() {
                    Some(issues) => issues,
                    None => vec![],
                };

                self.issues = issues;
            }
            _ => {}
        }

        self.table_state = TableState::default();
    }

    fn toggle_task_status(&mut self) {
        if let Some(i) = self.table_state.selected() {
            if let Some(task) = self.tasks.get_mut(i) {
                task.completed = !task.completed;
            }
        }
    }

    fn add_task(&mut self) {
        let description = self.input.clone();
        let time = Utc::now();

        self.input = String::new();

        self.tasks.push(Task {
            id: self.next_id,
            description,
            completed: false,
            created_at: time,
            modified_at: time,
        });

        self.next_id += 1;

        self.save_tasks();
    }

    fn update_task(&mut self) {
        if let Some(i) = self.table_state.selected() {
            if let Some(task) = self.tasks.get_mut(i) {
                task.description = self.input.clone();

                task.modified_at = Utc::now();

                self.save_tasks();
            }
        }

        self.input = String::new();
    }

    fn remove_task(&mut self) {
        if let Some(index) = self.table_state.selected() {
            self.tasks.remove(index);
            self.save_tasks();
        }
    }

    fn handle_input(&mut self, key_event: KeyEvent) {
        // we handle input differently based on the current view
        match &self.view {
            View::Task(action) => match action {
                Action::View => self.handle_key_for_task_view(key_event.code),
                _ => self.update_command_input(key_event.code),
            },

            View::Issues(action) => match action {
                Action::View => self.handle_key_for_issues_view(key_event.code),
                _ => self.update_command_input(key_event.code),
            },
        }
    }

    fn update_command_input(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char(c) => self.input.push(c),
            KeyCode::Enter => match &self.view {
                View::Task(action) => match action {
                    Action::Add => {
                        self.add_task();

                        self.view = View::Task(Action::View)
                    }
                    Action::Update => {
                        self.update_task();

                        self.view = View::Task(Action::View)
                    }
                    _ => {}
                },

                View::Issues(action) => {}
            },
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Esc => {
                self.input = String::new();
                self.view = View::Task(Action::View);
            }
            _ => {}
        }
    }

    fn handle_key_for_task_view(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char(c) => match c {
                'q' => self.should_quit = true,
                'j' => self.next(),
                'k' => self.previous(),
                'd' => self.toggle_task_status(),
                'a' => self.view = View::Task(Action::Add),
                'u' => self.view = View::Task(Action::Update),
                'x' => self.remove_task(),
                ']' => self.next_tab(),
                '[' => self.previous_tab(),
                _ => {}
            },
            KeyCode::Down => self.next(),
            KeyCode::Up => self.previous(),
            _ => {}
        }
    }

    fn handle_key_for_issues_view(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char(c) => match c {
                'q' => self.should_quit = true,
                'j' => self.next(),
                'k' => self.previous(),
                ']' => self.next_tab(),
                '[' => self.previous_tab(),
                _ => {}
            },
            KeyCode::Down => self.next(),
            KeyCode::Up => self.previous(),
            _ => {}
        }
    }

    pub fn load_tasks(&mut self) -> io::Result<()> {
        // check if a .hourglass file exist
        // if it does, load the content
        // otherwise, create an empty .hourglass file

        let current_dir = env::current_dir()?;

        let paths = fs::read_dir(current_dir).unwrap();
        let mut file_exists = false;

        for path in paths {
            let file_path = path.unwrap().path();

            if let Some(os_extension) = file_path.extension() {
                if let Some(extension) = os_extension.to_str() {
                    if extension == HOURGLASS_EXTENSION {
                        file_exists = true;

                        let content =
                            fs::read_to_string(file_path).expect("Unable to read .hourglass file");

                        let datas: Vec<Task> = serde_json::from_str(&content)?;

                        self.tasks = datas;
                    }
                }
            }
        }

        if !file_exists {
            fs::write(HOURGLASS_FILE_STORAGE_NAME, "")?;
        }

        Ok(())
    }

    fn save_tasks(&self) {
        let serialized = serde_json::to_string(&self.tasks).unwrap();

        fs::write(HOURGLASS_FILE_STORAGE_NAME, serialized).expect("Unable to write to file");
    }
}
