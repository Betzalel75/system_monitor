pub mod graph {
    use imgui::Ui;
    use std::str::FromStr;
    use std::time::{Duration, Instant};
    use std::{io, process::Command};
    use sysinfo::{Components, System};

    pub struct Cpu {
        pub cpu_percent: f32,
        pub temperatures: f32,
        pub fan_info: usize,
    }

    // Structure pour stocker les informations sur le ventilateur
    #[derive(Debug)]
    pub struct FanInfo {
        pub rpm: Option<i32>,
        pub min_rpm: Option<i32>,
        pub max_rpm: Option<i32>,
        pub state: Option<String>,
    }

    impl Cpu {
        pub fn new() -> Cpu {
            Self {
                cpu_percent: 0.0,
                temperatures: 0.0,
                fan_info: 0,
            }
        }

        pub fn get_cpu_temperatures() -> f32 {
            let components = Components::new_with_refreshed_list();
            let mut tmp = 0.0;
            for component in &components {
                if component.label().contains("coretemp Package id 0") {
                    tmp = component.temperature();
                    break;
                } else {
                    continue;
                }
            }
            tmp
        }

        pub fn get_all_fan_info() -> io::Result<Vec<FanInfo>> {
            // Exécutez la commande sensors
            let output = Command::new("sensors")
                .output()
                .expect("Erreur lors de l'exécution de la commande sensors");

            // Vérifiez si la commande a été exécutée avec succès
            if !output.status.success() {
                eprintln!("Erreur lors de l'exécution de la commande sensors.");
                return Ok(vec![]);
            }

            // Convertissez la sortie en string
            let result = String::from_utf8_lossy(&output.stdout);
            let mut fan_info_list: Vec<FanInfo> = Vec::new();

            // Parcourez chaque ligne de la sortie
            for line in result.lines() {
                if line.contains("fan1:") {
                    let mut fan_info = FanInfo {
                        rpm: None,
                        min_rpm: None,
                        max_rpm: None,
                        state: None,
                    };

                    // Utilisez des expressions régulières pour extraire les valeurs de vitesse et d'état
                    let re = regex::Regex::new(
                        r"fan1:\s*(\d+)\s*RPM\s*\(min\s*=\s*(\d+)\s*RPM,\s*max\s*=\s*(\d+)\s*RPM\)",
                    )
                    .unwrap();
                    if let Some(caps) = re.captures(line) {
                        fan_info.rpm = Some(i32::from_str(&caps[1]).unwrap_or_default());
                        fan_info.min_rpm = Some(i32::from_str(&caps[2]).unwrap_or_default());
                        fan_info.max_rpm = Some(i32::from_str(&caps[3]).unwrap_or_default());
                    }

                    // Vérifiez l'état du ventilateur (on/off)
                    if line.contains("fan:  on") {
                        fan_info.state = Some(String::from("On"));
                    } else if line.contains("fan:  off") {
                        fan_info.state = Some(String::from("Off"));
                    }

                    fan_info_list.push(fan_info);
                }
            }

            Ok(fan_info_list)
        }

        pub fn get_cpu_usage() -> f32 {
            let mut sys = System::new_all();
            // First we update all information of our `System` struct.
            sys.refresh_all();
            let cpu_usage: f32 = sys.global_cpu_info().cpu_usage();
            cpu_usage
        }
    }

    pub struct GraphData {
        pub data: Vec<f32>,
        pub max_points: usize,
        pub update_interval: Duration,
        pub last_update: Instant,
        pub is_paused: bool,
        pub fps: f32,
        pub y_scale: f32,
    }

    impl GraphData {
        pub fn new(max_points: usize, update_interval: Duration) -> Self {
            Self {
                data: Vec::with_capacity(max_points),
                max_points,
                update_interval,
                last_update: Instant::now(),
                is_paused: false,
                fps: 10.0,
                y_scale: 1.0,
            }
        }

        pub fn update(&mut self, new_value: f32) {
            if !self.is_paused && self.last_update.elapsed() >= self.update_interval {
                if self.data.len() == self.max_points {
                    self.data.remove(0);
                }
                self.data.push(new_value);
                self.last_update = Instant::now();
            }
        }
        // Votre fonction pour tracer les graphiques
        pub fn draw_graph(&self, ui: &Ui, label: &str, hover: &str) {
            let (min, max) = self
                .data
                .iter()
                .fold((f32::MAX, f32::MIN), |(min, max), &val| {
                    (min.min(val), max.max(val))
                });

            ui.plot_lines(label, &self.data)
                .graph_size([500.0, 100.0])
                .scale_min(min)
                .scale_max(max * self.y_scale)
                .overlay_text(hover)
                .build();
        }
    }
}

/*
pub fn print_all_info(){
            let cpu_usage = Cpu::get_cpu_usage();
            let cpu_temp = Cpu::get_cpu_temperatures();
            let fan_info_list = Cpu::get_all_fan_info().unwrap_or(Vec::new());

            println!("CPU Usage: {:.2}%", cpu_usage);
            println!("CPU Temperature: {:.2}°C", cpu_temp);
            println!("Fan Information:");
            for fan_info in fan_info_list {
                println!(
                    "RPM: {}, Min RPM: {}, Max RPM: {}, State: {}",
                    fan_info.rpm.unwrap_or(0),
                    fan_info.min_rpm.unwrap_or(0),
                    fan_info.max_rpm.unwrap_or(0),
                    fan_info.state.unwrap_or(String::from("Unknown"))
                );
            }
        }
*/
