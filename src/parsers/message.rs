use std::error::Error;
use std::fmt;

#[derive(PartialEq)]
#[derive(Debug)]
pub struct MessageParsingError {
    details: String
}

impl MessageParsingError {
    fn new(details: &str) -> Self {
        MessageParsingError { details: details.to_string() }
    }
}

impl fmt::Display for MessageParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for MessageParsingError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Debug)]
pub struct Message<'a> {
    tags: Option<&'a str>,
    prefix: Option<&'a str>,
    command: Option<&'a str>,
    params: Vec<&'a str>
}

impl<'a> Message<'a> {
    fn new() -> Self {
        Message {
            tags: None,
            prefix: None,
            command: None,
            params: vec![]
        }
    }

    fn add_param(&mut self, param: &'a str) {
        self.params.push(param);
    }
}

/// Returns a slice from the next non-whitespace character to the end of the
/// string. 
/// 
/// # Arguments
/// 
/// * `input` - the slice to process
fn consume_from_left<'a>(input: &str) -> (Option<&str>, Option<&str>) {
    if input.len() == 0 {
        return (None, None);
    }

    let chars = input.chars();

    for (idx, ch) in chars.enumerate() {
        match ch {
            ' ' => {},
            _ => {
                let trimmed = &input[idx..input.len()];
                let next_occurance = trimmed.find(' ');
                
                match next_occurance {
                    Some(pos) => {
                        let token = &trimmed[0..pos];
                        let remainder = &trimmed[pos..trimmed.len()];
                        return (Some(token), Some(remainder));
                    },
                    None => {
                        let token = &trimmed[0..trimmed.len()];
                        return (Some(token), None);
                    }
                };
            }
        }
    }

    (None, None)

}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Token<'a> {
    Tags(&'a str),
    Prefix(&'a str),
    Command(&'a str)
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Param<'a> {
    Middle(&'a str),
    Trailing(&'a str)
}

/// Returns the first token 
fn identify_token<'a>(
    token: Option<&'a str>,
    first_token: bool
) -> Result<Token<'a>, MessageParsingError> {
    match token {
        Some(seq) => {
            let first_char = seq.chars().next();

            match first_char {
                // if the first character is an @, this particular message is
                // tagged.
                Some('@') => {
                    if first_token {
                        Ok(Token::Tags(&seq[1..seq.len()]))
                    } else {
                        let desc = "message-tags are invalid here";
                        Err(MessageParsingError::new(desc))
                    }
                },
                // if the first character is a colon, it's the prefix of the
                // message.
                Some(':') => {
                    Ok(Token::Prefix(&seq[1..seq.len()]))
                },
                // otherwise, 
                _ => {
                    if !first_token {
                        Ok(Token::Command(seq))
                    } else {
                        let desc = "expected message-tags or prefix";
                        Err(MessageParsingError::new(desc))
                    }
                }
            }
        },
        None => {
            let err = MessageParsingError::new("");
            Err(err)
        }
    }
}

/// Returns the second token 
fn consume_params<'a>(
    current: Option<&'a str>
) -> Result<(Param<'a>, Option<&'a str>), MessageParsingError> {
    match current {
        Some(seq) => {
            let first_char = seq.chars().next();

            match first_char {
                // if the first character is a colon, we've got a trailing param
                Some(':') => {
                    let param = &seq[1..seq.len()];
                    Ok((Param::Trailing(param), None))
                },
                // otherwise,
                _ => {
                    let (param, remainder) = consume_from_left(seq);

                    match remainder {
                        Some(s) => {
                            Ok((Param::Middle(param.unwrap()), Some(s)))
                        },
                        None => {
                            Ok((Param::Middle(param.unwrap()), None))
                        }
                    }
                }
            }
        },
        None => {
            let desc = "expected data, got nothing";
            let err = MessageParsingError::new(desc);
            Err(err)
        }
    }
}

pub fn parse_line(line: &str) -> Result<Message, MessageParsingError> {
    let mut message = Message::new();
    // obtain the first token. if it begins with 
    let (raw_token, current) = consume_from_left(line);

    let parse_result = identify_token(raw_token, true);

    // the first token will either be a string of message-tags, or the prefix
    // of the message.
    match parse_result {
        Ok(token) => {
            match token {
                Token::Tags(t) => {
                    message.tags = Some(t);
                },
                Token::Prefix(pfx) => {
                    message.prefix = Some(pfx);
                },
                _ => {}
            }
        },
        Err(e) => {
            return Err(e);
        }
    }

    match current {
        None => {
            let desc = "unexpected end of message";
            let error = MessageParsingError::new(desc);
            return Err(error);
        }
        _ => {}
    }

    let (raw_token, mut current) = consume_from_left(current.unwrap());
    let parse_result = identify_token(raw_token, false);

    match parse_result {
        Ok(token) => {
            match token {
                Token::Prefix(pfx) => {
                    message.prefix = Some(pfx);
                },
                Token::Command(cmd) => {
                    message.command = Some(cmd);
                },
                _ => {}
            }
        },
        Err(e) => {
            return Err(e);
        }
    }

    if message.command == None {
        let (raw_token, params_string) = consume_from_left(current.unwrap());
        message.command = raw_token;
        current = params_string;
    }

    match current {
        None => {
            return Ok(message)
        },
        _ => {
            loop {
                let parse_result = consume_params(current);

                match parse_result {
                    Ok(param) => {
                        match param {
                            (Param::Middle(p), remaining_params) => {
                                current = remaining_params;
                                message.add_param(p);
                                continue;
                            },
                            (Param::Trailing(p), _) => {
                                message.add_param(p);
                                break;
                            }
                        }
                    },
                    Err(err) => {
                        return Err(err);
                    }
                }
            }

            return Ok(message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_new() {
        let message = Message::new();

        assert!(message.tags == None);
        assert!(message.prefix == None);
        assert!(message.command == None);
        assert!(message.params.len() == 0);
    }

    #[test]
    fn message_add_params() {
        let mut message = Message::new();

        message.add_param("param1");
        assert!(message.params.len() == 1);
        message.add_param("param2");
        assert!(message.params.len() == 2);
    }

    #[test]
    fn consume_left_empty() {
        let src = "";
        let (token, remainder) = consume_from_left(src);

        assert_eq!(token, None);
        assert_eq!(remainder, None);
    }

    #[test]
    fn consume_left_no_remainder() {
        let src = "no_remainder";
        let (token, remainder) = consume_from_left(src);

        assert_eq!(token, Some("no_remainder"));
        assert_eq!(remainder, None);
    }

    #[test]
    fn consume_left_single_space() {
        let src = " ";
        let (token, remainder) = consume_from_left(src);

        assert_eq!(token, None);
        assert_eq!(remainder, None);
    }

    #[test]
    fn consume_left_all_space() {
        let src = "       ";
        let (token, remainder) = consume_from_left(src);

        assert_eq!(token, None);
        assert_eq!(remainder, None);
    }

    #[test]
    fn consume_left_with_middle_space() {
        let src = "hello world";
        let (token, remainder) = consume_from_left(src);

        assert_eq!(token, Some("hello"));
        assert_eq!(remainder, Some(" world"));

        let (token, remainder) = consume_from_left(remainder.unwrap());

        assert_eq!(token, Some("world"));
        assert_eq!(remainder, None);
    }

    #[test]
    fn consume_left_with_initial_space() {
        let src = " hello world";
        let (token, remainder) = consume_from_left(src);

        assert_eq!(token, Some("hello"));
        assert_eq!(remainder, Some(" world"));

        let (token, remainder) = consume_from_left(remainder.unwrap());

        assert_eq!(token, Some("world"));
        assert_eq!(remainder, None);
    }

    #[test]
    fn consume_left_with_multi_middle_space() {
        let src = "hello    world";
        let (token, remainder) = consume_from_left(src);

        assert_eq!(token, Some("hello"));
        assert_eq!(remainder, Some("    world"));

        let (token, remainder) = consume_from_left(remainder.unwrap());

        assert_eq!(token, Some("world"));
        assert_eq!(remainder, None);
    }

    #[test]
    fn consume_left_with_multi_sporadic_space() {
        let src = "   lo rem   ipsum  dol       ar   ";

        let (token, remainder) = consume_from_left(src);
        assert_eq!(token, Some("lo"));
        assert_eq!(remainder, Some(" rem   ipsum  dol       ar   "));

        let (token, remainder) = consume_from_left(remainder.unwrap());
        assert_eq!(token, Some("rem"));
        assert_eq!(remainder, Some("   ipsum  dol       ar   "));

        let (token, remainder) = consume_from_left(remainder.unwrap());
        assert_eq!(token, Some("ipsum"));
        assert_eq!(remainder, Some("  dol       ar   "));

        let (token, remainder) = consume_from_left(remainder.unwrap());
        assert_eq!(token, Some("dol"));
        assert_eq!(remainder, Some("       ar   "));

        let (token, remainder) = consume_from_left(remainder.unwrap());
        assert_eq!(token, Some("ar"));
        assert_eq!(remainder, Some("   "));
    }

    #[test]
    fn identify_tag_token() {
        let raw_token = "@these=are;tags";
        let expected = "these=are;tags";

        let result = identify_token(Some(raw_token), true);
        assert_eq!(result, Ok(Token::Tags(expected)));
    }

    #[test]
    fn identify_prefix_first_token() {
        let raw_token = ":testprefix";
        let expected = "testprefix";

        let result = identify_token(Some(raw_token), true);
        assert_eq!(result, Ok(Token::Prefix(expected)));
    }

    #[test]
    fn identify_prefix_second_token() {
        let raw_token = ":testprefix";
        let expected = "testprefix";

        let result = identify_token(Some(raw_token), false);
        assert_eq!(result, Ok(Token::Prefix(expected)));
    }

    #[test]
    fn identify_command_token() {
        let token = "COMMAND";
        let expected = "COMMAND";

        let result = identify_token(Some(token), false);
        assert_eq!(result, Ok(Token::Command(expected)));
    }

    #[test]
    fn consume_param_middle_one() {
        let input = "#channel";
        let expected = Ok((
            Param::Middle(input),
            None
        ));

        let result = consume_params(Some(input));
        assert_eq!(result, expected);
    }

    #[test]
    fn consume_param_middle_two() {
        let input = "#channel something";
        let expected = Ok((
            Param::Middle(input),
            Some(" something")
        ));

        let result = consume_params(Some(input));
        assert_eq!(result, expected);
    }

    #[test]
    fn identify_param_trailing() {
        let token = "COMMAND";
        let expected = "COMMAND";

        let result = identify_token(Some(token), false);
        assert_eq!(result, Ok(Token::Command(expected)));
    }
}
