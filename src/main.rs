extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate sdl2;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use graphs::graph;
use imgui::*;
use imgui_opengl_renderer::Renderer;
use imgui_sdl2::ImguiSdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

use sysinfo::System;
use system_monitor::*;

#[tokio::main]
async fn main() {
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

    // Créez des instances de GraphData pour chaque type de graphique
    let cpu_graph = Arc::new(Mutex::new(graph::GraphData::new(
        100,
        Duration::from_secs_f32(1.0),
    )));
    let fan_graph = Arc::new(Mutex::new(graph::GraphData::new(
        100,
        Duration::from_secs_f32(1.0),
    )));
    let temp_graph = Arc::new(Mutex::new(graph::GraphData::new(
        100,
        Duration::from_secs_f32(1.0),
    )));

    let system = Arc::new(Mutex::new(System::new_all()));
    let mut selected_pids = HashSet::new();
    let system_clone = system.clone();
    tokio::spawn(async move {
        loop {
            {
                let mut sys = system_clone.lock().unwrap();
                sys.refresh_processes();
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    let computer = Computer::new();
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

        ui.window("== Memory and Processes ==")
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
                // Table des Processuses
                ui.text("\n");
                let binding = system.clone();
                let mut system = binding.lock().unwrap();
                draw_process_table(ui, &mut system, &mut selected_pids);
            });

        ui.window("== System ==")
            .size([600.0, 370.0], Condition::FirstUseEver)
            .position([10.0, 10.0], Condition::FirstUseEver)
            .build(|| {
                // Code pour la fenêtre Système
                computer.display(ui);
                ui.window("Graphics")
                    .size([600.0, 240.0], Condition::FirstUseEver)
                    .position([10.0, 140.0], Condition::FirstUseEver)
                    .build(|| {
                        // Appeler adjust_intervals chaque fois que le FPS est modifié
                        adjust_intervals(cpu_graph.clone(), fan_graph.clone(), temp_graph.clone());

                        if let Some(tab_bar) = ui.tab_bar("Performance Tabs") {
                            if let Some(tab) = ui.tab_item("CPU") {
                                let binding = cpu_graph.clone();
                                let mut cpu_graph = binding.lock().unwrap();
                                if !cpu_graph.is_paused && cpu_graph.last_update.elapsed() >= cpu_graph.update_interval {
                                    let cpu_usage = Cpu::get_cpu_usage();
                                    cpu_graph.last_update = Instant::now();
                                    {
                                        cpu_graph.update(cpu_usage);
                                    }
                                }
                                let hover = format!("CPU Usage: #%");
                                ui.checkbox("Pause Animation", &mut cpu_graph.is_paused);
                                ui.slider("FPS", 1.0, 60.0, &mut cpu_graph.fps);
                                ui.slider("Y Scale", 1.0, 10.0, &mut cpu_graph.y_scale);
                                cpu_graph.draw_graph(&ui, "CPU Usage", &hover);
                                tab.end();
                            }

                            if let Some(tab) = ui.tab_item("Fan") {
                                let hover: &str = "RPM: #";
                                let binding = fan_graph.clone();
                                let mut fan_graph = binding.lock().unwrap();
                                if !fan_graph.is_paused
                                    && fan_graph.last_update.elapsed() >= fan_graph.update_interval
                                {
                                    let fan_info_list = Cpu::get_all_fan_info().unwrap();
                                    fan_graph.last_update = Instant::now();
                                    {
                                        fan_graph.update(fan_info_list[0].rpm.unwrap_or(0) as f32);
                                    }
                                }

                                ui.checkbox("Pause Animation", &mut fan_graph.is_paused);
                                ui.slider("FPS", 1.0, 60.0, &mut fan_graph.fps);
                                ui.slider("Y Scale", 1.0, 10.0, &mut fan_graph.y_scale);
                                fan_graph.draw_graph(&ui, "Fan Speed", &hover);
                                tab.end();
                            }

                            if let Some(tab) = ui.tab_item("Thermal") {
                                let binding = temp_graph.clone();
                                let mut temp_graph = binding.lock().unwrap();

                                if !temp_graph.is_paused
                                    && temp_graph.last_update.elapsed()
                                        >= temp_graph.update_interval
                                {
                                    let cpu_temperature = Cpu::get_cpu_temperatures();
                                    temp_graph.last_update = Instant::now();
                                    {
                                        temp_graph.update(cpu_temperature);
                                    }
                                }

                                let hover = format!("CPU Temperature: #°C",);

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

        ui.window("== Network ==")
            .size([1260.0, 310.0], Condition::FirstUseEver)
            .position([10.0, 390.0], Condition::FirstUseEver)
            .build(|| {
                // Code pour la fenêtre Réseau
                if ui.button("IP-Address") {
                    show_ip = !show_ip;
                }
                if show_ip {
                    ui.separator();
                   draw_ip_table(ui, &network);
                }
                ui.text("\n");
                if let Some(tab_bar) = ui.tab_bar("Network") {
                    draw_rx_table(ui, &network);
                    draw_tx_table(ui, &network);
                    tab_bar.end();
                }
                // Barres de Progressions
                network_prog(ui, &mut show_rx_bar, &mut show_tx_bar, &network);
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

