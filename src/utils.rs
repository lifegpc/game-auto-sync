use std::env;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// Get executable location, if not found, return current directory (./)
pub fn get_exe_path_else_current() -> PathBuf {
    let re = env::current_exe();
    match re {
        Ok(pa) => {
            let mut p = pa.clone();
            p.pop();
            p
        }
        Err(_) => {
            let p = Path::new("./");
            p.to_path_buf()
        }
    }
}

pub fn ask_continue() -> bool {
    print!("Do you want to continue?(y/n)");
    std::io::stdout().flush().unwrap();
    let mut d = String::from("");
    loop {
        let re = std::io::stdin().read_line(&mut d);
        if re.is_err() {
            continue;
        }
        let d = d.trim().to_lowercase();
        if d == "y" {
            return true;
        } else {
            return false;
        }
    }
}
