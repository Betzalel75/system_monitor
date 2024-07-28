extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate sdl2;

use std::time::Duration;

use imgui::*;
use imgui_opengl_renderer::Renderer;
use imgui_sdl2::ImguiSdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

use system_monitor::*;

fn main() {
    // Initialize SDL
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    sdl2::hint::set("SDL_HINT_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 0);

    let window = video_subsystem
        .window("Dear ImGui SDL2+OpenGL3 example", 1280, 720)
        .opengl()
        .resizable()
        .allow_highdpi()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&_gl_context).unwrap();
    window.subsystem().gl_set_swap_interval(1).unwrap();

    // Initialize OpenGL loader
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    // Setup Dear ImGui context
    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let mut platform = ImguiSdl2::new(&mut imgui, &window);
    let renderer = Renderer::new(&mut imgui, |s| {
        video_subsystem.gl_get_proc_address(s) as *const _
    });

    let clear_color = [0.0, 0.0, 0.0, 1.0];

    //

    let mut cpu_graph = GraphData::new(100, Duration::from_millis(100));
    let mut fan_graph = GraphData::new(100, Duration::from_millis(100));
    let mut temp_graph = GraphData::new(100, Duration::from_millis(100));
    let mut network = Network::new();
    network.initialize();
    let mut show_ip = false;
    let mut show_rx_bar = false;
    let mut show_tx_bar = false;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            platform.handle_event(&mut imgui, &event);
            if let Event::Quit { .. }
            | Event::Window {
                win_event: sdl2::event::WindowEvent::Close,
                ..
            }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } = event
            {
                break 'running;
            }
        }

        platform.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let ui = imgui.frame();

        // Mettez à jour les données du graphique ici
        cpu_graph.update(Cpu::get_cpu_usage());
        temp_graph.update(Cpu::get_cpu_temperatures());

        ui.window("== Mémoire et processus ==")
            .size([620.0, 370.0], Condition::FirstUseEver)
            .position([650.0, 10.0], Condition::FirstUseEver)
            .build(|| {
                // Code pour la fenêtre Mémoire et processus
                let mut memory = Memory::new();
                // Affichage des informations dans la fenêtre
                memory.get_memory();
                ui.text("Memory Information:");
                ui.text(format!(
                    "Total RAM: {}",
                    convert_bytes_to_any(memory.ram.total_ram)
                ));
                // ui.text(format!("Used RAM: {}", convert_bytes_to_any(memory.ram.used_ram)));
                let free_memory = (memory.ram.used_ram) as f32 / memory.ram.total_ram as f32;
                ProgressBar::new(free_memory)
                    .size([300.0, 24.0])
                    .overlay_text(format!(
                        "Free RAM: {}",
                        convert_bytes_to_any(memory.ram.free_ram)
                    ))
                    .build(&ui);
                ui.text(format!(
                    "Total Swap: {}",
                    convert_bytes_to_any(memory.swap.total_swap)
                ));
                // ui.text(format!("Used Swap: {}", convert_bytes_to_any(memory.swap.used_swap)));
                let free_swap = (memory.swap.used_swap) as f32 / memory.swap.total_swap as f32;
                ProgressBar::new(free_swap)
                    .size([300.0, 24.0])
                    .overlay_text(format!(
                        "Free Swap: {}",
                        convert_bytes_to_any(memory.swap.free_swap)
                    ))
                    .build(&ui);
                ui.text(format!(
                    "Total Storage: {}",
                    convert_bytes_to_any(memory.storage.total_disk)
                ));
                // ui.text(format!("Used Storage: {}", convert_bytes_to_any(memory.storage.used_disk)));
                let free_storage =
                    (memory.storage.used_disk) as f32 / memory.storage.total_disk as f32;
                ProgressBar::new(free_storage)
                    .size([300.0, 24.0])
                    .overlay_text(format!(
                        "Free Storage: {}",
                        convert_bytes_to_any(memory.storage.free_disk)
                    ))
                    .build(&ui);
                ui.text("\n");
                ui.separator();
            });

        ui.window("== Système ==")
            .size([600.0, 370.0], Condition::FirstUseEver)
            .position([10.0, 10.0], Condition::FirstUseEver)
            .build(|| {
                // Code pour la fenêtre Système
                let computer = Computer::new();
                // Affichage des informations dans la fenêtre
                ui.text("System Information:");
                ui.text(format!("Computer Name: {}", computer.hostname));
                ui.text(format!("User Name: {}", computer.username));
                ui.text(format!("OS Info: {}", computer.os_info));
                ui.text(format!("CPU Info: {}", computer.cpu_info));
                ui.text(format!("CPU Core Count: {}", computer.cpu_core_count));
                ui.text("\n");
                ui.window("Graphics")
                    .size([600.0, 240.0], Condition::FirstUseEver)
                    .position([10.0, 140.0], Condition::FirstUseEver)
                    .build(|| {
                        // Code pour la fenêtre Graphics
                        if let Some(tab_bar) = ui.tab_bar("Performance Tabs") {
                            if let Some(tab) = ui.tab_item("CPU") {
                                let hover = format!("CPU Usage: {:.2}%", Cpu::get_cpu_usage());
                                // ui.text(&hover);
                                ui.checkbox("Pause Animation", &mut cpu_graph.is_paused);
                                ui.slider("FPS", 1.0, 60.0, &mut cpu_graph.fps);
                                ui.slider("Y Scale", 1.0, 10.0, &mut cpu_graph.y_scale);
                                cpu_graph.draw_graph(&ui, "CPU Usage", &hover);
                                tab.end();
                            }

                            if let Some(tab) = ui.tab_item("Fan") {
                                let fan_info = Cpu::get_all_fan_info().unwrap_or_default();
                                let mut hover = String::new();
                                for fan in fan_info {
                                    fan_graph.update(fan.rpm.unwrap_or(0) as f32);
                                    hover = format!("RPM: {}", fan.rpm.unwrap_or(0));
                                    // ui.text(format!(
                                    //     "RPM: {}, Min RPM: {}, Max RPM: {}, State: {}",
                                    //     fan.rpm.unwrap_or(0),
                                    //     fan.min_rpm.unwrap_or(0),
                                    //     fan.max_rpm.unwrap_or(0),
                                    //     fan.state.unwrap_or("Unknown".to_string())
                                    // ));
                                }
                                ui.checkbox("Pause Animation", &mut fan_graph.is_paused);
                                ui.slider("FPS", 1.0, 60.0, &mut fan_graph.fps);
                                ui.slider("Y Scale", 1.0, 10.0, &mut fan_graph.y_scale);
                                fan_graph.draw_graph(&ui, "Fan Speed", &hover);
                                tab.end();
                            }

                            if let Some(tab) = ui.tab_item("Thermal") {
                                let hover = format!(
                                    "CPU Temperature: {:.2}°C",
                                    Cpu::get_cpu_temperatures()
                                );
                                // ui.text(&hover);
                                ui.checkbox("Pause Animation", &mut temp_graph.is_paused);
                                ui.slider("FPS", 1.0, 60.0, &mut temp_graph.fps);
                                ui.slider("Y Scale", 1.0, 10.0, &mut temp_graph.y_scale);
                                temp_graph.draw_graph(&ui, "Temperature", &hover);
                                tab.end();
                            }

                            tab_bar.end();
                        }
                    });
            });

        ui.window("== Réseau ==")
            .size([1260.0, 310.0], Condition::FirstUseEver)
            .position([10.0, 390.0], Condition::FirstUseEver)
            .build(|| {
                // Code pour la fenêtre Réseau
                if ui.button("IP-Address"){
                    show_ip = !show_ip;
                }
                if show_ip {
                    ui.separator();
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
                    ui.columns(1, "", true);
                }
                ui.text("\n");
                if let Some(tab_bar) = ui.tab_bar("Network") {
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
                                ui.text(format!("{}",interface.total_received));
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
                        ui.columns(1, "", true);
                        rx_tab.end();
                    }

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
                                ui.text(format!("{}",interface.total_transmitted));
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
                        ui.columns(1, "", true);
                        tx_tab.end();
                    }
                    tab_bar.end();
                }
                // Barres de Progressions
                network_prog(ui, &mut show_rx_bar,&mut show_tx_bar, &network);
            });

        platform.prepare_render(&ui, &window);
        unsafe {
            gl::Viewport(0, 0, 1280, 720);
            gl::ClearColor(
                clear_color[0],
                clear_color[1],
                clear_color[2],
                clear_color[3],
            );
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        renderer.render(&mut imgui);
        window.gl_swap_window();
    }
}

/*
fn network_prog(ui: &Ui, show_rx_bar: &mut bool, show_tx_bar: &mut bool, stats: &Network) {
    const MAX: f32 = 1024.0 * 1024.0 * 1024.0 * 2.0; // 2GB en bytes

    // Fonction pour déterminer la couleur de la barre de progression
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
            let rx: f32 = stat.total_received as f32 / MAX;
            let _color = get_color(stat.total_received as f32);
            ui.text(&stat.name);
            ProgressBar::new(rx)
                .size([300.0, 24.0])
                .overlay_text(format!("{}",
                    convert_bytes_to_any(stat.total_received)
                ))
                .build(&ui);
            let label = format!(" {}", convert_bytes_to_any(MAX as u64));
            ui.same_line_with_spacing(0.0, 10.0); // Pour afficher à droite de la barre
            ui.text(&label);
            ui.text("\n");
        }
    }

    if *show_tx_bar {
        for stat in &stats.interfaces {
            let tx = stat.total_transmitted as f32 / MAX;
            ui.text(&stat.name);
            ProgressBar::new(tx)
                .size([300.0, 24.0])
                .overlay_text(format!("{}",
                    convert_bytes_to_any(stat.total_transmitted)
                ))
                .build(&ui);
            let label = format!(" {}", convert_bytes_to_any(MAX as u64));
            ui.same_line_with_spacing(0.0, 10.0); // Pour afficher à droite de la barre
            ui.text(&label);
            ui.text("\n");
        }
    }
}
*/

fn network_prog(ui: &Ui, show_rx_bar: &mut bool, show_tx_bar: &mut bool, stats: &Network) {
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
            let (r,g,b) = (color[0] , color[1] , color[2]);
            ui.text(&stat.name);

            // Dessiner la barre de progression
            let draw_list = ui.get_window_draw_list();
            let pos = ui.cursor_screen_pos();
            let size = [300.0, 24.0];
            let mut fill_end = pos[0] + size[0] * rx;

            // Dessiner la barre de fond (blanche)
            draw_list.add_rect(
                pos,
                [pos[0] + size[0], pos[1] + size[1]],
                ImColor32::WHITE,
            ).build();

            // Dessiner la barre remplie
            if fill_end > size[0] {
                fill_end = pos[0] + size[0];
            }
            if rx > 0.0 {
                draw_list.add_rect(
                    pos,
                    [fill_end, pos[1] + size[1]],
                    ImColor32::from_rgb_f32s(r, g, b),
                ).build();
            }
            ui.invisible_button("progress_bar", size);

            let label = format!("{}",
                convert_bytes_to_any(MAX as u64)
            );
            ui.same_line_with_spacing(0.0, 10.0); // Pour afficher à droite de la barre
            ui.text(&label);
            ui.text("\n");
        }
    }

    if *show_tx_bar {
        for stat in &stats.interfaces {
            let tx = stat.total_transmitted as f32 / MAX;
            let color = get_color(stat.total_transmitted as f32);
            let (r,g,b) = (color[0] , color[1] , color[2]);
            ui.text(&stat.name);

            // Dessiner la barre de progression
            let draw_list = ui.get_window_draw_list();
            let pos = ui.cursor_screen_pos();
            let size = [300.0, 24.0];
            let fill_end = pos[0] + size[0] * tx;

            // Dessiner la barre de fond (blanche)
            draw_list.add_rect(
                pos,
                [pos[0] + size[0], pos[1] + size[1]],
                ImColor32::WHITE,
            ).build();

            // Dessiner la barre remplie
            if tx > 0.0 {
                draw_list.add_rect(
                    pos,
                    [fill_end, pos[1] + size[1]],
                    ImColor32::from_rgb_f32s(r, g, b),
                ).build();
            }

            ui.invisible_button("progress_bar", size);

            let label = format!("{}",
                convert_bytes_to_any(MAX as u64)
            );
            ui.same_line_with_spacing(0.0, 10.0); // Pour afficher à droite de la barre
            ui.text(&label);
            ui.text("\n");
        }
    }
}

