# watchdog
Watch processes for file changes and exits

## Usage
`watchdog -p ./my_process.exe -w`
```
Usage: watchdog.exe [OPTIONS] --path <PATH>
                                           
Options:                                   
  -p, --path <PATH>                        
          Process executable path          
  -w, --watchFiles                         
          Restart application on file changes in directory/subdirectories
  -z, --onlyNonZeroExit
          Only exit on a non-zero status code
  -r, --restartDelay <RESTART_DELAY>
          Delay before restarting process in ms [default: 1000]
  -c, --recheckDelay <RECHECK_DELAY>
          How often the process is checked in ms [default: 500]
      --forceRestartDelay <FORCE_RESTART_DELAY>
          Forces a restart after a delay in ms [0 = Disabled] [default: 0]
  -h, --help
          Print help
```