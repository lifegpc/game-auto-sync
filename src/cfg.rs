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

    pub fn get_bool<S: AsRef<str> + ?Sized>(&self, s: &S) -> Option<&bool> {
        match self.get(s) {
            Some(y) => match y {
                Yaml::Boolean(i) => Some(i),
                _ => None,
            },
            None => None,
        }
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
                Yaml::Array(s) => Some(
                    s.iter()
                        .filter_map(|s| match s {
                            Yaml::String(s) => Some(s.to_owned()),
                            Yaml::Integer(i) => Some(i.to_string()),
                            Yaml::Real(i) => Some(i.to_owned()),
                            _ => None,
                        })
                        .collect(),
                ),
                _ => None,
            },
            None => None,
        }
    }

    pub fn game_backuper_cfg(&self) -> Option<String> {
        let mut pb = crate::utils::get_exe_path_else_current();
        pb.push(
            self.get_str("game_backuper_cfg")
                .unwrap_or("game_backuper.yml"),
        );
        if !pb.exists() {
            return None;
        }
        Some(pb.to_string_lossy().to_string())
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

    pub fn pause_at_exit(&self) -> bool {
        self.get_bool("pause_at_exit")
            .map(|s| s.to_owned())
            .unwrap_or(false)
    }

    pub fn pause_on_backup_error(&self) -> bool {
        self.get_bool("pause_on_backup_error")
            .map(|s| s.to_owned())
            .unwrap_or(false)
    }

    pub fn rclone_exe(&self) -> String {
        match self.get_str("rclone_exe") {
            Some(s) => s.to_owned(),
            None => String::from("rclone"),
        }
    }

    pub fn rclone_remote(&self) -> Option<&str> {
        self.get_str("rclone_remote")
    }

    pub fn rclone_local(&self) -> Option<&str> {
        self.get_str("rclone_local")
    }

    pub fn rclone_flag(&self) -> Vec<String> {
        self.get_str_vec("rclone_flag")
            .unwrap_or(vec!["-P".to_owned()])
    }

    #[cfg(windows)]
    pub fn hide_window_when_running_exe(&self) -> bool {
        self.get_bool("hide_window_when_running_exe")
            .map(|s| s.to_owned())
            .unwrap_or(true)
    }

    pub fn continue_when_run_failed(&self) -> bool {
        self.get_bool("continue_when_run_failed")
            .map(|s| s.to_owned())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    pub fn hook_dll(&self) -> Vec<String> {
        self.get_str_vec("hook_dll").unwrap_or(vec![])
    }
}
