use std::io::{self, BufRead, BufReader};
use std::fs::File;
use nucleo_matcher::pattern::{Normalization, CaseMatching, Pattern, AtomKind};
use nucleo_matcher::{Matcher, Config};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};

mod data_model;
mod dbs;

fn main() -> io::Result<()> {
    let mut term = ratatui::init();
    let app_result = App::new().run(&mut term);
    ratatui::restore();
    app_result
}

// fn main() -> io::Result<()> {
//     let file_path = "cards.txt"; 
// 
//     let file = File::open(file_path).expect("File not found.");
//     let buf = BufReader::new(file);
//     let contents: Vec<String> = buf.lines().map(|l| l.expect("Could not parse")).collect();
// 
//     let mut matcher = Matcher::new(Config::DEFAULT);
// 
//     let matches = Pattern::parse("Angel", CaseMatching::Ignore, Normalization::Smart).match_list(contents, &mut matcher);
//     println!("{:?}", matches);
// 
//     Ok(())
// }

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
            exit: false,
        }
    }

    pub fn run(&mut self, term: &mut DefaultTerminal) -> io::Result<()> {
        //TODO: Make the matcher object and contents able to be read by get_results()
        // let file_path = "cards.txt"; 
    
        // let file = File::open(file_path).expect("File not found.");
        // let buf = BufReader::new(file);
        // self.contents = buf.lines().map(|l| l.expect("Could not parse")).collect();
    
        // let mut matcher = Matcher::new(Config::DEFAULT);

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

    fn get_results(&self) {
        // let matches = Pattern::parse(&self.search, CaseMatching::Ignore, Normalization::Smart).match_list(self.contents, matcher);
        todo!();
    }

    fn delete_char(& mut self) {
        self.search.pop();
        self.get_results()
        // self.results = self.get_results(matcher);
    }

    fn add_char(& mut self, c: char) {
        self.search.push(c);
        self.get_results()
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
                    KeyCode::Enter => self.input_mode = InputMode::Normal,
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
