mod cfg;
mod utils;
#[cfg(windows)]
mod windows;

use getopts::Options;
use std::process::ExitCode;
use subprocess::ExitStatus;

pub fn print_usage(prog: &str, opts: &Options) {
    let brief = format!(
        "{}
{} [options]",
        "Usage:", prog,
    );
    println!("{}", opts.usage(brief.as_str()));
}

#[derive(Debug, derive_more::Display, derive_more::From)]
enum Error {
    #[cfg(not(windows))]
    Popen(subprocess::PopenError),
    #[cfg(windows)]
    Popen(windows::PopenError),
    Exited,
}

struct Main {
    _cfg: cfg::Config,
    _dryrun: bool,
    _rclone_enable: bool,
    _skip_restore: bool,
    _backup_only: bool,
    _run_only: bool,
}

impl Main {
    fn new(cfg: cfg::Config, dryrun: bool, skip_restore: bool, backup_only: bool, run_only: bool) -> Self {
        Self {
            _rclone_enable: cfg.rclone_remote().is_some() && cfg.rclone_local().is_some(),
            _cfg: cfg,
            _dryrun: dryrun,
            _skip_restore: skip_restore,
            _backup_only: backup_only,
            _run_only: run_only,
        }
    }

    fn backup(&self) -> Result<(), Error> {
        let cml = match self._cfg.backup_command() {
            Some(cml) => cml,
            None => {
                let mut def = vec![self._cfg.game_backuper_exe()];
                if let Some(cfg_path) = self._cfg.game_backuper_cfg() {
                    def.push(String::from("-c"));
                    def.push(cfg_path);
                }
                def.push(String::from("backup"));
                def
            }
        };
        if self._dryrun {
            println!("Backup command line: {:?}", cml);
            Ok(())
        } else {
            let e = Self::call(cml)?;
            let ok = match &e {
                ExitStatus::Exited(c) => *c == 0,
                _ => false,
            };
            if !ok {
                println!("Backup failed: {:?}.", e);
                if self._rclone_enable {
                    if !utils::ask_continue() {
                        return Err(Error::Exited);
                    }
                    return Ok(());
                }
                return Err(Error::Exited);
            }
            Ok(())
        }
    }

    fn backup_rclone(&self) -> Result<(), Error> {
        let mut cml = vec![self._cfg.rclone_exe(), String::from("sync")];
        cml.push(self._cfg.rclone_local().unwrap().to_owned());
        cml.push(self._cfg.rclone_remote().unwrap().to_owned());
        cml.extend_from_slice(&self._cfg.rclone_flag());
        if self._dryrun {
            println!("Rclone backup command line: {:?}", cml);
            Ok(())
        } else {
            let e = Self::call(cml)?;
            let ok = match &e {
                ExitStatus::Exited(c) => *c == 0,
                _ => false,
            };
            if !ok {
                println!("Rclone backup failed: {:?}.", e);
                if self._rclone_enable {
                    if !utils::ask_continue() {
                        return Err(Error::Exited);
                    }
                    return Ok(());
                }
                return Err(Error::Exited);
            }
            Ok(())
        }
    }

    #[cfg(not(windows))]
    fn call(cml: Vec<String>) -> Result<ExitStatus, subprocess::PopenError> {
        let mut p = subprocess::Popen::create(&cml, subprocess::PopenConfig::default())?;
        p.wait()
    }

    #[cfg(windows)]
    fn call(cml: Vec<String>) -> Result<ExitStatus, windows::PopenError> {
        let t = Vec::<String>::new();
        windows::call(&cml, &t, None::<String>).map(|c| ExitStatus::Exited(c))
    }

    #[cfg(windows)]
    fn call2(cml: Vec<String>, dlls: Vec<String>, cdir: Option<String>) -> Result<ExitStatus, windows::PopenError> {
        windows::call(&cml, &dlls, cdir).map(|c| ExitStatus::Exited(c))
    }

    fn restore(&self) -> Result<(), Error> {
        let cml = match self._cfg.restore_command() {
            Some(cml) => cml,
            None => {
                let mut def = vec![self._cfg.game_backuper_exe()];
                if let Some(cfg_path) = self._cfg.game_backuper_cfg() {
                    def.push(String::from("-c"));
                    def.push(cfg_path);
                }
                def.push(String::from("restore"));
                def
            }
        };
        if self._dryrun {
            println!("Restore command line: {:?}", cml);
            Ok(())
        } else {
            let e = Self::call(cml)?;
            let ok = match &e {
                ExitStatus::Exited(c) => *c == 0,
                _ => false,
            };
            if !ok {
                println!("Restore failed: {:?}.", e);
                if !utils::ask_continue() {
                    return Err(Error::Exited);
                }
            }
            Ok(())
        }
    }

    fn restore_rclone(&self) -> Result<(), Error> {
        let mut cml = vec![self._cfg.rclone_exe(), String::from("sync")];
        cml.push(self._cfg.rclone_remote().unwrap().to_owned());
        cml.push(self._cfg.rclone_local().unwrap().to_owned());
        cml.extend_from_slice(&self._cfg.rclone_flag());
        if self._dryrun {
            println!("Rclone restore command line: {:?}", cml);
            Ok(())
        } else {
            let e = Self::call(cml)?;
            let ok = match &e {
                ExitStatus::Exited(c) => *c == 0,
                _ => false,
            };
            if !ok {
                println!("Rclone restore failed: {:?}.", e);
                if !utils::ask_continue() {
                    return Err(Error::Exited);
                }
            }
            Ok(())
        }
    }

    fn run(&self) -> Result<(), Error> {
        if !self._run_only && !self._skip_restore && !self._backup_only {
            if self._rclone_enable {
                self.restore_rclone()?;
            }
            self.restore()?;
        }
        if self._run_only || !self._backup_only {
            self.run_exe()?;
        }
        if !self._run_only {
            self.backup()?;
            if self._rclone_enable {
                self.backup_rclone()?;
            }
        }
        Ok(())
    }

    fn run_exe(&self) -> Result<(), Error> {
        let cml = self._cfg.game_exe().unwrap();
        if self._dryrun {
            println!("Run command line: {:?}", cml);
            Ok(())
        } else {
            #[cfg(windows)]
            let need_hide = self._cfg.hide_window_when_running_exe();
            #[cfg(windows)]
            let hide = if need_hide {
                windows::hide_window()
            } else {
                false
            };
            #[cfg(windows)]
            if need_hide && !hide {
                println!("Failed to hide console window.");
            }
            #[cfg(not(windows))]
            let e = Self::call(cml)?;
            #[cfg(windows)]
            let e = Self::call2(cml, self._cfg.hook_dll(), self._cfg.current_dir())?;
            #[cfg(windows)]
            if hide {
                windows::show_window();
            }
            let ok = match &e {
                ExitStatus::Exited(c) => *c == 0,
                _ => false,
            };
            if !ok {
                println!("Run failed: {:?}.", e);
                if !self._cfg.continue_when_run_failed() && !utils::ask_continue() {
                    return Err(Error::Exited);
                }
            }
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    let mut opts = Options::new();
    opts.optflag("h", "help", "Print help message.");
    opts.optopt("c", "config", "The location of config file.", "FILE");
    opts.optflag("d", "dryrun", "Run without calling any process.");
    opts.optflag("r", "skip-restore", "Skip restore backup.");
    opts.optflag("b", "backup-only", "Backup only.");
    opts.optflag("R", "run-only", "Run only. Do not backup or restore.");
    let result = match opts.parse(&argv[1..]) {
        Ok(m) => m,
        Err(err) => {
            println!("{}", err.to_string());
            return ExitCode::from(1);
        }
    };
    if result.opt_present("h") {
        print_usage(&argv[0], &opts);
        return ExitCode::from(0);
    }
    let cfg_path = result.opt_str("c").unwrap_or_else(|| {
        let mut pb = utils::get_exe_path_else_current();
        pb.push("game-auto-sync.yml");
        pb.to_string_lossy().to_string()
    });
    let cfg = match cfg::Config::from_file_path(&cfg_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("{}", e);
            return ExitCode::from(1);
        }
    };
    if cfg.game_exe().unwrap_or(vec![]).is_empty() {
        println!("game_exe need be set.");
        return ExitCode::from(1);
    }
    let m = Main::new(
        cfg,
        result.opt_present("d"),
        result.opt_present("r"),
        result.opt_present("b"),
        result.opt_present("R"),
    );
    let e = match m.run() {
        Ok(_) => 0,
        Err(e) => {
            println!("{}", e);
            1
        }
    };
    if m._cfg.pause_at_exit() || (e == 1 && m._cfg.pause_on_backup_error()) {
        utils::enter_continue();
    }
    return ExitCode::from(e);
}
