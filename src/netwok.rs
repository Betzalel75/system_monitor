pub mod network {
    use std::net::Ipv4Addr;
    use sysinfo::Networks;
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
                let mut interface = Interface::new(name, ip, data.total_received(), data.total_transmitted());

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

    }
}
