pub mod system {
    use std::{fs::File, io::{BufRead, BufReader}};

    use sysinfo::System;
    use users::{get_current_uid, get_user_by_uid};

    pub struct Computer {
        pub cpu_core_count: usize,
        pub cpu_info: String,
        pub os_info: String,
        pub hostname: String,
        pub username: String,
    }

    impl Computer {
        pub fn new() -> Computer {
            let mut computer = Self {
                cpu_core_count: 0,
                cpu_info: "N/A".to_string(),
                os_info: "N/A".to_string(),
                hostname: "N/A".to_string(),
                username: "N/A".to_string(),
            };
            computer.initialize();
            computer
        }

        fn initialize(&mut self) {
            let mut sys = System::new_all();
            // First we update all information of our `System` struct.
            sys.refresh_all();
            self.cpu_core_count = sys.cpus().len();
            self.cpu_info = sys.cpus()[0].brand().to_string();
            self.os_info = format!(
                "{}",
                get_os_info()
            );
            self.hostname = System::host_name().unwrap().to_string();
            self.username = Self::get_user_name();
        }
        pub fn get_user_name() -> String {
            match get_user_by_uid(get_current_uid()) {
                Some(user) => user.name().to_str().unwrap().to_string(),
                None => "N/A".to_string(),
            }
        }
    }
    fn get_os_info() -> String {
        let file = File::open("/etc/os-release");
        if let Ok(file) = file {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if line.contains("PRETTY_NAME") {
                        let parts: Vec<&str> = line.split('=').collect();
                        if parts.len() > 1 {
                            let os_info = parts[1].trim_start_matches('"').trim_end_matches('"').to_string();
                            return os_info;
                        }
                    }
                }
            }
        }
        "N/A".to_string()
    }
}
