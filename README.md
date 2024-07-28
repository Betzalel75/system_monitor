
# System Monitor

This project is a computer system resource and performance monitoring application written in Rust. The application monitors various system components including the CPU, RAM, SWAP, fans, network, etc. The graphical interface is implemented using Dear ImGui.

## Features

- **CPU Monitoring**: Real-time CPU usage display.
- **Memory Monitoring**: Display of information about RAM, SWAP, and storage.
- **Network Monitoring**: Display of receive and transmit statistics for network interfaces.
- **Thermal Monitoring**: Display of component temperatures.
- **Fan Monitoring**: Display of fan speeds.

## Prerequisites

- [Rust](https://www.rust-lang.org/)
- [SDL2](https://www.libsdl.org/)
- [Dear ImGui](https://github.com/ocornut/imgui)
- [GL](https://crates.io/crates/gl)
- [imgui-rs](https://crates.io/crates/imgui)
- [imgui-sdl2](https://crates.io/crates/imgui-sdl2)
- [imgui-opengl-renderer](https://crates.io/crates/imgui-opengl-renderer)
- [sysinfo](https://crates.io/crates/sysinfo)
- [pnet](https://crates.io/crates/pnet)

## Installation

1. Clone the repository:
   ```sh
   git clone https://github.com/Betzalel75/system_monitor.git
   cd system_monitor
   ```

2. Install dependencies:
   ```sh
   cargo build
   ```

## Usage

To run the application:
```sh
cargo run
```

## Project Structure

- `src/main.rs`: Entry point of the application.
- `src/lib.rs`: Module handling.
- `src/graphs.rs`: Module containing functions for graphs.
- `src/memories`: Memory management.
- `src/network.rs`: Network interface management.
- `src/systems.rs`: System and CPU information management.

## Code Examples

### Network Initialization

```rust
extern crate sysinfo;
extern crate pnet;

use std::net::Ipv4Addr;
use sysinfo::{NetworkExt, NetworksExt};
use pnet::datalink;
use pnet::ipnetwork::IpNetwork;

pub struct Interface {
    pub name: String,
    pub ip: Ipv4Addr,
    pub total_received: u64,
    pub total_transmitted: u64,
}

impl Interface {
    pub fn new(name: String, ip: Ipv4Addr, total_received: u64, total_transmitted: u64) -> Interface {
        Interface { name, ip, total_received, total_transmitted }
    }
}

pub struct Network {
    pub interfaces: Vec<Interface>,
}

impl Network {
    pub fn new() -> Network {
        Network { interfaces: Vec::new() }
    }

    pub fn initialize(&mut self) {
        let networks = sysinfo::System::new_all().networks();
        for (interface_name, data) in networks {
            let ip = Ipv4Addr::new(0, 0, 0, 0);
            let name = interface_name.clone();
            let interface = Interface::new(name, ip, data.total_received(), data.total_transmitted());
            self.interfaces.push(interface);
        }

        let interfaces = datalink::interfaces();
        for (interface, net) in interfaces.iter().zip(self.interfaces.iter_mut()) {
            for ip in interface.ips.clone() {
                match ip {
                    IpNetwork::V4(addr) => {
                        if interface.name == net.name {
                            net.ip = addr.ip();
                            break;
                        }
                    },
                    _ => continue,
                }
            }
        }
    }
}
```

### Network Data Display

```rust
ui.window("== Network ==")
    .size([1260.0, 310.0], Condition::FirstUseEver)
    .position([10.0, 390.0], Condition::FirstUseEver)
    .build(|| {
        if let Some(tab) = ui.tab_item("Network") {
            if let Some(network_tab_bar) = ui.tab_bar("Network Tabs") {
                if let Some(rx_tab) = ui.tab_item("RX") {
                    for interface in &network.interfaces {
                        ui.text(format!("Interface: {}", interface.name));
                        ui.text(format!("IP: {}", interface.ip));
                        ui.text(format!(
                            "Total Received: {}",
                            convert_bytes_to_any(interface.total_received)
                        ));
                    }
                    rx_tab.end();
                }
                if let Some(tx_tab) = ui.tab_item("TX") {
                    for interface in &network.interfaces {
                        ui.text(format!("Interface: {}", interface.name));
                        ui.text(format!("IP: {}", interface.ip));
                        ui.text(format!(
                            "Total Transmitted: {}",
                            convert_bytes_to_any(interface.total_transmitted)
                        ));
                    }
                    tx_tab.end();
                }
                network_tab_bar.end();
            }
            tab.end();
        }
    });
```

## Contributions

Contributions to the project are welcome. To contribute, follow these steps:

1. Fork the repository
2. Create a branch (`git checkout -b feature/add-feature`)
3. Commit your changes (`git commit -am 'Add new feature'`)
4. Push to the branch (`git push origin feature/add-feature`)
5. Create a new Pull Request

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Autor

- 👤 **Betzalel75**: [Profile](https://github.com/Betzalel75)
