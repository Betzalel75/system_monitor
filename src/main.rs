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
                if let Some(tab_bar) = ui.tab_bar("Network") {
                    if let Some(rx_tab) = ui.tab_item("RX") {
                        for interface in &network.interfaces {
                            if let Some(rx_stats) = &interface.rx_stats {
                                ui.text(format!("Interface: {}", interface.name));
                                ui.text(format!("IP: {}", interface.ip));
                                ui.text(format!(
                                    "Total Received: {}",
                                    convert_bytes_to_any(interface.total_received)
                                ));
                                ui.text(format!("Packets: {}", rx_stats.packets));
                                ui.text(format!("Errors: {}", rx_stats.errs));
                                ui.text(format!("Dropped: {}", rx_stats.drop));
                                ui.text(format!("FIFO: {}", rx_stats.fifo));
                                ui.text(format!("Frame: {}", rx_stats.frame));
                                ui.text(format!("Compressed: {}", rx_stats.compressed));
                                ui.text(format!("Multicast: {}", rx_stats.multicast));
                            }
                        }
                        rx_tab.end();
                    }

                    if let Some(tx_tab) = ui.tab_item("TX") {
                        for interface in &network.interfaces {
                            if let Some(tx_stats) = &interface.tx_stats {
                                ui.text(format!("Interface: {}", interface.name));
                                ui.text(format!("IP: {}", interface.ip));
                                ui.text(format!(
                                    "Total Transmitted: {}",
                                    convert_bytes_to_any(interface.total_transmitted)
                                ));
                                ui.text(format!("Packets: {}", tx_stats.packets));
                                ui.text(format!("Errors: {}", tx_stats.errs));
                                ui.text(format!("Dropped: {}", tx_stats.drop));
                                ui.text(format!("FIFO: {}", tx_stats.fifo));
                                ui.text(format!("Collisions: {}", tx_stats.colls));
                                ui.text(format!("Carrier: {}", tx_stats.carrier));
                                ui.text(format!("Compressed: {}", tx_stats.compressed));
                            }
                        }
                        tx_tab.end();
                    }
                    tab_bar.end();
                }
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
