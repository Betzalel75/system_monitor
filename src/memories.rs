pub mod memory_util {
    use sysinfo::{Disks, System};

    pub struct Swap {
        pub total_swap: u64,
        pub used_swap: u64,
        pub free_swap: u64,
    }
    impl Swap {
        pub fn new() -> Swap {
            Swap {
                total_swap: 0,
                used_swap: 0,
                free_swap: 0,
            }
        }
    }
    pub struct Ram {
        pub total_ram: u64,
        pub used_ram: u64,
        pub free_ram: u64,
    }
    impl Ram {
        pub fn new() -> Ram {
            Ram {
                total_ram: 0,
                used_ram: 0,
                free_ram: 0,
            }
        }
    }
    pub struct Disk {
        pub total_disk: u64,
        pub used_disk: u64,
        pub free_disk: u64,
    }
    impl Disk {
        pub fn new() -> Disk {
            Disk {
                total_disk: 0,
                used_disk: 0,
                free_disk: 0,
            }
        }
    }
    pub struct Memory {
        pub storage: Disk,
        pub swap: Swap,
        pub ram: Ram,
    }
    impl Memory {
        pub fn new() -> Memory {
            let storage = Disk::new();
            let swap = Swap::new();
            let ram = Ram::new();
            Memory { storage, swap, ram }
        }

        pub fn get_memory(&mut self) {
            //... get memory information
            let mut sys = System::new_all();

            // First we update all information of our `System` struct.
            sys.refresh_all();
            // RAM and swap information:
            self.ram.total_ram = sys.total_memory();
            self.ram.used_ram = sys.used_memory();
            self.ram.free_ram = self.ram.total_ram - self.ram.used_ram;
            // swap information:
            self.swap.total_swap = sys.total_swap();
            self.swap.used_swap = sys.used_swap();
            self.swap.free_swap = self.swap.total_swap - self.swap.used_swap ;
            // disk information:
            let disks = Disks::new_with_refreshed_list();
            self.storage.free_disk = disks[0].available_space();
            self.storage.total_disk = disks[0].total_space();
            self.storage.used_disk = self.storage.total_disk - self.storage.free_disk;
        }
    }

    pub fn bytes_to_mo(size: u64) -> f32{
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            // Convert bytes to GB and format the result to two decimal places
            size as f32 / GB as f32
        } else if size >= MB {
            // Convert bytes to MB and format the result to two decimal places
             size as f32/ MB as f32
        } else if size >= KB {
            // Convert bytes to KB and format the result to two decimal places
            size as f32/ KB as f32
        } else {
            // If size is less than 1 KB, just show the size in bytes
            size as f32
        }
    }

    pub fn convert_bytes_to_any(size: u64) -> String {
        //... convert bytes to any unit (MB, GB, etc.)
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            // Convert bytes to GB and format the result to two decimal places
            format!("{:.2}Go", bytes_to_mo(size))
        } else if size >= MB {
            // Convert bytes to MB and format the result to two decimal places
            format!("{:.2}Mo", bytes_to_mo(size))
        } else if size >= KB {
            // Convert bytes to KB and format the result to two decimal places
            format!("{:.2}Ko", bytes_to_mo(size))
        } else {
            // If size is less than 1 KB, just show the size in bytes
            format!("{}B", size)
        }
    }
}
