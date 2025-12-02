use std::any::type_name;
use std::io::{self, BufRead, BufReader};
use std::fs::{File, read_to_string};
use serde_json;
use nucleo_matcher::pattern::{Normalization, CaseMatching, Pattern};
use nucleo_matcher::{Matcher, Config};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{ 
    layout::{Constraint, Direction, Layout, Flex, Rect}, 
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, List, ListState, Paragraph},
    DefaultTerminal, Frame,
};

use project::dbs::allcards::AllCardsDb;
use project::query;


enum InputMode {
    Normal, Editing, Decklist,
}

// Setup App struct
struct App {
    search: String,
    input_mode: InputMode,
    exit: bool,
    contents: Vec<String>,
    results: Vec<String>,
    selected: usize,
    decklist: Vec<String>,
    decklist_selected: usize,
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
            selected: 0,
            decklist: Vec::new(),
            decklist_selected: 0,
        }
    }

    pub fn run(&mut self, term: &mut DefaultTerminal) -> io::Result<()> {
        //TODO: Make the matcher object and contents able to be read by get_results()

        // TEMP
        // TEMP
        let db_file = "/home/wil/Documents/school/software/project/db";
        let db = AllCardsDb::open(db_file)?;       
        self.contents = db.all_cards().map(|x| x.name).collect::<Vec<_>>();

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

        // let text_pop = Paragraph::new("something").block(Block::bordered().title("Tag"));
        // let popup_area = center(
        //     frame.area(),
        //     Constraint::Length(20),
        //     Constraint::Length(5)
        // );
        // frame.render_widget(text_pop, popup_area);

        let help_area = left[0];      // Area for keybinds/help text
        let input_area = left[1];     // Area for input box                 || TODO: Refactor to searchbar
        let body_area = left[2];      // Area for results box               || TODO: Rename this
        let decklist_area = total[1]; // Area for decklist (rename this?)

        // Outline the searchbar/input box
        let search = Paragraph::new(self.search.as_str())
            // Change style based on if the person is typing in it or not
            .style(match self.input_mode {
                InputMode::Editing => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .block(Block::bordered().title("Input")); // Set border as box
        
        // Help text area
        // TODO: Change this lol
        let (msg, style) = match self.input_mode {
            InputMode::Normal => ("Normal", Style::default()),
            InputMode::Editing => ("Editing", Style::default()),
            InputMode::Decklist => ("Decklist", Style::default()),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_msg = Paragraph::new(text);
        
        // Results area
        // let body = Paragraph::new(self.results.join("\n")).block(Block::bordered().title("Results"));
        let mut state = ListState::default();
        let body = List::new(self.results.clone())
            .block(Block::bordered().title("Results"))
            .highlight_style(Style::new().reversed());

        match self.input_mode {
                InputMode::Normal => state.select(Some(self.selected)),
                _ => state.select(None),
            }

        // Decklist area
        let mut deck_state = ListState::default();
        let decklist = List::new(self.decklist.clone())
            .block(Block::bordered().title("Decklist"))
            .highlight_style(Style::new().reversed());

        match self.input_mode {
                InputMode::Decklist => deck_state.select(Some(self.decklist_selected)),
                _ => deck_state.select(None),
            }

        // Render stuff
        frame.render_widget(help_msg, help_area);
        frame.render_widget(search, input_area);
        frame.render_stateful_widget(body, body_area, &mut state);
        frame.render_stateful_widget(decklist, decklist_area, &mut deck_state);

    }

    fn get_results(& mut self, matcher: &mut Matcher) {
        self.results = Pattern::parse(&self.search, CaseMatching::Ignore, Normalization::Smart)
            .match_list(&self.contents, matcher)
            .into_iter()
            .map(|x| x.0.to_owned())
            .collect();
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
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Char('/') => self.input_mode = InputMode::Editing,
                    KeyCode::Char('f') => self.input_mode = InputMode::Decklist,
                    KeyCode::Char('j') => {
                        if self.results.len() > 0 {
                            self.selected = if self.selected < (self.results.len() - 1) { self.selected + 1 } else { 0 }
                        }
                    }
                    KeyCode::Char('k') => {
                        if self.results.len() > 0 {
                            self.selected = if self.selected > 0 { self.selected - 1 } else { self.results.len() - 1 }
                        }
                    }
                    KeyCode::Enter => {
                        let sel = self.results[self.selected].clone();
                        self.decklist.push(sel);
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
                InputMode::Decklist if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('d') => {
                        if self.decklist.len() > 1 {
                            self.decklist.remove(self.decklist_selected);
                            self.decklist_selected = if self.decklist_selected == 0 { 0 } else { self.decklist_selected - 1 };
                        } else if self.decklist.len() == 1 {
                            self.decklist.remove(self.decklist_selected);
                            self.input_mode = InputMode::Normal;
                        }
                    }
                    KeyCode::Enter => {
                        let sel = self.decklist[self.decklist_selected].clone();
                        self.decklist.push(sel);
                    }
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Char('/') => self.input_mode = InputMode::Editing,
                    KeyCode::Esc => self.input_mode = InputMode::Normal,
                    KeyCode::Char('j') => {
                        if self.decklist.len() > 0 {
                            self.decklist_selected = if self.decklist_selected < (self.decklist.len() - 1) { self.decklist_selected + 1 } else { 0 }
                        }
                    }
                    KeyCode::Char('k') => {
                        if self.decklist.len() > 0 {
                            self.decklist_selected = if self.decklist_selected > 0 { self.decklist_selected - 1 } else { self.decklist.len() - 1 }
                        }
                    }
                    _ => {}
                }
                _ => {}
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('s') => save_decklist(self.decklist.clone()),
                    _ => ()
                }
            }
        }
        Ok(())
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn save_decklist(deck: Vec<String>) {
    panic!("Not implemented");
}

// MAKES IT RUN
fn main() -> io::Result<()> {
    let mut term = ratatui::init();
    let app_result = App::new().run(&mut term);
    ratatui::restore();
    app_result
}

// fn main() -> io::Result<()> {
//     let db_file = "/home/wil/Documents/school/software/project/db";
//     let db = AllCardsDb::open(db_file)?;       
//     Ok(())
// }
