use clap::Parser;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Action {
    Send,
    Download(usize),
}

impl Default for Action {
    fn default() -> Self {
        Self::Send
    }
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "send" => Ok(Action::Send),
            _ if s.starts_with("download-") => {
                let number = s
                    .split('-')
                    .last()
                    .unwrap()
                    .parse::<usize>()
                    .map_err(|_| "Invalid number")?;
                Ok(Action::Download(number))
            }
            _ => Err(format!("{} is not a valid Action", s)),
        }
    }
}

impl ToString for Action {
    fn to_string(&self) -> String {
        match self {
            Action::Send => "send".to_string(),
            Action::Download(n) => n.to_string(),
        }
    }
}

#[derive(Parser, Debug, Default)]
#[clap(author = "Author Name", version, about)]
pub struct Argument {
    #[clap(short, long)]
    action: Action,

    #[clap(short, long, value_delimiter = ',')]
    file_names: Option<Vec<String>>,
}

impl Argument {
    pub fn action(&self) -> Action {
        self.action.clone()
    }

    pub fn file_names(&self) -> Vec<String> {
        self.file_names
            .clone()
            .expect("file names should not be absent")
    }

    // TODO(production): should add more validations and file sanitization
    fn validate_file_names(&self) -> Result<(), String> {
        for name in self
            .file_names
            .clone()
            .expect("file_names should not be absent")
            .iter()
        {
            if name.split(".").count() != 2 {
                return Err(String::from(
                    "file name should be in format 'file_name.file_type'",
                ));
            }
        }

        Ok(())
    }

    /// validate validates the Argument instance
    pub fn validate(&self) -> Result<(), String> {
        if let Action::Send = self.action {
            if self.file_names.is_none() {
                return Err(String::from(
                    "file names should be sent with the 'send' actions",
                ));
            }
            self.validate_file_names()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parsing_argument_works() {
        let file_names = vec![String::from("dummy1.txt"), String::from("dummy2.txt")];

        let args = Argument {
            action: Default::default(),
            file_names: Some(file_names.clone()),
        };

        assert_eq!(args.file_names, file_names);
        assert_eq!(args.action.to_string(), Action::Send.to_string());
    }
}
