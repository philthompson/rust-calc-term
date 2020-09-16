use std::io::{stdin, stdout, Write};
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use eval::{eval, Value};

enum CalcKey {
    Key(char),
    Delete
}

enum CalcResult {
    Float(f64),
    Integer(i64),
    Error(String)
}

struct Calculator {
    calc: String,
    calc_pos: u16,
    prev_calcs: Vec<(String, CalcResult)>,
    selected_calc: u8,
    selected_equals: bool,
}

fn main() {
    let mut is_help_requested = false;
    let help_text_short: String = format!(
        "{}{}[h: help] [ctrl+q: quit]{}{}",
        color::Bg(color::AnsiValue::grayscale(5)),
        color::Fg(color::AnsiValue::grayscale(11)),
        color::Bg(color::Reset),
        color::Fg(color::Reset));
    let help_text_long: String = format!(
        "{}{}Type an expression, like \"355/113\" or \"(9+8)/(7+6)\" and hit return!\n\rprevious calculations: [←↑↓→: select] [space: use selected] [pgdn/pgup: show more/fewer prevs]\n\rediting: [home/end: move]\n\rother: [h: hide help] [ctrl+q: quit]{}{}",
        color::Bg(color::AnsiValue::grayscale(5)),
        color::Fg(color::AnsiValue::grayscale(11)),
        color::Bg(color::Reset),
        color::Fg(color::Reset));
    let mut help_text = &help_text_short;

    let mut calc = Calculator {
        calc: String::from(""),
        calc_pos: 0,
        prev_calcs: vec![],
        selected_calc: 0,
        selected_equals: false,
    };
    let mut history_items: u8 = 10;

    let stdin = stdin();
    //setting up stdout and going into raw mode
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{}{}\n{}{}", termion::clear::All, termion::cursor::Goto(1,1), help_text, termion::cursor::Goto(1,1)).unwrap();

    stdout.flush().unwrap();

    //detecting keydown events
    for c in stdin.keys() {

        let key = c.unwrap();

        match &key {
            Key::Ctrl('q') => break,
            Key::Ctrl('c') => break,
            Key::Char('0') => calc.append_key_to_calc(&CalcKey::Key('0')),
            Key::Char('1') => calc.append_key_to_calc(&CalcKey::Key('1')),
            Key::Char('2') => calc.append_key_to_calc(&CalcKey::Key('2')),
            Key::Char('3') => calc.append_key_to_calc(&CalcKey::Key('3')),
            Key::Char('4') => calc.append_key_to_calc(&CalcKey::Key('4')),
            Key::Char('5') => calc.append_key_to_calc(&CalcKey::Key('5')),
            Key::Char('6') => calc.append_key_to_calc(&CalcKey::Key('6')),
            Key::Char('7') => calc.append_key_to_calc(&CalcKey::Key('7')),
            Key::Char('8') => calc.append_key_to_calc(&CalcKey::Key('8')),
            Key::Char('9') => calc.append_key_to_calc(&CalcKey::Key('9')),
            Key::Char('+') => calc.append_key_to_calc(&CalcKey::Key('+')),
            Key::Char('-') => calc.append_key_to_calc(&CalcKey::Key('-')),
            Key::Char('*') => calc.append_key_to_calc(&CalcKey::Key('*')),
            Key::Char('/') => calc.append_key_to_calc(&CalcKey::Key('/')),
            Key::Char('(') => calc.append_key_to_calc(&CalcKey::Key('(')),
            Key::Char(')') => calc.append_key_to_calc(&CalcKey::Key(')')),
            Key::Char('.') => calc.append_key_to_calc(&CalcKey::Key('.')),
            Key::Char('h') => {
                is_help_requested = !is_help_requested;
                if is_help_requested {
                    help_text = &help_text_long;
                } else {
                    help_text = &help_text_short;
                }
            },
            Key::Backspace => calc.append_key_to_calc(&CalcKey::Delete),
            Key::PageDown => {
                if history_items < 100 && usize::from(history_items) < calc.prev_calcs.len() {
                    history_items += 1;
                }
            },
            Key::PageUp => {
                if !calc.prev_calcs.is_empty() {
                    if usize::from(history_items) > calc.prev_calcs.len() {
                        while usize::from(history_items) >= calc.prev_calcs.len() {
                            history_items -= 1;
                        }
                    } else if history_items > 0 {
                        history_items -= 1;
                    }
                }
            },
            Key::Char(' ') => {
                if calc.selected_calc == 0 {
                    calc.append_key_to_calc(&CalcKey::Key(' '));
                } else {
                    calc.recall_previous_calc();
                }
            },
            Key::Left => {
                if calc.selected_calc == 0 {
                    if calc.calc_pos > 0 {
                        calc.calc_pos -= 1;
                    }
                } else {
                    calc.selected_equals = false;
                }
            },
            Key::Right => {
                if calc.selected_calc == 0 {
                    if usize::from(calc.calc_pos) < calc.calc.len() {
                        calc.calc_pos += 1;
                    }
                } else {
                    calc.selected_equals = true;
                }
            },
            Key::Home => calc.move_cursor_home(),
            Key::End => calc.move_cursor_end(),
            Key::Up => {
                if calc.selected_calc > 0 {
                    calc.selected_calc -= 1;
                }
            },
            Key::Down => {
                if calc.selected_calc < history_items &&
                        usize::from(calc.selected_calc) < calc.prev_calcs.len() {
                    calc.selected_calc += 1;
                }
            }
            Key::Char('\n') => calc.perform_calculation(),
            //x => { calc.calc = format!("{:?}", x); }
            _ => ()
        }

        write!(stdout, "{}{}{}\n",
            // clear the screen,
            termion::clear::All,
            // go to top left corner
            termion::cursor::Goto(1,1),
            // print the currently-being-typed calculation
            &calc.calc).unwrap();

        // print the last N previous calcs in backwards order
        let mut line = 1;
        for (input, output) in calc.prev_calcs.iter().rev().take(history_items.into()) {
            // check if this line is selected before incrementing line number, since
            //   selected previous calcs start at line 1 and printed previous calcs
            //   start at 2nd line on screen
            let mut is_selected_left = false;
            let mut is_selected_right = false;
            if calc.selected_calc == line {
                if calc.selected_equals {
                    is_selected_right = true;
                } else {
                    is_selected_left = true;
                }
            }
            line += 1;
            let formatted = Calculator::format_prev_calculation(&output);
            if is_selected_left {
                write!(stdout, "{}{}{}{}{}{} = {}\n",
                    termion::cursor::Goto(1,line.into()),
                    color::Bg(color::Blue),
                    color::Fg(color::Yellow),
                    input,
                    color::Bg(color::Reset),
                    color::Fg(color::Reset),
                    formatted).unwrap();
            } else if is_selected_right {
                write!(stdout, "{}{} = {}{}{}{}{}\n",
                    termion::cursor::Goto(1,line.into()),
                    input,
                    color::Bg(color::Blue),
                    color::Fg(color::Yellow),
                    formatted,
                    color::Bg(color::Reset),
                    color::Fg(color::Reset)).unwrap();
            } else {
                write!(stdout, "{}{} = {}\n",
                    termion::cursor::Goto(1,line.into()),
                    input,
                    formatted).unwrap();
            }
        }

        line += 1;
        write!(stdout, "{}{}\n",
            termion::cursor::Goto(1,line.into()),
            help_text).unwrap();

        write!(stdout, "{}",
            // go to end of currently-being-typed calculation
            termion::cursor::Goto(calc.calc_pos+1,1)).unwrap();

        stdout.flush().unwrap();
    }
}

impl Calculator {

    fn append_key_to_calc(&mut self, k: &CalcKey) {
        self.selected_calc = 0;
        self.selected_equals = false;
        match k {
            CalcKey::Key(x) => {
                self.calc.insert((self.calc_pos).into(), *x);
                self.calc_pos += 1;
            },
            CalcKey::Delete => {
                if self.calc_pos == 0 {
                    return;
                }
                let mut delete_pos: usize = (self.calc_pos).into();
                delete_pos -= 1;
                self.calc.remove(delete_pos);
                if self.calc_pos > 0 {
                    self.calc_pos -= 1;
                }
            },
        }
    }

    fn perform_calculation(&mut self) {
        let calc_copy = self.calc.clone();
        let calc_equals = match eval(&self.calc) {
            Ok(value) => match value {
                Value::Number(number) => {
                    if number.is_i64() || number.is_u64() {
                        CalcResult::Integer(number.as_i64().unwrap())
                    } else {
                        CalcResult::Float(number.as_f64().unwrap())
                    }
                },
                _ => CalcResult::Error(String::from("error")),
            },
            _ => CalcResult::Error(String::from("error"))
        };
        self.prev_calcs.push((calc_copy, calc_equals));
        while self.prev_calcs.len() > 1000 {
            self.prev_calcs.remove(0);
        }
        self.calc.clear();
        self.calc_pos = 0;
    }

    fn format_prev_calculation(output: &CalcResult) -> String {
        let mut formatted = String::from("");

        let formatted_output = match &output {
            CalcResult::Float(value) => value.to_string(),
            CalcResult::Integer(value) => value.to_string(),
            CalcResult::Error(string) => String::from(string)
        };
        formatted.push_str(&formatted_output);

        return formatted;
    }

    fn recall_previous_calc(&mut self) {
        let prev: &(String, CalcResult) =
            self.prev_calcs.get(
                self.prev_calcs.len() - usize::from(self.selected_calc))
                    .unwrap();
        self.calc.clear();
        if self.selected_equals {
            self.calc.push_str(&Calculator::format_prev_calculation(&prev.1));
        } else {
            self.calc.push_str(&prev.0);
        }
        self.move_cursor_end();
    }

    fn move_cursor_home(&mut self) {
        self.selected_calc = 0;
        self.selected_equals = false;
        self.calc_pos = 0;
    }

    fn move_cursor_end(&mut self) {
        // in case the new calc is shorter than the previous one, start by moving all the way home
        self.move_cursor_home();
        while usize::from(self.calc_pos) < self.calc.len() {
            self.calc_pos += 1;
        }
    }
}