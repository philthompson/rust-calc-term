use std::io::{stdin, stdout, Write};
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use eval::{eval, Value};
use rust_calc_term::tree::NodeIndex;
use rust_calc_term::tree::Tree;
use rust_calc_term::tree::TreeNode;
use rust_calc_term::tree::PostOrderIter;
use rust_calc_term::tree::ChildSide;

// TODO: fix precision of decimals somehow:
//     5.1 * 3 = 15.299999999999999
// the perhaps more egregious:
//     0.1 + 0.2 = 0.30000000000000004
//     was fixed by rounding results with format!({:.12})

enum CalcKey {
    Key(char),
    Delete
}

enum CalcResult {
    Float(f64),
    Integer(i64),
    Error(String)
}

// different types of tokens chars can belong to
#[derive(Clone,Copy,PartialEq)]
enum CalcJumpToken {
    Digit,
    Dot,
    Space,
    Operator,
    Paren
}

impl CalcJumpToken {
    const TOKEN_CHARS: [(CalcJumpToken, &'static str); 5] = [
        (CalcJumpToken::Digit, "0123456789"),
        (CalcJumpToken::Dot, "."),
        (CalcJumpToken::Space, " "),
        (CalcJumpToken::Operator, "+-*/"),
        (CalcJumpToken::Paren, "()"),
    ];

    fn get_token_matching_char(c: char) -> Option<CalcJumpToken> {
        for (t, s) in CalcJumpToken::TOKEN_CHARS.iter() {
            if s.contains(c) {
                return Some(*t);
            }
        }
        return None;
    }
}

#[derive(Clone,Copy,Debug,PartialEq)]
enum CalcParseToken {
    Value,
    Operator,
    OpenParen,
    CloseParen
}

impl CalcParseToken {
    const OPERATORS: &'static str = "+-*/";

    const TOKEN_CHARS: [(CalcParseToken, &'static str); 4] = [
        (CalcParseToken::Value, ".0123456789"),
        (CalcParseToken::Operator, CalcParseToken::OPERATORS),
        (CalcParseToken::OpenParen, "("),
        (CalcParseToken::CloseParen, ")")
    ];

    fn get_token_matching_char(c: char) -> Option<CalcParseToken> {
        for (t, s) in CalcParseToken::TOKEN_CHARS.iter() {
            if s.contains(c) {
                return Some(*t);
            }
        }
        return None;
    }

    fn get_token_matching_str(s: &str) -> Option<CalcParseToken> {
        if s == "(" {
            return Some(CalcParseToken::OpenParen);
        }
        if s == ")" {
            return Some(CalcParseToken::CloseParen);
        }
        if s.contains('(') || s.contains(')') {
            return None;
        }
        if s.len() == 1 && CalcParseToken::OPERATORS.contains(s) {
            return Some(CalcParseToken::Operator);
        }
        if (
                s.starts_with("-") || s.starts_with(".") ||
                s.starts_with('0') || s.starts_with('1') || s.starts_with('2') ||
                s.starts_with('3') || s.starts_with('4') || s.starts_with('5') ||
                s.starts_with('6') || s.starts_with('7') || s.starts_with('8') ||
                s.starts_with('9')
            ) &&
            (
                s.matches('.').count() == 0 ||
                (s.matches('.').count() == 1 && !s.ends_with("."))
            ) &&
            (
                s.matches('-').count() == 0 ||
                (s.matches('-').count() == 1 && s.starts_with("-"))
            ) {
            return Some(CalcParseToken::Value);
        }

        return None;
    }
}

struct CalcEvalItem {
    token_type: CalcParseToken,
    string_value: String,
}

impl CalcEvalItem {
    fn new(token_type: CalcParseToken, string_value: &str) -> CalcEvalItem {
        CalcEvalItem {
            token_type,
            string_value: String::from(string_value)
        }
    }
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
        "{}{}\
Type an expression, like \"355/113\" or \"(9+8)/(7+6)\" and hit return!\n\r\
previous calculations: [←↑↓→: select] [space: use selected] [a/z: show fewer/more prevs]\n\r\
editing: [q/r: move to beg/end] [w/e: jump left/right to item edge]\n\r\
other: [h: hide help] [ctrl+q: quit]{}{}",
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
            Key::Char('z') => {
                if history_items < 100 && usize::from(history_items) < calc.prev_calcs.len() {
                    history_items += 1;
                }
            },
            Key::Char('a') => {
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
                calc.move_cursor_left();
            },
            Key::Right => {
                calc.move_cursor_right();
            },
            Key::Char('q') => calc.move_cursor_home(),
            Key::Char('r') => calc.move_cursor_end(),
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
            Key::Char('w') => calc.move_cursor_left_token(),
            Key::Char('e') => calc.move_cursor_right_token(),
            Key::Char('p') => { Calculator::parse_calc_to_tokens(&calc.calc); },
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
            CalcResult::Float(value) => {
                String::from(
                    format!("{:.12}", value.to_string())
                        .trim_end_matches('0')
                )
            }
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

    fn move_cursor_left(&mut self) {
       if self.selected_calc == 0 {
            if self.calc_pos > 0 {
                self.calc_pos -= 1;
            }
        } else {
            self.selected_equals = false;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.selected_calc == 0 {
            if usize::from(self.calc_pos) < self.calc.len() {
                self.calc_pos += 1;
            }
        } else {
            self.selected_equals = true;
        }
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

    // get the type of token the cursor is at, then move left
    //   until the type of token changes
    // if there are any errors accessing the character at a position
    //   or matching things we can just return and not move the cursor
    fn move_cursor_left_token(&mut self) {
        if self.calc_pos as usize == self.calc.len() {
            self.move_cursor_left();
        }
        let start_token = match self.get_token_type_at_pos(self.calc_pos) {
            Some(t) => t,
            None => { return; }
        };
        let mut have_moved = false;
        while self.calc_pos > 0 {
            let pos_token = match self.get_token_type_at_pos(self.calc_pos - 1) {
                Some(t) => t,
                None => { return; }
            };
            if start_token != pos_token {
                if !have_moved {
                    self.move_cursor_left();
                }
                return;
            }
            have_moved = true;
            self.move_cursor_left();
        }
    }

    fn move_cursor_right_token(&mut self) {
        let start_token = match self.get_token_type_at_pos(self.calc_pos) {
            Some(t) => t,
            None => { return; }
        };
        let mut have_moved = false;
        while (self.calc_pos as usize) < self.calc.len() {
            let pos_token = match self.get_token_type_at_pos(self.calc_pos + 1) {
                Some(t) => t,
                None => { return; }
            };
            if start_token != pos_token {
                if !have_moved {
                    self.move_cursor_right();
                }
                return;
            }
            have_moved = true;
            self.move_cursor_right();
        }
    }

    fn get_token_type_at_pos(&mut self, pos: u16) -> Option<CalcJumpToken> {
        // to get Nth char, first skip N chars
        let pos_char = match self.calc.chars().skip(pos as usize).next() {
            Some(c) => c,
            None => { return None; }
        };
        return CalcJumpToken::get_token_matching_char(pos_char);
    }

    fn parse_calc_to_tokens(calc: &str) -> Vec<String> {
        let calc_no_space = calc.replace(" ", "");
        if calc_no_space.len() == 0 {
            return vec![];
        }
        let mut first_char = true;
        let mut tokens: Vec<String> = vec![];
        // initial token type is Operator to allow the calculation to start
        //   with a negative number
        let mut last_token_type = CalcParseToken::Operator;
        let mut token = String::from("");
        for c in calc_no_space.chars() {
            // TODO: return Err somehow, like any other parsing error?
            // perhaps the grammar, in terms of which tokens are allowed where,
            //   should just be handled elsewhere, and since we don't allow
            //   invalid chars to be typed in the first place, we can just
            //   panic here if we see an invalid char
            let token_type =
               CalcParseToken::get_token_matching_char(c).expect("Unparseable calculation");
            if first_char {
                first_char = false;
                if c == '-' {
                    last_token_type = CalcParseToken::Value;
                } else {
                    last_token_type = token_type;
                }
            } else if c == '-' &&
                    (last_token_type == CalcParseToken::Operator || last_token_type == CalcParseToken::OpenParen) {
                if !token.is_empty() {
                    tokens.push(token.clone());
                }
                token.clear();
                last_token_type = CalcParseToken::Value;
            // start a new token if token types are different, or if it's any
            //   type aside from value (digits can repeat, but parens and operators cannot)
            } else if last_token_type != token_type || token_type != CalcParseToken::Value {
                if !token.is_empty() {
                    tokens.push(token.clone());
                }
                token.clear();
                last_token_type = token_type;
            }
            token.push(c);
        }
        tokens.push(token.clone());
        return tokens;
    }

    fn build_calc_eval_tree(calc: &str) -> Result<Tree<CalcEvalItem>, String> {
        let mut tree = Tree::<CalcEvalItem>::new();

        let tokens = Calculator::parse_calc_to_tokens(calc);

        let mut cursor: Option<NodeIndex> = None;

        for token in tokens.iter() {
            let token_type = CalcParseToken::get_token_matching_str(token);
            if token_type.is_none() {
                return Err(format!("unknown token: [{}]", token));
            }
            let token_type = token_type.unwrap();
            match token_type {
                CalcParseToken::Value => {
                    let val_node_idx = Some(tree.add_node(TreeNode::new(
                        CalcEvalItem::new(token_type, token))));
                    match cursor {
                        Some(c) => {
                            let cursor_node = match tree.node_at_mut(c) {
                                Some(n) => n,
                                None => { return Err("no node exists at cursor index location".to_string()); }
                            };
                            match cursor_node.value.token_type {
                                CalcParseToken::Value => { return Err("cannot have two consecutive values: expected an operation or open paren".to_string()); },
                                CalcParseToken::Operator => {
                                    if !cursor_node.has_left() {
                                        return Err("cursor is an operator without a left hand side value".to_string());
                                    }
                                    if tree.set_node_child(c, val_node_idx, ChildSide::Right).is_err() {
                                        return Err("unable to set value as operator operand".to_string());
                                    }
                                    // for consistency, always move the cursor to the node that was
                                    //   last added to the tree
                                    cursor = val_node_idx;
                                },
                                CalcParseToken::OpenParen => {
                                    // when cursor is "(" there should be no way it has any children,
                                    //   since when going back up the tree after a ")" the cursor will
                                    //   be at a ")" not a "(" -- therefore checking for a left child is
                                    //   probably not necessary
                                    if cursor_node.has_left() {
                                        return Err("cursor is an open paren that already has a child value".to_string());
                                    }
                                    if tree.set_node_child(c, val_node_idx, ChildSide::Left).is_err() {
                                        return Err("unable to set value as child of an open paren".to_string());
                                    }
                                    cursor = val_node_idx;
                                },
                                CalcParseToken::CloseParen => {
                                    return Err("expected an operator, not a value, since the cursor was at a close paren".to_string());
                                }
                            }
                        },
                        None => {
                            cursor = val_node_idx;
                            tree.set_root(val_node_idx);
                        }
                    }
                },
                CalcParseToken::Operator => {
                    let op_node_idx = Some(tree.add_node(TreeNode::new(
                        CalcEvalItem::new(token_type, token))));
                    if cursor.is_none() || !tree.has_root() {
                        return Err("the first token cannot be an operator".to_string());
                    }
                    let cursor_loc = cursor.unwrap();
                    match token.as_str() {
                        "*" | "/" => {
                            let cursor_node = match tree.node_at(cursor_loc) {
                                Some(n) => n,
                                None => { return Err("no node exists at cursor index location".to_string()); }
                            };
                            match cursor_node.value.token_type {
                                // the "*/" operators have precedence, so they are inserted at the
                                //   bottom of the tree (where the cursor is) so that they are
                                //   processed first ("+-" operators cannot be handled so easily)
                                // if cursor is at a value or ")", insert the new operation node
                                //   between the cursor and its parent (and handle special case
                                //   where cursor is at root)
                                CalcParseToken::Value | CalcParseToken::CloseParen => {
                                    if tree.matches_root(cursor_loc) {
                                        let result = tree.replace_root_with_node(
                                            op_node_idx.unwrap(), ChildSide::Left);
                                        if result.is_err() {
                                            return Err("unable to replace root with operator node".to_string());
                                        }
                                        cursor = op_node_idx;
                                    } else {
                                        let result = tree.insert_node_above_node(cursor_loc, op_node_idx.unwrap(), ChildSide::Left);
                                        if result.is_err() {
                                            return Err("unable to insert new operator node in place of an existing value node".to_string());
                                        }
                                        cursor = op_node_idx;
                                    }
                                },
                                CalcParseToken::Operator => {
                                    // now that the cursor is always set to the last inserted node
                                    //   (whether it be a value, operator, or paren), it must be an
                                    //   error if the cursor is at an operator when another operator
                                    //   is the next token
                                    return Err("cannot have two consecutive operators: expected a value or open paren".to_string());
                                },
                                CalcParseToken::OpenParen => {
                                    // now that the cursor is always set to the last inserted node
                                    //   (whether it be a value, operator, or paren), it must be an
                                    //   error if the cursor is at an open paren when another operator
                                    //   is the next token
                                    return Err("cannot have an operator following an open paren: expected a value or open paren".to_string());
                                }
                            }
                        },
                        "+" | "-" => {
                            let cursor_node = match tree.node_at(cursor_loc) {
                                Some(n) => n,
                                None => { return Err("no node exists at cursor index location".to_string()); }
                            };
                            match cursor_node.value.token_type {
                                CalcParseToken::Value | CalcParseToken::CloseParen => {
                                    // go back up the tree, parent to parent, until (whichever is first):
                                    //   - root node, or
                                    //   - open paren (e.g. if closed paren that is not root, go up)
                                    // after stopping:
                                    //   - if root node, make new "+-" op the root, or
                                    //   - if open paren, insert new "+-" below the paren
                                    let mut insert_loc = cursor_loc;
                                    while !tree.matches_root(insert_loc) {
                                        let insert_loc_parent_loc = tree.get_node_parent(insert_loc);
                                        if insert_loc_parent_loc.is_none() {
                                            return Err("a node that isn't the root has no parent".to_string());
                                        }
                                        let insert_loc_parent_loc = insert_loc_parent_loc.unwrap();
                                        let insert_loc_parent = match tree.node_at(insert_loc_parent_loc) {
                                            Some(n) => n,
                                            None => { return Err("no node exists at a node's parent's location".to_string()); }
                                        };
                                        match insert_loc_parent.value.token_type {
                                            // if the parent is an "(" then stop here
                                            CalcParseToken::OpenParen => {
                                                break;
                                            },
                                            _ => ()
                                        }
                                        // if the node is not root and its parent is not a "(", keep going up
                                        insert_loc = insert_loc_parent_loc;
                                    }
                                    // this if/else can almost exactly be copied from above "*/" operator
                                    //   code for when cursor is at a value node
                                    if tree.matches_root(insert_loc) {
                                        let result = tree.replace_root_with_node(
                                            op_node_idx.unwrap(), ChildSide::Left);
                                        if result.is_err() {
                                            return Err("unable to replace root with operator node".to_string());
                                        }
                                        cursor = op_node_idx;
                                    } else {
                                        let result = tree.insert_node_above_node(insert_loc, op_node_idx.unwrap(), ChildSide::Left);
                                        if result.is_err() {
                                            return Err("unable to insert new operator node in place of an existing value node".to_string());
                                        }
                                        cursor = op_node_idx;
                                    }
                                },
                                CalcParseToken::Operator => {
                                    // now that the cursor is always set to the last inserted node
                                    //   (whether it be a value, operator, or paren), it must be an
                                    //   error if the cursor is at an operator when another operator
                                    //   is the next token
                                    return Err("cannot have two consecutive operators: expected a value or open paren".to_string());
                                },
                                CalcParseToken::OpenParen => {
                                    // now that the cursor is always set to the last inserted node
                                    //   (whether it be a value, operator, or paren), it must be an
                                    //   error if the cursor is at an open paren when another operator
                                    //   is the next token
                                    return Err("cannot have an operator following an open paren: expected a value or open paren".to_string());
                                }
                            }
                        },
                        _ => { return Err(format!("unknown operator [{}]", token)); }
                    }
                },
                CalcParseToken::OpenParen => {
                    let paren_node_idx = Some(tree.add_node(TreeNode::new(
                        CalcEvalItem::new(token_type, token))));
                    match cursor {
                        Some(c) => {
                            // if the cursor is at a value node, we have an error
                            // if the cursor is at an op node, and op left is unset, error
                            // if the cursor is at an op node with left set, set op right to this
                            let cursor_node = match tree.node_at_mut(c) {
                                Some(n) => n,
                                None => { return Err("improve this error message".to_string()); }
                            };
                            match cursor_node.value.token_type {
                                CalcParseToken::Value => { return Err("expected an operator or close paren, not an open paren, since the last token was a value".to_string()); },
                                CalcParseToken::Operator => {
                                    if !cursor_node.has_left() {
                                        return Err("the previous token was an operator, which should already have a left-hand side operand".to_string());
                                    }
                                    if tree.set_node_child(c, paren_node_idx, ChildSide::Right).is_err() {
                                        return Err("unable to set open paren as right child of an operator".to_string());
                                    }
                                    cursor = paren_node_idx;
                                },
                                CalcParseToken::OpenParen => {
                                    // if the cursor is already an open paren, it must not
                                    //   already have a left child (since this new token is
                                    //   another paren)
                                    if cursor_node.has_left() {
                                        return Err("the previous token was an open paren, which should not already have any child nodes".to_string());
                                    }
                                    if tree.set_node_child(c, paren_node_idx, ChildSide::Left).is_err() {
                                        return Err("unable to set open paren as left child of the previous open paren".to_string());
                                    }
                                    cursor = paren_node_idx;
                                },
                                CalcParseToken::CloseParen => {
                                    return Err("a close paren cannot immediately be followed by an open paren".to_string());
                                }
                            }
                        },
                        None => {
                            cursor = paren_node_idx;
                            tree.set_root(paren_node_idx);
                        }
                    }
                },
                CalcParseToken::CloseParen => {
                    // first stab at this:
                    // go back up tree until first "(", then check that:
                    //   - all descendant operators have 2 children, and
                    //   - all descendant parens are ")", not "("

                    // can we just go up parent-to-parent?  or must we re-descend down the
                    //   tree with a traversal?

                    if cursor.is_none() || !tree.has_root() {
                        return Err("the first token cannot be a closed paren".to_string());
                    }
                    let cursor_loc = cursor.unwrap();

                    // this is the "go up then traverse down" approach
                    // let mut open_paren_loc = cursor_loc;
                    // while !tree.matches_root(open_paren_loc) {
                    //     let open_paren_loc_parent_loc = tree.get_node_parent(open_paren_loc);
                    //     if open_paren_loc_parent_loc.is_none() {
                    //         return Err("a node that isn't the root has no parent");
                    //     }
                    //     open_paren_loc = open_paren_loc_parent_loc.unwrap();
                    //     let open_paren_node = match tree.node_at(open_paren_loc) {
                    //         Some(n) => n,
                    //         None => { return Err("no node exists at a node's parent's location"); }
                    //     };
                    //     match open_paren_node.value.token_type {
                    //         // if the parent is an "(" then stop here
                    //         CalcParseToken::OpenParen => {
                    //             break;
                    //         },
                    //         _ => ()
                    //     }
                    // }

                    // let open_paren_node = match tree.node_at(open_paren_loc) {
                    //     Some(n) => n,
                    //     None => { return Err("no node exists at found nearest open paren"); }
                    // };
                    // match open_paren_node.value.token_type {
                    //     CalcParseToken::OpenParen => {
                    //         // do traversal starting at open_paren_loc
                    //     },
                    //     _ => {
                    //         // handle the case where we hit the root before finding an open paren
                    //         return Err("no corresponding open paren token found for the new close paren");
                    //     }
                    // }

                    // this is the "just go up" approach
                    let mut reverse_cursor_loc = cursor_loc;
                    while !tree.matches_root(reverse_cursor_loc) {
                        let parent_loc = tree.get_node_parent(reverse_cursor_loc);
                        if parent_loc.is_none() {
                            return Err("a node that isn't the root has no parent".to_string());
                        }
                        reverse_cursor_loc = parent_loc.unwrap();
                        let open_paren_node = match tree.node_at(reverse_cursor_loc) {
                            Some(n) => n,
                            None => { return Err("no node exists at a node's parent's location".to_string()); }
                        };
                        match open_paren_node.value.token_type {
                            // if the parent is an "(" then stop here
                            CalcParseToken::OpenParen => {
                                break;
                            },
                            CalcParseToken::Operator => {
                                if !open_paren_node.has_left() || !open_paren_node.has_right() {
                                    return Err("close paren not expected because previous operator node does not have two operands".to_string());
                                }
                            },
                            CalcParseToken::Value | CalcParseToken::CloseParen => ()
                        }
                    }
                    let open_paren_node = match tree.node_at_mut(reverse_cursor_loc) {
                        Some(n) => n,
                        None => { return Err("no node exists at found nearest open paren".to_string()); }
                    };
                    match open_paren_node.value.token_type {
                        CalcParseToken::OpenParen => {
                            // change open paren to a close paren
                            open_paren_node.value = CalcEvalItem::new(token_type, token);
                            cursor = Some(reverse_cursor_loc);
                        },
                        _ => {
                            // handle the case where we hit the root before finding an open paren
                            return Err("no corresponding open paren token found for the new close paren".to_string());
                        }
                    }
                }
            }
        }
        return Ok(tree);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_empty() {
        assert_eq!(Vec::<String>::new(), Calculator::parse_calc_to_tokens(""));
    }

    #[test]
    fn tokenize_only_space() {
        assert_eq!(Vec::<String>::new(), Calculator::parse_calc_to_tokens("  "));
    }

    #[test]
    fn tokenize_operator() {
        assert_eq!(vec!["1", "+", "2"], Calculator::parse_calc_to_tokens("1 + 2"));
    }

    #[test]
    fn tokenize_multichar_value() {
        assert_eq!(vec!["123"], Calculator::parse_calc_to_tokens("123"));
    }

    #[test]
    fn tokenize_decimal() {
        assert_eq!(vec!["123.456", "*", "0.789"], Calculator::parse_calc_to_tokens("123.456 * 0.789"));
    }

    #[test]
    fn tokenize_negative_only() {
        assert_eq!(vec!["-123.456"], Calculator::parse_calc_to_tokens("-123.456"));
    }

    #[test]
    fn tokenize_negative_value() {
        assert_eq!(vec!["1", "-", "-1"], Calculator::parse_calc_to_tokens("1 - -1"));
    }

    #[test]
    fn tokenize_paren_negative_value() {
        assert_eq!(vec!["1", "-", "(", "-1", ")"], Calculator::parse_calc_to_tokens("1 - (-1)"));
    }

    #[test]
    fn tokenize_paren_then_subtract() {
        assert_eq!(vec!["(", "1", ")", "-", "1"], Calculator::parse_calc_to_tokens("(1)-1"));
    }

    #[test]
    fn tokenize_nested_paren() {
        assert_eq!(vec!["(","(","1","*","2",")","*","3",")"], Calculator::parse_calc_to_tokens("((1*2)*3)"));
    }

    #[test]
    fn tokenize_not_starting_nested_paren() {
        assert_eq!(vec!["1","+","(","(","2",")",")"], Calculator::parse_calc_to_tokens("1+((2))"));
    }

    #[test]
    fn tokenize_double_plus() {
        assert_eq!(vec!["1","+","+","2"], Calculator::parse_calc_to_tokens("1++2"));
    }

    #[test]
    fn tokenize_double_times() {
        assert_eq!(vec!["1","*","*","2"], Calculator::parse_calc_to_tokens("1**2"));
    }

    #[test]
    fn tokenize_double_divide() {
        assert_eq!(vec!["1","/","/","2"], Calculator::parse_calc_to_tokens("1//2"));
    }

    #[test]
    fn tokenize_minus_plus() {
        assert_eq!(vec!["1","-","+","2"], Calculator::parse_calc_to_tokens("1-+2"));
    }

    #[test]
    fn tokenize_multiply_divide() {
        assert_eq!(vec!["1","*","/","2"], Calculator::parse_calc_to_tokens("1*/2"));
    }

    #[test]
    fn tree_add_minimal() {
        let mut tree = Tree::<&str>::new();

        let index_loc = tree.add_node(TreeNode::new("a"));

        assert_eq!(0, index_loc);
    }

    #[test]
    fn tree_add_a_few() {
        let mut tree = Tree::<&str>::new();

        let a = tree.add_node(TreeNode::new("a"));
        let b = tree.add_node(TreeNode::new("b"));
        let c = tree.add_node_with_children(TreeNode::new("c"), Some(a), Some(b));

        assert_eq!(2, c);
    }

    #[test]
    fn tree_postorder_empty() {
        let tree = Tree::<&str>::new();

        let mut output = String::from("");
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push_str(&node.value.to_string());
        }
        assert_eq!("", output);
    }

    #[test]
    fn tree_postorder_index_minimal() {
        let mut tree = Tree::<&str>::new();

        let a = tree.add_node(TreeNode::new("a"));
        tree.set_root(Some(a));

        let mut postorder = PostOrderIter::new(&tree);
        let index_loc = postorder.next().expect("Node does not exist at given index");

        assert_eq!(0, index_loc);
    }

    #[test]
    fn tree_postorder_minimal() {
        let mut tree = Tree::<&str>::new();

        let a = tree.add_node(TreeNode::new("a"));
        tree.set_root(Some(a));

        let mut output = String::from("");
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push_str(&node.value.to_string());
        }
        assert_eq!("a", output);
    }

    // example tree from https://en.wikipedia.org/wiki/Tree_traversal#Post-order_(LRN)
    #[test]
    fn tree_postorder_complex() {
        let mut tree = Tree::<&str>::new();

        let a = tree.add_node(TreeNode::new("a"));
        let c = tree.add_node(TreeNode::new("c"));
        let e = tree.add_node(TreeNode::new("e"));
        let d = tree.add_node_with_children(TreeNode::new("d"), Some(c), Some(e));
        let b = tree.add_node_with_children(TreeNode::new("b"), Some(a), Some(d));
        let h = tree.add_node(TreeNode::new("h"));
        let i = tree.add_node_with_children(TreeNode::new("i"), Some(h), None);
        let g = tree.add_node_with_children(TreeNode::new("g"), None, Some(i));
        let f = tree.add_node_with_children(TreeNode::new("f"), Some(b), Some(g));
        tree.set_root(Some(f));

        let mut output = String::from("");
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push_str(&node.value.to_string());
        }
        assert_eq!("acedbhigf", output);
    }

    #[test]
    fn get_char_token_digit() {
        for c in "0123456789".chars() {
            assert_eq!(CalcParseToken::Value, CalcParseToken::get_token_matching_char(c).unwrap());
        }
    }

    #[test]
    fn get_char_token_operator() {
        for c in "+-*/".chars() {
            assert_eq!(CalcParseToken::Operator, CalcParseToken::get_token_matching_char(c).unwrap());
        }
    }

    #[test]
    fn get_char_token_open_paren() {
        assert_eq!(CalcParseToken::OpenParen, CalcParseToken::get_token_matching_char('(').unwrap());
    }

    #[test]
    fn get_char_token_close_paren() {
        assert_eq!(CalcParseToken::CloseParen, CalcParseToken::get_token_matching_char(')').unwrap());
    }

    #[test]
    fn get_char_token_invalid() {
        for c in "abc!$,=".chars() {
            assert_eq!(None, CalcParseToken::get_token_matching_char(c));
        }
    }

    #[test]
    fn get_str_token_digit() {
        for c in "0123456789".chars() {
            assert_eq!(CalcParseToken::Value, CalcParseToken::get_token_matching_str(&String::from(c)).unwrap());
        }
    }

    #[test]
    fn get_str_token_operator() {
        for c in "+-*/".chars() {
            assert_eq!(CalcParseToken::Operator, CalcParseToken::get_token_matching_str(&String::from(c)).unwrap());
        }
    }

    #[test]
    fn get_str_token_open_paren() {
        assert_eq!(CalcParseToken::OpenParen, CalcParseToken::get_token_matching_str("(").unwrap());
    }

    #[test]
    fn get_str_token_close_paren() {
        assert_eq!(CalcParseToken::CloseParen, CalcParseToken::get_token_matching_str(")").unwrap());
    }

    #[test]
    fn get_str_token_integer() {
        assert_eq!(CalcParseToken::Value, CalcParseToken::get_token_matching_str("123").unwrap());
    }

    #[test]
    fn get_str_token_negative_integer() {
        assert_eq!(CalcParseToken::Value, CalcParseToken::get_token_matching_str("-456").unwrap());
    }

    #[test]
    fn get_str_token_decimal() {
        assert_eq!(CalcParseToken::Value, CalcParseToken::get_token_matching_str("3.14").unwrap());
    }

    #[test]
    fn get_str_token_negative_decimal() {
        assert_eq!(CalcParseToken::Value, CalcParseToken::get_token_matching_str("-3.14").unwrap());
    }

    #[test]
    fn get_str_token_zero_decimal() {
        assert_eq!(CalcParseToken::Value, CalcParseToken::get_token_matching_str("0.42").unwrap());
    }

    #[test]
    fn get_str_token_negative_zero_decimal() {
        assert_eq!(CalcParseToken::Value, CalcParseToken::get_token_matching_str("-0.42").unwrap());
    }

    #[test]
    fn get_str_token_just_decimal() {
        assert_eq!(CalcParseToken::Value, CalcParseToken::get_token_matching_str(".25").unwrap());
    }

    #[test]
    fn get_str_token_contains_open_paren() {
        assert_eq!(None, CalcParseToken::get_token_matching_str("(10"));
    }

    #[test]
    fn get_str_token_contains_close_paren() {
        assert_eq!(None, CalcParseToken::get_token_matching_str("10)"));
    }

    #[test]
    fn get_str_token_negative_just_decimal() {
        assert_eq!(CalcParseToken::Value, CalcParseToken::get_token_matching_str("-.25").unwrap());
    }

    #[test]
    fn get_str_token_two_decimals() {
        assert_eq!(None, CalcParseToken::get_token_matching_str("5.2.5"));
    }

    #[test]
    fn get_str_token_negative_two_decimals() {
        assert_eq!(None, CalcParseToken::get_token_matching_str("-.2.5"));
    }

    #[test]
    fn get_str_token_two_negatives() {
        assert_eq!(None, CalcParseToken::get_token_matching_str("-2-5"));
    }

    #[test]
    fn build_tree_add() {
        let tree = Calculator::build_calc_eval_tree("123 + 456").unwrap();
        let mut output = Vec::<&str>::new();
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push(&node.value.string_value);
        }
        assert_eq!(vec!["123","456","+"], output);
    }

    #[test]
    fn build_tree_multiply() {
        let tree = Calculator::build_calc_eval_tree("123 * 456").unwrap();
        let mut output = Vec::<&str>::new();
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push(&node.value.string_value);
        }
        assert_eq!(vec!["123","456","*"], output);
    }

    #[test]
    fn build_tree_add_three() {
        let tree = Calculator::build_calc_eval_tree("123 + 456 + 789").unwrap();
        let mut output = Vec::<&str>::new();
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push(&node.value.string_value);
        }
        assert_eq!(vec!["123","456","+","789","+"], output);
    }

    #[test]
    fn build_tree_add_then_multiply() {
        let tree = Calculator::build_calc_eval_tree("123 + 456 * 789").unwrap();
        let mut output = Vec::<&str>::new();
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push(&node.value.string_value);
        }
        assert_eq!(vec!["123","456","789","*","+"], output);
    }

    #[test]
    fn build_tree_add_multiply_add() {
        let tree = Calculator::build_calc_eval_tree("1 + 2 * 3 + 4").unwrap();
        let mut output = Vec::<&str>::new();
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push(&node.value.string_value);
        }
        assert_eq!(vec!["1","2","3","*","+","4","+"], output);
    }

    #[test]
    fn build_tree_multiply_add_multiply() {
        let tree = Calculator::build_calc_eval_tree("1 * 2 + 3 * 4").unwrap();
        let mut output = Vec::<&str>::new();
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push(&node.value.string_value);
        }
        assert_eq!(vec!["1","2","*","3","4","*","+"], output);
    }

    #[test]
    fn build_tree_multiply_multiply_multiply() {
        let tree = Calculator::build_calc_eval_tree("1 * 2 * 3 * 4").unwrap();
        let mut output = Vec::<&str>::new();
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push(&node.value.string_value);
        }
        assert_eq!(vec!["1","2","3","4","*","*","*"], output);
    }

    #[test]
    fn build_tree_paren_add() {
        let tree = Calculator::build_calc_eval_tree("(1 + 2").unwrap();
        let mut output = Vec::<&str>::new();
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push(&node.value.string_value);
        }
        assert_eq!(vec!["1","2","+","("], output);
    }

    #[test]
    fn build_tree_two_parens_then_multiply() {
        let tree = Calculator::build_calc_eval_tree("(1+2) + (3+4)*5").unwrap();
        let mut output = Vec::<&str>::new();
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push(&node.value.string_value);
        }
        assert_eq!(vec!["1","2","+",")","3","4","+",")","5","*","+"], output);
    }

    #[test]
    fn build_tree_nested_parens() {
        let tree = Calculator::build_calc_eval_tree("((1+2)*(3+4))/5").unwrap();
        let mut output = Vec::<&str>::new();
        let mut postorder = PostOrderIter::new(&tree);
        while let Some(index) = postorder.next() {
            let node = tree.node_at(index).expect("Node does not exist at given index");
            output.push(&node.value.string_value);
        }
        assert_eq!(vec!["1","2","+",")","3","4","+",")","*",")","5","/"], output);
    }

    // add tets for invalid inputs for get_token_matching_str()

    // if needed, add tests for whitespace-removed calcs with negative numbers, like "1 - -.1" and "5 * -0.1"
}