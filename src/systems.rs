pub mod system {
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
                "{} {}",
                System::name().unwrap(),
                System::os_version().unwrap()
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
}
