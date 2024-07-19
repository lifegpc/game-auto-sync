mod cfg;
mod utils;

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
    Popen(subprocess::PopenError),
    Exited,
}

struct Main {
    _cfg: cfg::Config,
    _dryrun: bool,
}

impl Main {
    fn new(cfg: cfg::Config, dryrun: bool) -> Self {
        Self {
            _cfg: cfg,
            _dryrun: dryrun,
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
                ExitStatus::Exited(c) => *c != 0,
                _ => false,
            };
            if !ok {
                println!("Backup failed: {:?}.", e);
                return Err(Error::Exited);
            }
            Ok(())
        }
    }

    fn call(cml: Vec<String>) -> Result<ExitStatus, subprocess::PopenError> {
        let mut p = subprocess::Popen::create(&cml, subprocess::PopenConfig::default())?;
        p.wait()
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
                ExitStatus::Exited(c) => *c != 0,
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

    fn run(&self) -> Result<(), Error> {
        self.restore()?;
        self.run_exe()?;
        self.backup()?;
        Ok(())
    }

    fn run_exe(&self) -> Result<(), Error> {
        let cml = self._cfg.game_exe().unwrap();
        if self._dryrun {
            println!("Run command line: {:?}", cml);
            Ok(())
        } else {
            let e = Self::call(cml)?;
            let ok = match &e {
                ExitStatus::Exited(c) => *c != 0,
                _ => false,
            };
            if !ok {
                println!("Run failed: {:?}.", e);
                if !utils::ask_continue() {
                    return Err(Error::Exited);
                }
            }
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    println!("Hello, world!");
    let argv: Vec<String> = std::env::args().collect();
    let mut opts = Options::new();
    opts.optflag("h", "help", "Print help message.");
    opts.optopt("c", "config", "The location of config file.", "FILE");
    opts.optflag("d", "dryrun", "Run without calling any process.");
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
    let m = Main::new(cfg, result.opt_present("d"));
    match m.run() {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            return ExitCode::from(1);
        }
    }
    return ExitCode::from(0);
}
