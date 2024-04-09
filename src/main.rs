mod config;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use clap::Parser;
use crate::config::Config;

#[derive(Debug, Parser)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "Process executable path", required = true)]
    pub path: String,
    #[arg(short = 'w', long = "watchFiles", default_value_t = false, help = "Restart application on file changes in directory/subdirectories")]
    pub watch_files: bool,
    #[arg(short = 'z', long = "onlyNonZeroExit", default_value_t = false, help = "Only exit on a non-zero status code")]
    pub only_non_zero_exit: bool,
    #[arg(short = 'r', long = "restartDelay", default_value_t = 0, help = "Delay before restarting process in ms")]
    pub restart_delay: usize,
    #[arg(short = 'c', long = "recheckDelay", default_value_t = 0, help = "How often the process is checked in ms")]
    pub recheck_delay: usize,
    #[arg(long = "forceRestartDelay", default_value_t = 0, help = "Forces a restart after a delay in ms [0 = Disabled]")]
    pub force_restart_delay: usize,
}


fn main() {
    let args = CommandLineArguments::parse();
    let mut config: Config = args.into();
    let recheck_delay = config.recheck_delay() as u64;
    let restart_delay = config.restart_delay() as u64;
    let dir = config.directory().to_path_buf();
    let watch_files = config.watch_files();
    println!("Starting Process");

    config.start();

    let config = Arc::new(Mutex::new(config));
    let e_config = Arc::clone(&config);


    let t_config = Arc::clone(&config);
    let file_thread = std::thread::spawn(move || {
        if watch_files {
            let cache = Rc::new(Mutex::new(HashMap::new()));
            loop {
                {
                    let changes = Config::check_file_changes(dir.as_path(), Rc::clone(&cache));
                    if changes {
                        println!("File changes detected!");
                        let mut lock = t_config.lock().unwrap();
                        if lock.stop() {
                            if restart_delay > 0 {
                                std::thread::sleep(Duration::from_millis(restart_delay));
                            }
                            lock.start()
                        }
                    }
                }
                if recheck_delay > 0 {
                    std::thread::sleep(Duration::from_millis(recheck_delay));
                }
            }
        }
    });

    let t_config = Arc::clone(&e_config);
    let aliveness_task = std::thread::spawn(move || {
        loop {
            {
                t_config.lock().unwrap().check_process_exit();
            }
            std::thread::sleep(Duration::from_millis(recheck_delay));
        }
    });

    let _ = file_thread.join();
    let _ = aliveness_task.join();

    e_config.lock().unwrap().stop();
}
