pub mod graph {
    use imgui::Ui;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    use std::{io, process::Command};
    use sysinfo::{Components, System};
    use tokio::time::interval;

    pub struct Cpu {
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
                    fan_info.state = Some(if fan_info.rpm.unwrap() > 0 {
                        "On".to_string()
                    } else {
                        "Off".to_string()
                    });

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
        pub data: Arc<Mutex<Vec<f32>>>,
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
                data: Arc::new(Mutex::new(Vec::with_capacity(max_points))),
                max_points,
                update_interval,
                last_update: Instant::now(),
                is_paused: false,
                fps: 1.0,
                y_scale: 1.0,
            }
        }

        pub fn update(&mut self, new_value: f32) {
            if !self.is_paused {
                let mut data = self.data.lock().unwrap();
                data.push(new_value);
                if data.len() >= self.max_points {
                    data.remove(0);
                }
            }
        }
        // Votre fonction pour tracer les graphiques
        pub fn draw_graph(&self, ui: &Ui, label: &str, hover: &str) {
            let data = self.data.lock().unwrap();
            let (min, max) = data.iter().fold((f32::MAX, f32::MIN), |(min, max), &val| {
                (min.min(val), max.max(val))
            });
            let last_value = data.last().unwrap_or(&0.0);
            let last_value_str = format!("{:.2}", last_value);
            let overlay_text = hover.replace('#', &last_value_str);

            ui.plot_lines(label, &data)
                .graph_size([500.0, 100.0])
                .scale_min(min)
                .scale_max(max * self.y_scale)
                .overlay_text(overlay_text)
                .build();
        }
    }

    pub async fn update_cpu_graph(graph_data: Arc<Mutex<GraphData>>) {
        if graph_data.lock().unwrap().is_paused {
            println!("update_cpu_graph called");
            return;
        }
        let mut interval = interval(Duration::from_secs_f32(
            1.0 / graph_data.lock().unwrap().fps,
        ));
        loop {
            interval.tick().await;
            let cpu_usage = Cpu::get_cpu_usage();
            {
                let mut graph = graph_data.lock().unwrap();
                graph.update(cpu_usage);
            }
        }
    }
    pub async fn update_fan_graph(graph_data: Arc<Mutex<GraphData>>) {
        if graph_data.lock().unwrap().is_paused {
            println!("update_fan_graph called");
            return;
        }
        let mut interval = interval(Duration::from_secs_f32(
            1.0 / graph_data.lock().unwrap().fps,
        ));
        loop {
            interval.tick().await;
            let fan_info_list = Cpu::get_all_fan_info().unwrap();
            {
                let mut graph = graph_data.lock().unwrap();
                graph.update(fan_info_list[0].rpm.unwrap_or(0) as f32);
            }
        }
    }
    pub async fn update_temperature_graph(graph_data: Arc<Mutex<GraphData>>) {
        if graph_data.lock().unwrap().is_paused {
            println!("update_temperature_graph called");
            return;
        }
        let mut interval = interval(Duration::from_secs_f32(
            1.0 / graph_data.lock().unwrap().fps,
        ));
        loop {
            interval.tick().await;
            let cpu_temperature = Cpu::get_cpu_temperatures();
            {
                let mut graph = graph_data.lock().unwrap();
                graph.update(cpu_temperature);
            }
        }
    }

    // Fonction pour ajuster dynamiquement les intervalles en fonction de la valeur de FPS
    pub fn adjust_intervals(
        cpu_graph: Arc<Mutex<GraphData>>,
        fan_graph: Arc<Mutex<GraphData>>,
        temp_graph: Arc<Mutex<GraphData>>,
    ) {
        let fps_cpu = cpu_graph.lock().unwrap().fps;
        let fps_fan = fan_graph.lock().unwrap().fps;
        let fps_temp = temp_graph.lock().unwrap().fps;

        cpu_graph.lock().unwrap().update_interval = Duration::from_secs_f32(1.0 / fps_cpu);
        fan_graph.lock().unwrap().update_interval = Duration::from_secs_f32(1.0 / fps_fan);
        temp_graph.lock().unwrap().update_interval = Duration::from_secs_f32(1.0 / fps_temp);
    }
}
