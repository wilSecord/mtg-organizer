use std::io::{self, BufRead, BufReader};
use std::fs::File;
use nucleo_matcher::pattern::{Normalization, CaseMatching, Pattern};
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

// Setup App struct
struct App {
    search: String,
    input_mode: InputMode,
    exit: bool,
    contents: Vec<String>,
    results: Vec<String>,
}

// Implement App
impl App {    
    // new function initializing most things
    const fn new() -> Self {
        Self {
            search: String::new(),
            input_mode: InputMode::Normal,
            exit: false,
            contents: Vec::new(),
            results: Vec::new(),
        }
    }

    pub fn run(&mut self, term: &mut DefaultTerminal) -> io::Result<()> {
        //TODO: Make the matcher object and contents able to be read by get_results()

        // TEMP
        // TEMP
        let file_path = "cards.txt"; 
    
        let file = File::open(file_path).expect("File not found.");
        let buf = BufReader::new(file);
        self.contents = buf.lines().map(|l| l.expect("Could not parse")).collect();
        // TEMP
        // TEMP

        let mut matcher = Matcher::new(Config::DEFAULT);

        // As long as self.exit == true, run the gameloop stuff (drawing, handling inputs)
        while !self.exit {
            term.draw(|frame| self.draw(frame))?; // Drawing
            self.handle_events(&mut matcher)?;    // Handling inputs
        }
        Ok(()) // ok :+1:
    }

    fn draw(&self, frame: &mut Frame) {
        // Full layout 
        let total = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(75),
                Constraint::Percentage(25),
            ]).split(frame.area());
        
        // Setting up the left side of the screen
        let left = Layout::default()
            .direction(Direction::Vertical) // Multiple tiles on top of each other
            .constraints([
                Constraint::Length(1), // Help line
                Constraint::Length(3), // Input box
                Constraint::Min(1),    // Results box
            ])
            .split(total[0]);

        let help_area = left[0];      // Area for keybinds/help text
        let input_area = left[1];     // Area for input box                 || TODO: Refactor to searchbar
        let body_area = left[2];      // Area for results box               || TODO: Rename this
        let decklist_area = total[1]; // Area for decklist (rename this?)

        // Outline the searchbar/input box
        let search = Paragraph::new(self.search.as_str())
            // Change style based on if the person is typing in it or not
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input")); // Set border as box
        
        // Help text area
        // TODO: Change this lol
        let (msg, style) = match self.input_mode {
            InputMode::Normal => ("Normal", Style::default()),
            InputMode::Editing => ("Editing", Style::default()),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_msg = Paragraph::new(text);
        
        // Results area
        let body = Paragraph::new("test").block(Block::bordered().title("Results"));

        // Render stuff
        frame.render_widget(help_msg, help_area);
        frame.render_widget(search, input_area);
        frame.render_widget(body, body_area);
        frame.render_widget(Paragraph::new("thing").block(Block::bordered().title("Decklist")), decklist_area);

    }

    fn get_results(& mut self, matcher: &mut Matcher) {
        self.results = Pattern::parse(&self.search, CaseMatching::Ignore, Normalization::Smart).match_list(&self.contents, matcher).into_iter().map(|x| x.0.to_owned()).collect();
        //TODO Update Results widget
    }

    fn delete_char(& mut self, matcher: &mut Matcher) {
        self.search.pop();
        self.get_results(matcher);
    }

    fn add_char(& mut self, c: char, matcher: &mut Matcher) {
        self.search.push(c);
        self.get_results(matcher);
    }

    fn handle_events(&mut self, matcher: &mut Matcher) -> io::Result<()> {
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
                    KeyCode::Backspace => self.delete_char(matcher),
                    KeyCode::Char(c) => self.add_char(c, matcher),
                    KeyCode::Esc => self.input_mode = InputMode::Normal,
                    _ => {}
                }
                _ => {}
            }
        }
        Ok(())
    }
}
