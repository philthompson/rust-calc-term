use std::io::{stdin, stdout, Write};
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
    let mut calc = Calculator {
        calc: String::from(""),
        calc_pos: 0,
        prev_calcs: vec![],
        selected_calc: 0,
        selected_equals: false,
    };
    let history_items: u8 = 10;

    let stdin = stdin();
    //setting up stdout and going into raw mode
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1,1),).unwrap();

    //printing welcoming message, clearing the screen and going to left top corner with the cursor
    //write!(stdout, r#"{}{}ctrl + q to exit, ctrl + h to print "Hello world!", alt + t to print "termion is cool""#, termion::cursor::Goto(1, 1), termion::clear::All)
    //        .unwrap();
    stdout.flush().unwrap();

    //detecting keydown events
    for c in stdin.keys() {

        let key = c.unwrap();

        match &key {
            Key::Ctrl('q') => break,
            Key::Ctrl('c') => break,
            // TODO implement as method so we can do this instead:
            //Key::Char('0') => calc.append_key_to_calc(&CalcKey::Key('0')),
            Key::Char('0') => append_key_to_calc(&mut calc, &CalcKey::Key('0')),
            Key::Char('1') => append_key_to_calc(&mut calc, &CalcKey::Key('1')),
            Key::Char('2') => append_key_to_calc(&mut calc, &CalcKey::Key('2')),
            Key::Char('3') => append_key_to_calc(&mut calc, &CalcKey::Key('3')),
            Key::Char('4') => append_key_to_calc(&mut calc, &CalcKey::Key('4')),
            Key::Char('5') => append_key_to_calc(&mut calc, &CalcKey::Key('5')),
            Key::Char('6') => append_key_to_calc(&mut calc, &CalcKey::Key('6')),
            Key::Char('7') => append_key_to_calc(&mut calc, &CalcKey::Key('7')),
            Key::Char('8') => append_key_to_calc(&mut calc, &CalcKey::Key('8')),
            Key::Char('9') => append_key_to_calc(&mut calc, &CalcKey::Key('9')),
            Key::Char('+') => append_key_to_calc(&mut calc, &CalcKey::Key('+')),
            Key::Char('-') => append_key_to_calc(&mut calc, &CalcKey::Key('-')),
            Key::Char('*') => append_key_to_calc(&mut calc, &CalcKey::Key('*')),
            Key::Char('/') => append_key_to_calc(&mut calc, &CalcKey::Key('/')),
            Key::Char('(') => append_key_to_calc(&mut calc, &CalcKey::Key('(')),
            Key::Char(')') => append_key_to_calc(&mut calc, &CalcKey::Key(')')),
            Key::Char(' ') => append_key_to_calc(&mut calc, &CalcKey::Key(' ')),
            Key::Backspace => append_key_to_calc(&mut calc, &CalcKey::Delete),
            Key::Left => {
                if calc.calc_pos > 0 {
                    calc.calc_pos -= 1;
                }
            },
            Key::Right => {
                if usize::from(calc.calc_pos) < calc.calc.len() {
                    calc.calc_pos += 1;
                }
            },
            Key::Up => {
                if calc.selected_calc > 0 {
                    calc.selected_calc -= 1;
                }
            },
            Key::Down => {
                if calc.selected_calc < history_items {
                    calc.selected_calc += 1;
                }
            }
            Key::Char('\n') => perform_calculation(&mut calc),
            //x => println!("{:?}", x)
            _ => ()
        }

        write!(stdout, "{}{}{}\n",
            // clear the screen,
            termion::clear::All,
            // go to top left corner
            termion::cursor::Goto(1,1),
            // print the currently-being-typed calculation
            &calc.calc).unwrap();

        // print the last 10 previous calcs in backwards order
        // TODO: truncate vector to only keep at most the last 100? 1000? calcs
        let mut line = 1;
        for (calc, output) in calc.prev_calcs.iter().rev().take(history_items.into()) {
            line += 1;
            let formatted = format_prev_calculation(&output);
            write!(stdout, "{}{} = {}\n",
                termion::cursor::Goto(1,line),
                calc,
                formatted).unwrap();
        }

        write!(stdout, "{}",
            // go to end of currently-being-typed calculation
            termion::cursor::Goto(calc.calc_pos+1,1)).unwrap();

        match &key {
            //x => println!("{:?}", x)
            _ => ()
        }

        stdout.flush().unwrap();
    }
}

fn append_key_to_calc(calc: &mut Calculator, k: &CalcKey) {
    // calc: &mut String, pos: &mut u16,
    match k {
        CalcKey::Key(x) => {
            calc.calc.insert((calc.calc_pos).into(), *x);
            calc.calc_pos += 1;
        },
        CalcKey::Delete => {
            if calc.calc_pos == 0 {
                return;
            }
            let mut delete_pos: usize = (calc.calc_pos).into();
            delete_pos -= 1;
            calc.calc.remove(delete_pos);
            if calc.calc_pos > 0 {
                calc.calc_pos -= 1;
            }
        },
    }
}

fn perform_calculation(calc: &mut Calculator) {
    let calc_copy = calc.calc.clone();
    let calc_equals = match eval(&calc.calc) {
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
    calc.prev_calcs.push((calc_copy, calc_equals));
    calc.calc.clear();
    calc.calc_pos = 0;
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
/*
fn format_prev_calculations(&mut self) -> String {
    let mut formatted = String::from("");
    for (calc, output) in &self.prev_calculations {
        if !formatted.is_empty() {
            formatted.push('\n')
        }
        formatted.push_str(calc);
        formatted.push_str(" = ");
        let formatted_output = match &output {
            CalcResult::Float(value) => value.to_string(),
            CalcResult::Integer(value) => value.to_string(),
            CalcResult::Error(string) => String::from(string)
        };
        formatted.push_str(&formatted_output);
    }
    return formatted;
}
*/
