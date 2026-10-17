use thiserror::Error;

pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
    Del { key: String },
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("InvalidCommandName : {0}")]
    InvalidCommandName(String),

    #[error("InvalidArguments : {0}")]
    InvalidArguments(String),
}

impl Command {
    pub fn parse(raw_str: String) -> Result<Self, CommandError> {
        // separate parts of raw_string command
        let command_parts: Vec<&str> = raw_str.split(' ').collect();

        match command_parts[0] {
            "set" => {
                if command_parts.len() != 3 {
                    Err(CommandError::InvalidArguments(String::from(
                        "'set' command takes 2 arguments (key & value)",
                    )))
                } else {
                    Ok(Self::Set {
                        key: command_parts[1].to_string(),
                        value: command_parts[2].to_string(),
                    })
                }
            }

            "get" => {
                if command_parts.len() != 2 {
                    Err(CommandError::InvalidArguments(String::from(
                        "'get' command takes 1 argument (key)",
                    )))
                } else {
                    Ok(Self::Get {
                        key: command_parts[1].to_string(),
                    })
                }
            }

            "del" => {
                if command_parts.len() != 2 {
                    Err(CommandError::InvalidArguments(String::from(
                        "'del' command takes 1 argument (key)",
                    )))
                } else {
                    Ok(Self::Del {
                        key: command_parts[1].to_string(),
                    })
                }
            }

            _ => Err(CommandError::InvalidCommandName(String::from(
                "Your command should start with set, get, or del.",
            ))),
        }
    }
}

pub enum Answer {
    Ok,
    Found { value: String },
    NotFound,
}

impl Answer {
    pub fn format(&self) -> String {
        match self {
            Self::Ok => String::from("change saved"),
            Self::Found { value } => value.clone(),
            Self::NotFound => String::from("given key does not exist"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_set() {
        assert!(Command::parse(String::from("set lilian cool")).is_ok());
    }

    #[test]
    fn invalid_set() {
        assert!(matches!(
            Command::parse(String::from("set invalid")),
            Err(CommandError::InvalidArguments(_))
        ));
    }

    #[test]
    fn valid_get() {
        assert!(Command::parse(String::from("get lilian")).is_ok());
    }

    #[test]
    fn invalid_get() {
        assert!(matches!(
            Command::parse(String::from("get invalid args")),
            Err(CommandError::InvalidArguments(_))
        ));
    }

    #[test]
    fn valid_del() {
        assert!(Command::parse(String::from("del arg")).is_ok());
    }

    #[test]
    fn invalid_del() {
        assert!(matches!(
            Command::parse(String::from("del too many args")),
            Err(CommandError::InvalidArguments(_))
        ));
    }

    #[test]
    fn invalid_command() {
        assert!(matches!(
            Command::parse(String::from("not a command at all")),
            Err(CommandError::InvalidCommandName(_))
        ));
    }

    #[test]
    fn ok_answer() {
        assert_eq!(Answer::Ok.format(), String::from("change saved"));
    }

    #[test]
    fn found_answer() {
        assert_eq!(
            Answer::Found {
                value: String::from("test_value")
            }
            .format(),
            String::from("test_value")
        );
    }

    #[test]
    fn not_found_answer() {
        assert_eq!(
            Answer::NotFound.format(),
            String::from("given key does not exist")
        );
    }
}
