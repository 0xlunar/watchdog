use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{Duration, UNIX_EPOCH};
use crate::CommandLineArguments;

pub struct Config {
    directory: PathBuf,
    file_name: OsString,
    process: Option<Child>,
    watch_files: bool,
    only_non_zero_exit: bool,
    restart_delay: usize,
    recheck_delay: usize,
}

impl Config {
    pub fn start(&mut self) {
        let mut full_path = self.directory.clone();
        full_path.push(self.file_name.as_os_str());

        if !full_path.try_exists().unwrap() {
            panic!("File doesn't exist")
        }

        let process = std::process::Command::new(full_path)
            .current_dir(&self.directory)
            .spawn().expect("Cannot spawn process");

        self.process = Some(process);
    }
    pub fn stop(&mut self) -> bool {
        let success = match &mut self.process {
            None => {
                println!("No process");
                false
            },
            Some(c) => match c.kill() {
                Ok(_) => true,
                Err(e) => {
                    println!("Failed to terminate process: {}", e);
                    false
                },
            }
        };

        if success {
            self.process = None;
        }

        success
    }

    pub fn directory(&self) -> &Path {
        self.directory.as_path()
    }
    pub fn restart_delay(&self) -> usize {
        self.restart_delay
    }
    pub fn recheck_delay(&self) -> usize {
        self.recheck_delay
    }
    pub fn watch_files(&self) -> bool {
        self.watch_files
    }

    pub fn check_file_changes(directory: &Path, cache: Rc<Mutex<HashMap<OsString, u64>>>) -> bool {
        let paths = fs::read_dir(directory).unwrap();
        let mut file_updated = false;
        let mut cache_lock = cache.lock().unwrap();
        for path in paths {
            let path = path.unwrap();
            let file_name = path.file_name();
            let metadata = path.metadata().unwrap();
            let value = match metadata.modified() {
                // If System supports modified time we use the time, otherwise just use file size. Will miss if change resulted in exact same file size
                Ok(t) => t.duration_since(UNIX_EPOCH).unwrap().as_secs(),
                Err(_) => metadata.len()
            };

            cache_lock.entry(file_name).and_modify(|v| {
                if *v != value {
                    file_updated = true;
                    *v = value;
                }
            }).or_insert(value);

            if file_updated {
                break;
            }
        }

        file_updated
    }
    pub fn check_process_exit(&mut self) {
        match &mut self.process {
            Some(c) => match &c.try_wait() {
                Ok(t) => match t {
                    Some(es) => {
                        println!("Process Exited: {:?}", es);
                        if es.success() && !self.only_non_zero_exit {
                            if self.restart_delay > 0 {
                                std::thread::sleep(Duration::from_millis(self.restart_delay as u64));
                            }
                            self.start()
                        } else if !es.success() {
                            if self.restart_delay > 0 {
                                std::thread::sleep(Duration::from_millis(self.restart_delay as u64));
                            }
                            self.start()
                        }
                    },
                    None => ()
                },
                Err(e) => println!("Error waiting: {}", e)
            },
            None => ()
        }
    }
}



impl From<CommandLineArguments> for Config {
    fn from(item: CommandLineArguments) -> Self {
        let directory = PathBuf::from(item.path);
        let mut directory = fs::canonicalize(directory);
        println!("Dir: {:?}", directory);
        let mut directory = directory.unwrap();
        let file_name = match directory.extension() {
            Some(_) => {
                let stem = directory.file_name().unwrap().to_os_string();
                directory.pop();
                stem
            },
            None => panic!("Invalid file path, eg \"./My Application\\app.exe\" or \"C:\\Users\\You\\Desktop\\My Application\\app.exe\"")
        };

        Self {
            directory,
            file_name,
            process: None,
            watch_files: item.watch_files,
            only_non_zero_exit: item.only_non_zero_exit,
            restart_delay: item.restart_delay,
            recheck_delay: item.recheck_delay,
        }
    }
}