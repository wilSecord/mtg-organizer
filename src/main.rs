use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    let mut term = ratatui::init();
    let app_result = App::new().run(&mut term);
    ratatui::restore();
    app_result
}

enum InputMode {
    Normal,
    Editing,
}

struct App {
    search: String,
    input_mode: InputMode,
    exit: bool,
}

impl App {    

    const fn new() -> Self {
        Self {
            search: String::new(),
            input_mode: InputMode::Normal,
            exit: false
        }
    }

    pub fn run(&mut self, term: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            term.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let total = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(75),
                Constraint::Percentage(25),
            ]).split(frame.area());

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(total[0]);

        let help_area = left[0];
        let input_area = left[1];
        let body_area = left[2];
        let decklist_area = total[1];

        let search = Paragraph::new(self.search.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(search, input_area);

        let (msg, style) = match self.input_mode {
            InputMode::Normal => ("Normal", Style::default()),
            InputMode::Editing => ("Editing", Style::default()),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_msg = Paragraph::new(text);
        frame.render_widget(help_msg, help_area);

        let body = Paragraph::new("test").block(Block::bordered().title("Results"));

        frame.render_widget(body, body_area);

        frame.render_widget(Paragraph::new("thing").block(Block::bordered().title("Decklist")), decklist_area);

    }

    fn delete_char(& mut self) {
        self.search.pop();
    }

    fn add_char(& mut self, c: char) {
        self.search.push(c);
    }

    fn send_search(&self) {
        todo!();
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            match self.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        self.exit = true;
                    }
                    KeyCode::Char('f') => {
                        self.input_mode = InputMode::Editing;
                    }
                    _ => {}
                }
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => self.send_search(),
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Char(c) => self.add_char(c),
                    KeyCode::Esc => self.input_mode = InputMode::Normal,
                    _ => {}
                }
                _ => {}
            }
        }
        Ok(())
    }
}
