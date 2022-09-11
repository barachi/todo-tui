use std::{ io };
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, ListState},
    Frame, Terminal,
};

struct StateList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StateList<T> {
    fn with_items(items: Vec<T>) -> StateList<T> {
        StateList { state: ListState::default(), items }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn unselect(&mut self) {
        self.state.select(None);
    }
    fn push(&mut self, value: T) {
        self.items.push(value);
    }
}

enum InputMode {
    Normal,
    Editing,
}

struct App {
    popup_input: String,
    input_mode: InputMode,
    input_width: u16,
    items: StateList<(String, usize)>,
    show_popup: bool,
}

impl App {
    fn new() -> App {
        App {
            items: StateList::with_items(vec![]),
            input_mode: InputMode::Normal,
            input_width: 0,
            show_popup: false,
            popup_input: String::new(),
        }
    }
    fn input_width(&self) -> u16 {
        let width = self.input_width;
        return width;
    }
    fn set_input_width(&mut self) {
        self.input_width = self.popup_input.chars().count() as u16;
    }
    fn push(&mut self) {
        let new_value = self.popup_input.to_string(); 
        self.items.push((new_value.to_string(), 1));
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        if let Event::Key(KeyEvent {code, modifiers, ..}) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match (code, modifiers) {
                    (KeyCode::Char('p'), KeyModifiers::NONE) => {
                        app.show_popup = !app.show_popup;
                        app.input_mode = InputMode::Editing;
                    },
                    (KeyCode::Esc, KeyModifiers::NONE) => {
                        return Ok(());
                    },
                    (KeyCode::Left, _) => app.items.unselect(),
                    (KeyCode::Down, _) => app.items.next(),
                    (KeyCode::Up, _) => app.items.previous(),
                    _ => {}
                },
                InputMode::Editing => match (code, modifiers) {
                    (KeyCode::Enter, KeyModifiers::SHIFT) => {},
                    (KeyCode::Enter, KeyModifiers::NONE) => {
                        app.show_popup = !app.show_popup;
                        app.push();
                        app.popup_input = String::new();
                        app.input_mode = InputMode::Normal;
                        app.set_input_width();
                    },
                    (KeyCode::Char(c), _) => {
                        if app.show_popup {
                            app.popup_input.push(c);
                            app.set_input_width();
                        }
                    },
                    (KeyCode::Backspace, KeyModifiers::NONE) => {
                        app.popup_input.pop();
                        app.set_input_width();
                    },
                    (KeyCode::Esc, KeyModifiers::NONE) => {
                        if app.show_popup {
                            app.popup_input = String::new();
                            app.input_mode = InputMode::Normal;
                            app.show_popup = !app.show_popup;
                        }
                    },
                    _ => {}
                }
            }
        }
    }

}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // window setting
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),
            Constraint::Percentage(90),
        ].as_ref(),)
        .split(f.size());
    let main = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Percentage(100),
        ].as_ref(),)
        .split(chunks[1]);

    // help message
    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc key", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to input popup."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop edit, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to add todo list. "),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    // todo list ui
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let lines = vec![Spans::from((i.0).to_string())];
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("TODO List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, main[0], &mut app.items.state);

    // popup ui
    let size = f.size();
    if app.show_popup {
        let items: Vec<ListItem> = vec![
            ListItem::new(app.popup_input.to_string())
        ];
        let items = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Add TODO"))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        let area = centered_rect(60, 10, size);
        match app.input_mode {
            InputMode::Normal => {},
            InputMode::Editing => {
                f.set_cursor(
                    area.x + app.input_width() as u16 + 1,
                    area.y + 1,
                )
            }
        }
        f.render_widget(Clear, area);
        f.render_widget(items, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
