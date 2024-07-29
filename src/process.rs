pub mod process {
    use std::{
        collections::HashSet,
        fs,
        io::{BufRead, BufReader, Read},
        path::Path,
    };

    use imgui::Ui;
    use sysinfo::{Process, System};

    #[derive(Debug, Clone)]
    pub struct ProcessInfo {
        pub pid: usize,
        pub name: String,
        pub state: String,
        pub cpu_usage: f32,
        pub memory_usage: f32,
    }

    impl ProcessInfo {
        fn from_process(process: &Process, system: &System) -> Self {
            let pid = process.pid();
            let name = process.name().to_string();
            let state = process.status().to_string();
            let cpu_usage = process.cpu_usage(); // CPU usage of this process
            let memory_usage = (process.memory() as f32) / (system.total_memory() as f32) * 100.0; // Memory usage percentage

            Self {
                pid: pid.as_u32() as usize,
                name,
                state,
                cpu_usage,
                memory_usage,
            }
        }
    }

    pub fn get_processes_info(system: &mut System) -> Vec<ProcessInfo> {
        // First we update all information of our `System` struct.
        system.refresh_all();
        system
            .processes()
            .iter()
            .map(|(_, process)| ProcessInfo::from_process(process, system))
            .collect()
    }

    pub fn get_process_info() -> Vec<ProcessInfo> {
        let mut processes = Vec::new();

        let proc_dir = Path::new("/proc");
        if let Ok(entries) = fs::read_dir(proc_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(pid_str) = path.file_name().and_then(|s| s.to_str()) {
                        if let Ok(pid) = pid_str.parse::<usize>() {
                            let mut process = ProcessInfo {
                                pid,
                                name: String::new(),
                                state: String::new(),
                                cpu_usage: 0.0,
                                memory_usage: 0.0,
                            };

                            // Lire le nom du processus à partir du fichier cmdline
                            let cmdline_path = path.join("cmdline");
                            if let Ok(cmdline_file) = fs::File::open(&cmdline_path) {
                                let mut cmdline = String::new();
                                if let Ok(_) =
                                    BufReader::new(cmdline_file).read_to_string(&mut cmdline)
                                {
                                    process.name = cmdline.trim_matches('\0').to_string();
                                }
                            }

                            // Lire les informations à partir du fichier stat
                            let stat_path = path.join("stat");
                            if let Ok(stat_file) = fs::File::open(&stat_path) {
                                let mut line = String::new();
                                if let Ok(_) = BufReader::new(stat_file).read_line(&mut line) {
                                    let parts: Vec<&str> = line.split_whitespace().collect();
                                    if parts.len() > 13 {
                                        process.name = parts[1]
                                            .trim_matches('(')
                                            .trim_matches(')')
                                            .to_string();
                                        process.state = parts[2].to_string();
                                        let utime = parts[13].parse::<f32>().unwrap_or(0.0);
                                        let stime = parts[14].parse::<f32>().unwrap_or(0.0);
                                        process.cpu_usage = (utime * 100.0) / (utime + stime);
                                    }
                                }
                            }

                            // Lire la mémoire virtuelle (VmSize) et la mémoire résidente (VmRSS) à partir du fichier status
                            let status_path = path.join("status");
                            if let Ok(status_file) = fs::File::open(&status_path) {
                                for line in BufReader::new(status_file).lines() {
                                    if let Ok(line) = line {
                                        let mut parts = line.split_whitespace();
                                        if let (Some(key), Some(value)) =
                                            (parts.next(), parts.next())
                                        {
                                            match key {
                                                "VmSize:" => {
                                                    let vsize = value.parse::<i64>().unwrap_or(0);
                                                    let total_ram = 16 * 1024 * 1024; // Exemple de taille totale de RAM en Ko
                                                    process.memory_usage =
                                                        (vsize as f32 * 100.0) / total_ram as f32;
                                                }
                                                "VmRSS:" => {
                                                    let rss = value.parse::<i64>().unwrap_or(0);
                                                    let total_ram = 16 * 1024 * 1024; // Exemple de taille totale de RAM en Ko
                                                    process.memory_usage =
                                                        (rss as f32 * 100.0) / total_ram as f32;
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }

                            processes.push(process);
                        }
                    }
                }
            }
        }

        processes
    }

    // Process Table
    pub fn draw_process_table(ui: &Ui, _system: &mut System, selected_pids: &mut HashSet<usize>) {
        // Obtenez les informations des processus
        let processes = get_process_info();

        let draw_list: imgui::DrawListMut = ui.get_window_draw_list();

        // Afficher le champ de filtre
        let mut search_buffer = String::new();
        let mut selectables: Vec<bool> = vec![false; processes.len()];

        ui.text("Process Informations:");
        ui.text(format!("Total Processes: {}", processes.len()));
        ui.input_text("Search", &mut search_buffer).build();

        // Afficher le tableau
        ui.columns(6, "ProcessColumns", true);
        ui.text("PID");
        ui.next_column();
        ui.text("Name");
        ui.next_column();
        ui.text("State");
        ui.next_column();
        ui.text("CPU Usage");
        ui.next_column();
        ui.text("Memory Usage");
        ui.next_column();
        ui.separator();

        // Filtrez les processus en fonction de la recherche
        let filtered_processes: Vec<&ProcessInfo> = processes
            .iter()
            .filter(|p| p.name.contains(&search_buffer))
            .collect();

        for (i, process) in filtered_processes.iter().enumerate() {
            let selectable = selectables[i];

            let process_name = &process.name;
            let sz_el = [
                ui.content_region_avail()[0],
                ui.text_line_height_with_spacing() * filtered_processes.len() as f32,
            ];
            ui.selectable(process_name);

            selectables[i] = selectable;

            draw_list.channels_split(2, |channels| {
                if selected_pids.contains(&process.pid) {
                    channels.set_current(1);
                    // ... Draw channel 1
                    draw_list
                        .add_rect(
                            ui.cursor_screen_pos(),
                            [
                                ui.cursor_screen_pos()[0] + sz_el[0],
                                ui.cursor_screen_pos()[1] + ui.text_line_height_with_spacing(),
                            ],
                            imgui::ImColor32::from_rgb(0, 128, 255),
                        )
                        .build();
                } else {
                    channels.set_current(0);
                    // ... Draw channel 0
                    draw_list
                        .add_rect(
                            ui.cursor_screen_pos(),
                            [
                                ui.cursor_screen_pos()[0] + sz_el[0],
                                ui.cursor_screen_pos()[1] + ui.text_line_height_with_spacing(),
                            ],
                            imgui::ImColor32::from_rgb(0, 0, 0),
                        )
                        .build();
                }
            });

            if selectable {
                if selected_pids.contains(&process.pid) {
                    selected_pids.remove(&process.pid);
                } else {
                    selected_pids.insert(process.pid);
                }
            }

            ui.next_column();
            ui.text(format!("{}", process.pid));
            ui.next_column();
            ui.text(&process.name);
            ui.next_column();
            ui.text(&process.state);
            ui.next_column();
            ui.text(format!("{:.2}%", process.cpu_usage));
            ui.next_column();
            ui.text(format!("{:.2}%", process.memory_usage));
            ui.next_column();
            ui.separator();
        }
    }
}
