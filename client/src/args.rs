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
                let number = s.split('-').last().unwrap().parse::<usize>().map_err(|_| "Invalid number")?;
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
            Action::Download(n) => n.to_string()
        }
    }
}

#[derive(Parser, Debug, Default)]
#[clap(author = "Author Name", version, about)]
/// A text compressor
pub struct Argument {
    /// action to carry out
    #[clap(short, long)]
    action: Action,
    /// name of the text file to compress
    #[clap(short, long, value_delimiter=',')]
    file_names: Vec<String>,
}

impl Argument {
    pub fn action(&self) -> Action {
        self.action.clone()
    }

    pub fn file_names(&self) -> Vec<String> {
        self.file_names.clone()
    }

    /// validate_file_name checks that the file name is a valid one and eats whitespaces
    // TODO(production): should add more validations and file sanitization
    fn validate_file_name(&self, name: &String) -> Result<(), String> {
        if name.split(".").count() != 2 {
            return Err(String::from("file name should be in format 'file_name.file_type'"));
        }

        Ok(())
    }

    pub fn validate_file_names(&self) -> Result<(), String> {
        for name in self.file_names.iter() {
            let result = self.validate_file_name(name);
            if result.is_err() {
                return result;
            }
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
            file_names: file_names.clone(),
        };

        assert_eq!(args.file_names, file_names);
        assert_eq!(args.action.to_string(), Action::Send.to_string());
    }
}
