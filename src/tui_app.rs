use core::panic;
use std::{collections::HashMap, io};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, ListState, Paragraph, Widget, Wrap},
};

use crate::data_strcuts::Value;
use crate::tui_fuzzy_widget::FuzzyListWidget;

#[derive(PartialEq, Debug, Default)]
enum InputMode {
    #[default]
    Amount,
    FromCurrency,
    ToCurrency,
}

#[derive(Default)]
pub struct App<'a> {
    // Currency values as string
    amount: String,
    result: String,
    // Control flow of the TUI
    exit: bool,
    input_mode: InputMode,
    // Static data
    currencies: Vec<&'a str>,
    map: HashMap<&'a str, Value<'a>>,
    // Fields relevant for fuzzy search and rendering
    from_input: String,
    to_input: String,
    from_index: ListState,
    to_index: ListState,
    from_fuzzy_matches: Vec<(&'a str, i64)>,
    to_fuzzy_matches: Vec<(&'a str, i64)>,
    matcher: SkimMatcherV2,
}

impl<'a> App<'a> {
    pub fn new(curs: Vec<&'static str>, map: HashMap<&'static str, Value<'static>>) -> Self {
        let matches: Vec<(&str, i64)> = curs.iter().map(|s| (*s, 0)).collect();
        Self {
            amount: String::from("0"),
            result: String::from(""),
            exit: false,
            from_input: String::from(""),
            to_input: String::from(""),
            input_mode: InputMode::Amount,
            from_index: ListState::default().with_selected(Some(0)),
            to_index: ListState::default().with_selected(Some(0)),
            currencies: curs,
            matcher: SkimMatcherV2::default(),
            from_fuzzy_matches: matches.clone(),
            to_fuzzy_matches: matches,
            map,
        }
    }
}

impl<'a> App<'a> {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut from_list_state = ListState::default();
        let mut to_list_state = ListState::default();
        from_list_state.select(Some(0));
        to_list_state.select(Some(0));

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit()
            }
            KeyCode::Esc => self.exit(),
            KeyCode::Tab => {
                self.input_mode = match self.input_mode {
                    InputMode::Amount => InputMode::FromCurrency,
                    InputMode::FromCurrency => InputMode::ToCurrency,
                    InputMode::ToCurrency => InputMode::Amount,
                };
            }
            KeyCode::BackTab => {
                self.input_mode = match self.input_mode {
                    InputMode::Amount => InputMode::ToCurrency,
                    InputMode::FromCurrency => InputMode::Amount,
                    InputMode::ToCurrency => InputMode::FromCurrency,
                };
            }
            KeyCode::Char(c) => match self.input_mode {
                InputMode::Amount if c.is_ascii_digit() || c == '.' => self.amount.push(c),
                InputMode::FromCurrency => {
                    self.from_input.push(c);
                    self.update_fuzzy_matches(InputMode::FromCurrency);
                }
                InputMode::ToCurrency => {
                    self.to_input.push(c);
                    self.update_fuzzy_matches(InputMode::ToCurrency);
                }
                _ => {}
            },
            KeyCode::Backspace => match self.input_mode {
                InputMode::Amount => {
                    Self::backspace_field(&mut self.amount);
                }
                InputMode::FromCurrency => {
                    self.from_input.pop();
                    self.update_fuzzy_matches(InputMode::FromCurrency);
                    if self.from_index.selected().is_none() && !self.from_fuzzy_matches.is_empty() {
                        self.from_index.select(Some(0));
                    }
                }
                InputMode::ToCurrency => {
                    self.to_input.pop();
                    self.update_fuzzy_matches(InputMode::ToCurrency);
                    if self.to_index.selected().is_none() && !self.to_fuzzy_matches.is_empty() {
                        self.to_index.select(Some(0));
                    }
                }
            },
            KeyCode::Enter => {
                self.result = self.calculate();
            }
            KeyCode::Up => match self.input_mode {
                InputMode::Amount => {}
                InputMode::FromCurrency => {
                    Self::move_selection(
                        &mut self.from_index,
                        self.from_fuzzy_matches.len(),
                        false,
                    );
                    self.update_fuzzy_matches(InputMode::FromCurrency);
                }
                InputMode::ToCurrency => {
                    Self::move_selection(&mut self.to_index, self.to_fuzzy_matches.len(), false);
                    self.update_fuzzy_matches(InputMode::ToCurrency);
                }
            },
            KeyCode::Down => match self.input_mode {
                InputMode::Amount => {}
                InputMode::FromCurrency => {
                    Self::move_selection(&mut self.from_index, self.from_fuzzy_matches.len(), true);
                }
                InputMode::ToCurrency => {
                    Self::move_selection(&mut self.to_index, self.to_fuzzy_matches.len(), true)
                }
            },
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn backspace_field(field: &mut String) {
        if field.len() <= 1 {
            *field = String::from("0");
        } else {
            field.pop();
        }
    }

    fn current_currency(&self, mode: InputMode) -> &str {
        let mut ret = "a";
        let mut lsst: &ListState = &self.from_index;
        let mut matches = &self.from_fuzzy_matches;
        match mode {
            InputMode::FromCurrency => {
                lsst = &self.from_index;
                matches = &self.from_fuzzy_matches
            }
            InputMode::ToCurrency => {
                lsst = &self.to_index;
                matches = &self.to_fuzzy_matches
            }
            _ => {}
        }
        if let Some(index) = lsst.selected()
            && let Some((currency_code, _score)) = matches.get(index)
        {
            ret = *currency_code;
        }
        ret
    }

    fn calculate(&self) -> String {
        let mut res: f64 = 0.0;
        let val: f64 = self.amount.parse().unwrap();
        let cur1 = self.current_currency(InputMode::FromCurrency);
        let cur2 = self.current_currency(InputMode::ToCurrency);
        if let (Some(Value::Float(fac_from)), Some(Value::Float(fac_to))) =
            (self.map.get(&cur1), self.map.get(&cur2))
        {
            res = val * fac_to / fac_from;
        }
        format!("{} {} = {} {}\n", val, cur1, res, cur2)
    }

    fn move_selection(state: &mut ListState, len: usize, down: bool) {
        if len == 0 {
            return;
        }
        let i = match state.selected() {
            Some(i) => {
                if down {
                    (i + 1) % len
                } else {
                    (i + len - 1) % len
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }

    fn update_fuzzy_matches(&mut self, mode: InputMode) {
        let (fuzzy_input, fuzzy_matches) = match mode {
            InputMode::FromCurrency => (&mut self.from_input, &mut self.from_fuzzy_matches),
            InputMode::ToCurrency => (&mut self.to_input, &mut self.to_fuzzy_matches),
            _ => panic!("Cannot pass this"),
        };

        *fuzzy_matches = {
            let filter: &str = fuzzy_input;
            let mut matches: Vec<(&str, i64)> = self
                .currencies
                .iter()
                .filter_map(|c| {
                    if filter.is_empty() {
                        return Some((*c, 0));
                    }
                    self.matcher.fuzzy_match(c, filter).map(|score| (*c, score))
                })
                .collect();

            if !filter.is_empty() {
                matches.sort_by(|a, b| b.1.cmp(&a.1));
            }
            matches
        };
    }

    fn draw(&mut self, frame: &mut Frame) {
        // 1. Create your main 4-column layout
        let chunks = Layout::horizontal([
            Constraint::Percentage(15),
            Constraint::Percentage(30), // From Column
            Constraint::Percentage(30), // To Column
            Constraint::Percentage(25),
        ])
        .split(frame.area());
        frame.render_widget(InputArea { app: self }, chunks[0]);

        {
            // 2. Prepare the "From" List data
            let from_widget = FuzzyListWidget {
                title: " From Currency ",
                input: &self.from_input,
                items: self.from_fuzzy_matches.iter().map(|m| m.0).collect(),
                is_focused: self.input_mode == InputMode::FromCurrency,
            };
            // 3. Render it passing the mutable state
            frame.render_stateful_widget(from_widget, chunks[1], &mut self.from_index);
        }
        {
            // 4. Prepare the "To" List data
            let to_widget = FuzzyListWidget {
                title: " To Currency ",
                input: &self.to_input,
                items: self.to_fuzzy_matches.iter().map(|m| m.0).collect(),
                is_focused: self.input_mode == InputMode::ToCurrency,
            };
            // 3. Render it passing the mutable state
            frame.render_stateful_widget(to_widget, chunks[2], &mut self.to_index);
        }

        frame.render_widget(OutputArea { app: self }, chunks[3]);
    }
}

pub struct InputArea<'a> {
    app: &'a App<'a>,
}
pub struct OutputArea<'a> {
    app: &'a App<'a>,
}
impl<'a> Widget for OutputArea<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Result ".bold());
        let instructions = Line::from(vec![" Quit <Esc | Ctrl+C> ".blue().bold()]);

        // --- COLUMN 1: AMOUNT ---
        let amount_style = if self.app.input_mode == InputMode::Amount {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let val = self.app.calculate();
        Paragraph::new(val)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title.clone())
                    .title_bottom(instructions.clone().centered())
                    .border_style(amount_style),
            )
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
impl<'a> Widget for InputArea<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Amount ".bold());

        // --- COLUMN 1: AMOUNT ---
        let amount_style = if self.app.input_mode == InputMode::Amount {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let mut print_this = self.app.amount.clone();
        if print_this.starts_with("0") && !print_this.is_empty() {
            print_this.remove(0);
        }
        Paragraph::new(print_this)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title.clone())
                    .border_style(amount_style),
            )
            .render(area, buf);
    }
}

impl<'a> App<'a> {}
