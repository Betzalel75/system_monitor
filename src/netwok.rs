pub mod network {
    use imgui::{ImColor32, Ui};
    use std::net::Ipv4Addr;
    use sysinfo::Networks;

    use crate::convert_bytes_to_any;
    extern crate pnet;

    pub struct RxStats {
        pub bytes: u64,
        pub packets: u64,
        pub errs: u64,
        pub drop: u64,
        pub fifo: u64,
        pub frame: u64,
        pub compressed: u64,
        pub multicast: u64,
    }

    pub struct TxStats {
        pub bytes: u64,
        pub packets: u64,
        pub errs: u64,
        pub drop: u64,
        pub fifo: u64,
        pub colls: u64,
        pub carrier: u64,
        pub compressed: u64,
    }

    pub struct Interface {
        pub name: String,
        pub ip: Ipv4Addr,
        pub total_received: u64,
        pub total_transmitted: u64,
        pub rx_stats: Option<RxStats>,
        pub tx_stats: Option<TxStats>,
    }

    impl Interface {
        pub fn new(
            name: String,
            ip: Ipv4Addr,
            total_received: u64,
            total_transmitted: u64,
        ) -> Interface {
            Interface {
                name,
                ip,
                total_received,
                total_transmitted,
                rx_stats: None,
                tx_stats: None,
            }
        }
    }
    pub struct Network {
        pub interfaces: Vec<Interface>,
    }
    impl Network {
        pub fn new() -> Network {
            Network {
                interfaces: Vec::new(),
            }
        }

        pub fn initialize(&mut self) {
            // Obtenir la liste des interfaces réseau
            let networks = Networks::new_with_refreshed_list();

            for (interface_name, data) in &networks {
                let ip = Ipv4Addr::new(0, 0, 0, 0); // Placeholder, update with actual IP later
                let name = interface_name.clone();
                let mut interface =
                    Interface::new(name, ip, data.total_received(), data.total_transmitted());

                interface.rx_stats = Some(RxStats {
                    bytes: data.received(),
                    packets: data.received(),
                    errs: data.received(),
                    drop: data.received(),
                    fifo: data.received(),
                    frame: data.received(),
                    compressed: data.received(),
                    multicast: data.received(),
                });

                interface.tx_stats = Some(TxStats {
                    bytes: data.transmitted(),
                    packets: data.transmitted(),
                    errs: data.transmitted(),
                    drop: data.transmitted(),
                    fifo: data.transmitted(),
                    colls: data.transmitted(),
                    carrier: data.transmitted(),
                    compressed: data.transmitted(),
                });

                self.interfaces.push(interface);
            }

            // Mettre à jour les adresses IP des interfaces réseau
            let pnet_interfaces = pnet::datalink::interfaces();
            for pnet_interface in &pnet_interfaces {
                for ip_network in &pnet_interface.ips {
                    if let pnet::ipnetwork::IpNetwork::V4(ipv4_network) = ip_network {
                        for interface in &mut self.interfaces {
                            if interface.name == pnet_interface.name {
                                interface.ip = ipv4_network.ip();
                                break;
                            }
                        }
                    }
                }
            }
        }

        pub fn get_max_received(&self) -> u64 {
            self.interfaces
                .iter()
                .map(|i| i.total_received)
                .max()
                .unwrap_or(1)
        }

        pub fn get_max_transmitted(&self) -> u64 {
            self.interfaces
                .iter()
                .map(|i| i.total_transmitted)
                .max()
                .unwrap_or(1)
        }
    }

    pub fn draw_ip_table(ui: &Ui, network: &Network) {
        ui.columns(2, "IP-Address", true);
        ui.text("Interface");
        ui.next_column();
        ui.text("IP");
        ui.next_column();
        ui.separator();
        for interface in &network.interfaces {
            ui.text(format!("{}", interface.name));
            ui.next_column();
            ui.text(format!("{}", interface.ip));
            ui.next_column();
            ui.separator();
        }
        ui.columns(1, "", false);
    }

    pub fn draw_rx_table(ui: &Ui, network: &Network) {
        if let Some(rx_tab) = ui.tab_item("RX") {
            // En-têtes du tableau RX
            ui.separator();
            ui.columns(9, "RXColumns", true);
            ui.text("Interface");
            ui.next_column();
            ui.text("Bytes");
            ui.next_column();
            ui.text("Packets");
            ui.next_column();
            ui.text("Errs");
            ui.next_column();
            ui.text("Drop");
            ui.next_column();
            ui.text("Fifo");
            ui.next_column();
            ui.text("Frame");
            ui.next_column();
            ui.text("Compressed");
            ui.next_column();
            ui.text("Multicast");
            ui.next_column();
            ui.separator();
            for interface in &network.interfaces {
                if let Some(rx_stats) = &interface.rx_stats {
                    ui.text(format!("{}", interface.name));
                    ui.next_column();
                    ui.text(format!("{}", interface.total_received));
                    ui.next_column();
                    ui.text(format!("{}", rx_stats.packets));
                    ui.next_column();
                    ui.text(format!("{}", rx_stats.errs));
                    ui.next_column();
                    ui.text(format!("{}", rx_stats.drop));
                    ui.next_column();
                    ui.text(format!("{}", rx_stats.fifo));
                    ui.next_column();
                    ui.text(format!("{}", rx_stats.frame));
                    ui.next_column();
                    ui.text(format!("{}", rx_stats.compressed));
                    ui.next_column();
                    ui.text(format!("{}", rx_stats.multicast));
                    ui.next_column();
                    ui.separator();
                }
            }
            ui.columns(1, "", false);
            rx_tab.end();
        }
    }

    pub fn draw_tx_table(ui: &Ui, network: &Network) {
        if let Some(tx_tab) = ui.tab_item("TX") {
            // En-têtes du tableau TX
            ui.separator();
            ui.columns(9, "TXColumns", true);
            ui.text("Interface");
            ui.next_column();
            ui.text("Bytes");
            ui.next_column();
            ui.text("Packets");
            ui.next_column();
            ui.text("Errs");
            ui.next_column();
            ui.text("Drop");
            ui.next_column();
            ui.text("Fifo");
            ui.next_column();
            ui.text("Colls");
            ui.next_column();
            ui.text("Compressed");
            ui.next_column();
            ui.text("Carrier");
            ui.next_column();
            ui.separator();
            for interface in &network.interfaces {
                if let Some(tx_stats) = &interface.tx_stats {
                    ui.text(format!("{}", interface.name));
                    ui.next_column();
                    ui.text(format!("{}", interface.total_transmitted));
                    ui.next_column();
                    ui.text(format!("{}", tx_stats.packets));
                    ui.next_column();
                    ui.text(format!("{}", tx_stats.errs));
                    ui.next_column();
                    ui.text(format!("{}", tx_stats.drop));
                    ui.next_column();
                    ui.text(format!("{}", tx_stats.fifo));
                    ui.next_column();
                    ui.text(format!("{}", tx_stats.colls));
                    ui.next_column();
                    ui.text(format!("{}", tx_stats.carrier));
                    ui.next_column();
                    ui.text(format!("{}", tx_stats.compressed));
                    ui.next_column();
                    ui.separator();
                }
            }
            ui.columns(1, "", false);
            tx_tab.end();
        }
    }

    pub fn network_prog(ui: &Ui, show_rx_bar: &mut bool, show_tx_bar: &mut bool, stats: &Network) {
        const MAX: f32 = 1024.0 * 1024.0 * 1024.0 * 2.0; // 2GB en bytes

        fn get_color(value: f32) -> [f32; 4] {
            if value <= MAX / 2.0 {
                [0.0, 1.0, 0.0, 1.0] // Vert
            } else if value > MAX / 2.0 && value <= MAX * 2.0 / 3.0 {
                [1.0, 1.0, 0.0, 1.0] // Jaune
            } else {
                [1.0, 0.0, 0.0, 1.0] // Rouge
            }
        }

        ui.text("\n");
        if ui.button("Network-Receiver") {
            *show_rx_bar = !*show_rx_bar;
            *show_tx_bar = false;
        }

        ui.same_line_with_pos(150.0); // Alignez le bouton suivant sur la même ligne
        if ui.button("Network-Transmitter") {
            *show_tx_bar = !*show_tx_bar;
            *show_rx_bar = false;
        }

        ui.separator();

        if *show_rx_bar {
            for stat in &stats.interfaces {
                let rx = stat.total_received as f32 / MAX;
                let color = get_color(stat.total_received as f32);
                let (r, g, b) = (color[0], color[1], color[2]);
                ui.text(&stat.name);

                // Dessiner la barre de progression
                let draw_list = ui.get_window_draw_list();
                let pos = ui.cursor_screen_pos();
                let size = [300.0, 24.0];
                let mut fill_end = pos[0] + size[0] * rx;

                // Dessiner la barre de fond (blanche)
                draw_list
                    .add_rect(pos, [pos[0] + size[0], pos[1] + size[1]], ImColor32::WHITE)
                    .build();

                // Dessiner la barre remplie
                if fill_end > size[0] {
                    fill_end = pos[0] + size[0];
                }
                if rx > 0.0 {
                    draw_list
                        .add_rect(
                            pos,
                            [fill_end, pos[1] + size[1]],
                            ImColor32::from_rgb_f32s(r, g, b),
                        )
                        .build();
                }
                ui.invisible_button("progress_bar", size);

                let label = format!("{}", convert_bytes_to_any(MAX as u64));
                ui.same_line_with_spacing(0.0, 10.0); // Pour afficher à droite de la barre
                ui.text(&label);
                ui.text("\n");
            }
        }

        if *show_tx_bar {
            for stat in &stats.interfaces {
                let tx = stat.total_transmitted as f32 / MAX;
                let color = get_color(stat.total_transmitted as f32);
                let (r, g, b) = (color[0], color[1], color[2]);
                ui.text(&stat.name);

                // Dessiner la barre de progression
                let draw_list = ui.get_window_draw_list();
                let pos = ui.cursor_screen_pos();
                let size = [300.0, 24.0];
                let fill_end = pos[0] + size[0] * tx;

                // Dessiner la barre de fond (blanche)
                draw_list
                    .add_rect(pos, [pos[0] + size[0], pos[1] + size[1]], ImColor32::WHITE)
                    .build();

                // Dessiner la barre remplie
                if tx > 0.0 {
                    draw_list
                        .add_rect(
                            pos,
                            [fill_end, pos[1] + size[1]],
                            ImColor32::from_rgb_f32s(r, g, b),
                        )
                        .build();
                }

                ui.invisible_button("progress_bar", size);

                let label = format!("{}", convert_bytes_to_any(MAX as u64));
                ui.same_line_with_spacing(0.0, 10.0); // Pour afficher à droite de la barre
                ui.text(&label);
                ui.text("\n");
            }
        }
    }
}
