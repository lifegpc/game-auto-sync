use std::path::Path;
use std::{fs::File, io::Read};
use yaml_rust::{yaml::Hash, ScanError, Yaml, YamlLoader};

#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum ConfigError {
    YAML(ScanError),
    IO(std::io::Error),
    Invalid,
}

#[derive(Debug)]
pub struct Config {
    obj: Hash,
}

impl Config {
    pub fn from_file_path<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Self, ConfigError> {
        let mut f = File::open(path.as_ref())?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        Self::from_str(&s)
    }

    pub fn from_str<S: AsRef<str> + ?Sized>(s: &S) -> Result<Self, ConfigError> {
        let re = YamlLoader::load_from_str(s.as_ref())?;
        if re.is_empty() {
            return Err(ConfigError::Invalid);
        }
        let re = re[0].clone();
        let obj = match re {
            Yaml::Hash(h) => h,
            _ => {
                return Err(ConfigError::Invalid);
            }
        };
        Ok(Self { obj })
    }

    pub fn get<S: AsRef<str> + ?Sized>(&self, s: &S) -> Option<&Yaml> {
        let k = Yaml::from_str(s.as_ref());
        self.obj.get(&k)
    }

    pub fn get_str<S: AsRef<str> + ?Sized>(&self, s: &S) -> Option<&str> {
        match self.get(s) {
            Some(y) => match y {
                Yaml::String(s) => Some(s),
                _ => None,
            },
            None => None,
        }
    }

    pub fn get_str_vec<S: AsRef<str> + ?Sized>(&self, s: &S) -> Option<Vec<String>> {
        match self.get(s) {
            Some(e) => match e {
                Yaml::String(s) => Some(vec![s.to_owned()]),
                Yaml::Array(s) => Some(s
                    .iter()
                    .filter_map(|s| match s {
                        Yaml::String(s) => Some(s.to_owned()),
                        Yaml::Integer(i) => Some(i.to_string()),
                        Yaml::Real(i) => Some(i.to_owned()),
                        _ => None,
                    })
                    .collect()),
                _ => None,
            },
            None => None,
        }
    }

    pub fn game_backuper_cfg(&self) -> String {
        match self.get_str("game_backuper_cfg") {
            Some(s) => {
                let mut pb = crate::utils::get_exe_path_else_current();
                pb.push(s);
                pb.to_string_lossy().to_string()
            },
            None => {
                let mut pb = crate::utils::get_exe_path_else_current();
                pb.push("game_backuper.yml");
                pb.to_string_lossy().to_string()
            },
        }
    }

    pub fn game_backuper_exe(&self) -> String {
        match self.get_str("game_backuper_exe") {
            Some(s) => s.to_owned(),
            None => String::from("game-backuper"),
        }
    }

    pub fn game_exe(&self) -> Option<Vec<String>> {
        self.get_str_vec("game_exe")
    }

    pub fn backup_command(&self) -> Option<Vec<String>> {
        self.get_str_vec("backup_command")
    }

    pub fn restore_command(&self) -> Option<Vec<String>> {
        self.get_str_vec("restore_command")
    }
}
