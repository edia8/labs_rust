use std::{collections::{HashMap, VecDeque}, sync::mpsc::{Receiver, Sender, channel}, thread, time::Duration};

use eframe::egui;
use eframe::epaint::{PathShape, Pos2, Stroke, Color32};
use egui_extras::{TableBuilder,Column};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, Users};
#[derive(PartialEq)]
enum ViewMode {
    List=0,
    Tree=1,
    Overview=2,
}
#[derive(Clone,Debug)]
struct ProcessData {
    pid: u32,
    name: String,
    ppid: Option<u32>,
    cpu_usage: f32,
    memory: u64,
    path: String,
    username: String
}
#[derive(Clone,Debug)]
struct Stats {
    host_name:Option<String>,
    system_name: Option<String>,
    cpu_architecture: String,
    os_version: Option<String>,
    global_cpu_usage: f32,
    kernel_version: Option<String>,
    kernel_long_version: String,
    cores: u32,
    distribution_id: String,
    used_memory: u64,
    total_memory: u64,
    used_swap: u64,
    total_swap: u64,
    uptime: u64,
    cpu_history: VecDeque<f32>,
    mem_history: VecDeque<f32>,
    swap_history: VecDeque<f32>,
}
#[derive(Clone,Debug)]
struct Overall_Data {
    procese: Vec<ProcessData>,
    stats: Stats
}

struct TaskManager {
    rx: Receiver<Overall_Data>,
    //ui type shit
    view: ViewMode,
    cur_data: Option<Overall_Data>,

    //tree ui
    radacini: Vec<u32>,
    tree_cache: HashMap<u32, Vec<u32>>,
    process_map: HashMap<u32,ProcessData>,

    // graph toggles
    show_cpu_graph: bool,
    show_mem_graph: bool,
    show_swap_graph: bool,
}



impl TaskManager {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {

        let ctx = cc.egui_ctx.clone();
        ctx.set_visuals(egui::Visuals::dark());

        let (tx,rx) = channel();
        thread::spawn(move || {
            backend(tx,ctx);
        });

        Self {
            rx,
            view: ViewMode::Overview,
            cur_data: None,
            radacini: Vec::new(),
            tree_cache: HashMap::new(),
            process_map: HashMap::new(),
            show_cpu_graph: false,
            show_mem_graph: false,
            show_swap_graph: false,
        }
    }

    fn prepare_data(&mut self, data: Overall_Data) {
        self.process_map.clear();
        self.process_map.reserve(data.procese.len());
        for p in &data.procese {
            self.process_map.insert(p.pid, p.clone());
        }
        self.tree_cache.clear();
        self.radacini.clear();
        for p in &data.procese {
            if let Some(ppid) = p.ppid {
                if self.process_map.contains_key(&ppid) {
                    self.tree_cache.entry(ppid).or_default().push(p.pid);
                } else { //parinte fantoma
                self.radacini.push(p.pid);
                }
            } else {
                self.radacini.push(p.pid);
            }
        }
        self.cur_data = Some(data);
    }
    fn render_list(&self, ui: &mut egui::Ui, processes: &[ProcessData]) {
        let text_height = egui::TextStyle::Body.resolve(ui.style()).size + 2.0;

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(60.0))
            .column(Column::initial(150.0))
            .column(Column::auto().at_least(60.0))
            .column(Column::auto().at_least(80.0))
            .column(Column::initial(80.0))
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| { ui.strong("PID"); });
                header.col(|ui| { ui.strong("Name"); });
                header.col(|ui| { ui.strong("CPU %"); });
                header.col(|ui| { ui.strong("Mem"); });
                header.col(|ui| { ui.strong("User"); });
                header.col(|ui| { ui.strong("Path"); });
            })
            .body(| body| {
                body.rows(text_height, processes.len(), |mut row| {
                    let p = &processes[row.index()];
                    row.col(|ui| { ui.label(p.pid.to_string()); });
                    row.col(|ui| { ui.label(&p.name); });
                    row.col(|ui| { ui.label(format!("{:.1}", p.cpu_usage)); });
                    row.col(|ui| { ui.label(format!("{:.2} MB", bytes_to_mb(p.memory))); });
                    row.col(|ui| { ui.label(&p.username); });
                    row.col(|ui| { ui.label(&p.path); });
                });
            });
    }

    fn render_tree(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for &pid in &self.radacini {
                self.render_tree_node(ui, pid);
            }
        });
    }

    fn render_tree_node(&self, ui: &mut egui::Ui, pid: u32) {
        if let Some(p) = self.process_map.get(&pid) {
            let children = self.tree_cache.get(&pid);
            let has_children = children.is_some() && !children.unwrap().is_empty();

            let label = format!("{} [{}] ({:.1}%)", p.name, pid, p.cpu_usage);

            if has_children {
                egui::CollapsingHeader::new(label)
                    .id_salt(pid)
                    .show(ui, |ui| {
                        if let Some(kids) = children {
                            for &kid_pid in kids {
                                self.render_tree_node(ui, kid_pid);
                            }
                        }
                    });
            } else {
                ui.label(format!("   {}", label));
            }
        }
    }

    fn draw_circle_gauge(&self, ui: &mut egui::Ui, value_percent: f32, color: Color32, label: &str) -> egui::Response {
        let size = 120.0;
        let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
        
        let center = rect.center();
        let radius = size / 2.0 - 5.0;
        let start_angle = std::f32::consts::PI * 0.75; // Start at bottom left
        let end_angle = std::f32::consts::PI * 2.25;   // End at bottom right
        let total_angle = end_angle - start_angle;
        
        // Animated/Filled angle
        let current_angle = start_angle + (total_angle * (value_percent / 100.0));

        let painter = ui.painter();

        // 1. Background Arc (Dimmed)
        let bg_color = color.linear_multiply(0.2);
        self.stroke_arc(painter, center, radius, start_angle, end_angle, Stroke::new(4.0, bg_color));

        // 2. Foreground Arc (Bright)
        self.stroke_arc(painter, center, radius, start_angle, current_angle, Stroke::new(6.0, color));

        // 3. Center Text
        painter.text(
            center - egui::vec2(0.0, 10.0),
            egui::Align2::CENTER_CENTER,
            format!("{:.1}%", value_percent),
            egui::FontId::proportional(22.0),
            Color32::WHITE,
        );
        painter.text(
            center + egui::vec2(0.0, 15.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::monospace(12.0),
            color,
        );

        response
    }

    // --- HELPER: Draw a Cyberpunk Graph ---
    fn draw_graph(&self, ui: &mut egui::Ui, history: &VecDeque<f32>, color: Color32) {
        let height = 60.0;
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
        let painter = ui.painter();

        // Background box
        painter.rect_filled(rect, 5.0, Color32::from_black_alpha(100));
        painter.rect_stroke(rect, 5.0, Stroke::new(1.0, color.linear_multiply(0.3)),egui::StrokeKind::Inside);

        if history.len() < 2 { return; }

        let points: Vec<Pos2> = history.iter().enumerate().map(|(i, &val)| {
            let x = rect.min.x + (i as f32 / (history.len() - 1) as f32) * rect.width();
            // Invert Y (0 is top)
            let y = rect.max.y - (val / 100.0) * rect.height(); 
            Pos2::new(x, y)
        }).collect();

        // Draw Line
        painter.add(PathShape::line(points, Stroke::new(2.0, color)));
    }

    // --- HELPER: Arc Math ---
    fn stroke_arc(&self, painter: &egui::Painter, center: Pos2, radius: f32, start_angle: f32, end_angle: f32, stroke: Stroke) {
        let n_points = 30;
        let points: Vec<Pos2> = (0..=n_points).map(|i| {
            let angle = start_angle + (end_angle - start_angle) * (i as f32 / n_points as f32);
            Pos2::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            )
        }).collect();
        painter.add(PathShape::line(points, stroke));
    }

    // --- MAIN RENDER FUNCTION ---
    fn render_overview(&mut self, ui: &mut egui::Ui, global: &Stats) {
        // Aesthetic Constants
        let color_cpu = Color32::from_rgb(0, 255, 255);   // Cyan
        let color_mem = Color32::from_rgb(255, 0, 255);   // Magenta
        let color_swap = Color32::from_rgb(255, 165, 0);  // Orange

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(10.0);

            // 1. SYSTEM IDENTITY (Neofetch Style)
            egui::Frame::group(ui.style())
                .fill(Color32::from_black_alpha(50))
                .stroke(Stroke::new(1.0, Color32::from_white_alpha(30)))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.heading(egui::RichText::new("SYSTEM IDENTITY").monospace().strong().color(Color32::WHITE));
                    ui.add_space(10.0);
                    
                    egui::Grid::new("sys_info_grid")
                        .spacing([40.0, 12.0])
                        .striped(false)
                        .show(ui, |ui| {
                            let label_fmt = |s: &str| egui::RichText::new(s).color(Color32::from_gray(180)).size(14.0);
                            let val_fmt = |s: &str| egui::RichText::new(s).code().color(Color32::WHITE).size(14.0);

                            ui.label(label_fmt("🖥️  Host")); ui.label(val_fmt(global.host_name.as_ref().unwrap().as_str())); 
                            ui.label(label_fmt("⚙️  Arch")); ui.label(val_fmt(&global.cpu_architecture)); 
                            ui.end_row();

                            ui.label(label_fmt("💿  OS")); ui.label(val_fmt(&format!("{} {}", global.system_name.as_ref().unwrap(), global.os_version.as_ref().unwrap())));
                            ui.label(label_fmt("🧠  Cores")); ui.label(val_fmt(global.cores.to_string().as_str()));
                            ui.end_row();

                            ui.label(label_fmt("🐧  Kernel")); ui.label(val_fmt(global.kernel_version.as_ref().unwrap().as_str()));
                            ui.label(label_fmt("⏱️  Uptime")); ui.label(val_fmt(global.uptime.to_string().as_str())); 
                            ui.end_row();
                        });
                });

            ui.add_space(30.0);

            // 2. TELEMETRY DASHBOARD (Circular Gauges)
            
            // CPU Section
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("PROCESSOR").strong().color(color_cpu));
                    if self.draw_circle_gauge(ui, global.global_cpu_usage, color_cpu, "CPU").clicked() {
                        self.show_cpu_graph = !self.show_cpu_graph;
                    }
                });
                if self.show_cpu_graph {
                    ui.add_space(20.0);
                    self.draw_graph(ui, &global.cpu_history, color_cpu);
                }
            });
            ui.add_space(20.0);

            // Memory Section
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let mem_perc = (global.used_memory as f32 / global.total_memory as f32) * 100.0;
                    ui.label(egui::RichText::new("MEMORY MOD").strong().color(color_mem));
                    if self.draw_circle_gauge(ui, mem_perc, color_mem, "RAM").clicked() {
                        self.show_mem_graph = !self.show_mem_graph;
                    }
                    ui.label(egui::RichText::new(format!("{:.1}/{:.1} GB", 
                        bytes_to_gb(global.used_memory), bytes_to_gb(global.total_memory)))
                        .size(10.0).weak());
                });
                if self.show_mem_graph {
                    ui.add_space(20.0);
                    self.draw_graph(ui, &global.mem_history, color_mem);
                }
            });
            ui.add_space(20.0);

            // Swap Section
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let swap_perc = if global.total_swap > 0 {
                        (global.used_swap as f32 / global.total_swap as f32) * 100.0
                    } else { 0.0 };
                    
                    ui.label(egui::RichText::new("VIRTUAL MEM").strong().color(color_swap));
                    if self.draw_circle_gauge(ui, swap_perc, color_swap, "SWAP").clicked() {
                        self.show_swap_graph = !self.show_swap_graph;
                    }
                    ui.label(egui::RichText::new(format!("{:.1}/{:.1} GB", 
                        bytes_to_gb(global.used_swap), bytes_to_gb(global.total_swap)))
                        .size(10.0).weak());
                });
                if self.show_swap_graph {
                    ui.add_space(20.0);
                    self.draw_graph(ui, &global.swap_history, color_swap);
                }
            });
        });
    }
}

fn bytes_to_mb(bytes: u64) -> f64 { bytes as f64 / 1_048_576.0 }
fn bytes_to_gb(bytes: u64) -> f64 { bytes as f64 / 1_073_741_824.0 }

fn backend(tx: Sender<Overall_Data>, ctx: egui::Context) {
        let mut system = System::new();
        let mut users = Users::new_with_refreshed_list();
        let mut cpu_history = VecDeque::from(vec![0.0; 60]); // Init with zeros
        let mut mem_history = VecDeque::from(vec![0.0; 60]);
        let mut swap_history = VecDeque::from(vec![0.0; 60]);
        loop {
            system.refresh_cpu_all();
            system.refresh_memory();
            system.refresh_processes_specifics(ProcessesToUpdate::All, true,ProcessRefreshKind::nothing()
                                                                                                                .with_cpu()
                                                                                                                .with_exe(sysinfo::UpdateKind::Always)
                                                                                                                .with_user(sysinfo::UpdateKind::Always));
            
            users.refresh();
            let mut processes: Vec<ProcessData> = system.processes().iter().map(|(pid, proc)| {
                let usr = if let Some(uid) = proc.user_id() {
                    users.get_user_by_id(uid).map(|u|u.name().to_string()).unwrap_or_else(||"Unknown".to_string())
                } else {
                    "system".to_string()
                };
                let path_str:String = if let Some(pth) = proc.exe() {
                    pth.to_string_lossy().to_string()
                } else {
                    "Unknown".to_string()
                };
                let proc_name = proc.name().to_string_lossy().to_string();

                ProcessData { pid: pid.as_u32(), name: proc_name, ppid: proc.parent().map(|p|p.as_u32()), cpu_usage: proc.cpu_usage()/system.cpus().len() as f32, memory: proc.memory(), path: path_str, username: usr }
            }).collect();

            processes.sort_by(|a,b| b.cpu_usage.total_cmp(&a.cpu_usage));

                cpu_history.push_back(system.global_cpu_usage());
    if cpu_history.len() > 60 { cpu_history.pop_front(); }

    mem_history.push_back((system.used_memory() as f32 / system.total_memory() as f32) * 100.0);
    if mem_history.len() > 60 { mem_history.pop_front(); }
    
    let swap_p = if system.total_swap() > 0 { (system.used_swap() as f32 / system.total_swap() as f32) * 100.0 } else { 0.0 };
    swap_history.push_back(swap_p);
    if swap_history.len() > 60 { swap_history.pop_front(); }

            let stat = Stats {
                host_name: System::host_name(),
                system_name: System::name(),
                cpu_architecture: System::cpu_arch(),
                os_version: System::long_os_version(),
                global_cpu_usage: system.global_cpu_usage(),
                kernel_version: System::kernel_version(),
                kernel_long_version: System::kernel_long_version(),
                cores: system.cpus().len() as u32,
                distribution_id: System::distribution_id(),
                used_memory: system.used_memory(),
                total_memory: system.total_memory(),
                used_swap: system.used_swap(),
                total_swap: system.total_swap(),
                uptime: System::uptime(),
                cpu_history: cpu_history.clone(),
                mem_history: mem_history.clone(),
                swap_history: swap_history.clone()
            };
            ctx.request_repaint();
            if tx.send(Overall_Data { procese: processes, stats: stat }).is_err() {
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }
    }

impl eframe::App for TaskManager {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        if let Ok(data) = self.rx.try_recv() {
            self.prepare_data(data);
        }

 egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Process Monitor");
                ui.separator();
                egui::ComboBox::from_id_salt("view_mode")
                    .selected_text(match self.view {
                        ViewMode::List => "List View",
                        ViewMode::Tree => "Tree View",
                        ViewMode::Overview => "Overview",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.view, ViewMode::List, "List View");
                        ui.selectable_value(&mut self.view, ViewMode::Tree, "Tree View");
                        ui.selectable_value(&mut self.view, ViewMode::Overview, "Overview");
                    });
            });
            ui.separator();
            if self.cur_data.is_some() {
                match self.view {
                    ViewMode::List => {
                        if let Some(data) = &self.cur_data {
                            self.render_list(ui, &data.procese);
                        }
                    }
                    ViewMode::Tree => self.render_tree(ui),
                    ViewMode::Overview => {
                        let stats = self.cur_data.as_ref().map(|d| d.stats.clone());
                        if let Some(stats) = stats {
                            self.render_overview(ui, &stats);
                        }
                    }
                }
            } else {
                ui.centered_and_justified(|ui| ui.spinner());
            }
        });
    }
   }


fn main() -> Result<(),eframe::Error> {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([750.0,750.0])
        .with_min_inner_size([750.0,750.0]);
    let native_options = eframe::NativeOptions{viewport, ..Default::default()};
    eframe::run_native("Rust Process Monitor", native_options, Box::new(|cc| Ok(Box::new(TaskManager::new(cc)))))
}
