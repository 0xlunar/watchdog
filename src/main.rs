mod config;

use std::collections::HashMap;
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
    #[arg(short = 'r', long = "restartDelay", default_value_t = 1000, help = "Delay before restarting process in ms")]
    pub restart_delay: u64,
    #[arg(short = 'c', long = "recheckDelay", default_value_t = 500, help = "How often the process is checked in ms")]
    pub recheck_delay: u64,
    #[arg(long = "forceRestartDelay", default_value_t = 0, help = "Forces a restart after a delay in ms [0 = Disabled]")]
    pub force_restart_delay: u64,
}


fn main() {
    let args = CommandLineArguments::parse();
    let force_restart = args.force_restart_delay;
    let recheck_delay = args.recheck_delay;
    let restart_delay = args.restart_delay;
    let watch_files = args.watch_files;
    let mut config: Config = args.into();
    let dir = config.directory();
    println!("Starting Process");

    config.start();

    let config = Arc::new(Mutex::new(config));

    let t_config = Arc::clone(&config);
    let file_thread = std::thread::spawn(move || {
        let dir_path = dir.as_path();
        if watch_files {
            let mut cache = HashMap::new();
            loop {
                {
                    let changes = Config::check_file_changes(dir_path, &mut cache);
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

    let t_config = Arc::clone(&config);
    let aliveness_task = std::thread::spawn(move || {
        loop {
            {
                t_config.lock().unwrap().check_process_exit();
            }
            std::thread::sleep(Duration::from_millis(recheck_delay));
        }
    });

    let t_config = Arc::clone(&config);
    let force_restart_task = std::thread::spawn(move || {
       if force_restart > 0 {
           loop {
               println!("Application will restart in: {} seconds", force_restart as f64 / 1000.00);
               std::thread::sleep(Duration::from_millis(force_restart)); // wait time period
               println!("Force restarting application");
               let mut lock = t_config.lock().unwrap();
               if lock.stop() {
                   lock.start();
               }
           }
       }
    });

    let _ = file_thread.join();
    let _ = aliveness_task.join();
    let _ = force_restart_task.join();

    config.lock().unwrap().stop();
}
