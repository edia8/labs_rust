use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc::{Receiver, Sender, channel},
    thread,
    time::Duration,
};

use eframe::egui;
use eframe::epaint::{Color32, PathShape, Pos2, Stroke};
use egui_extras::{Column, TableBuilder};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, ThreadKind, Users};
#[derive(PartialEq)]
enum ViewMode {
    List = 0,
    Tree = 1,
    Overview = 2,
}
#[derive(PartialEq, Clone, Copy)]
enum SortColumn {
    Pid,
    Name,
    Cpu,
    Memory,
}
#[derive(Clone, Debug)]
struct ProcessData {
    pid: u32,
    name: String,
    ppid: Option<u32>,
    cpu_usage: f32,
    memory: u64,
    path: String,
    username: String,
    is_thread: Option<ThreadKind>,
}
#[derive(Clone, Debug)]
struct Stats {
    host_name: Option<String>,
    system_name: Option<String>,
    cpu_architecture: String,
    os_version: Option<String>,
    global_cpu_usage: f32,
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
#[derive(Clone, Debug)]
struct OverallData {
    procese: Vec<ProcessData>,
    stats: Stats,
}

struct TaskManager {
    rx: Receiver<OverallData>,
    //ui type shit
    view: ViewMode,
    cur_data: Option<OverallData>,
    show_threads: bool,

    //tree ui
    radacini: Vec<u32>,
    tree_cache: HashMap<u32, Vec<u32>>,
    process_map: HashMap<u32, ProcessData>,

    //statisticile
    show_cpu_graph: bool,
    show_mem_graph: bool,
    show_swap_graph: bool,

    //sortare list
    sort_col: SortColumn,
    sort_desc: bool,
}

impl TaskManager {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();
        ctx.set_visuals(egui::Visuals::dark());

        let (tx, rx) = channel();
        thread::spawn(move || {
            backend(tx, ctx);
        });

        Self {
            rx,
            view: ViewMode::Overview,
            cur_data: None,
            show_threads: false,
            radacini: Vec::new(),
            tree_cache: HashMap::new(),
            process_map: HashMap::new(),
            show_cpu_graph: false,
            show_mem_graph: false,
            show_swap_graph: false,
            sort_col: SortColumn::Cpu,
            sort_desc: true,
        }
    }

    fn prepare_data(&mut self, data: OverallData) {
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
                } else {
                    //parinte fantoma
                    self.radacini.push(p.pid);
                }
            } else {
                self.radacini.push(p.pid);
            }
        }
        self.cur_data = Some(data);
    }
    fn render_list(&mut self, ui: &mut egui::Ui) {
        let data = if let Some(d) = &self.cur_data {
            d
        } else {
            return;
        };
        let processes = &data.procese;

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_threads, "Show Threads");
        });

        //mapez proces la indice, sortez numai indicii pt eficienta
        let mut indices: Vec<usize> = processes
            .iter()
            .enumerate()
            .filter(|(_, p)| self.show_threads || p.is_thread.is_none())
            .map(|(i, _)| i)
            .collect();

        let sort_col = self.sort_col;
        let sort_desc = self.sort_desc;

        match sort_col {
            SortColumn::Pid => indices.sort_by(|&a, &b| {
                let cmp = processes[a].pid.cmp(&processes[b].pid);
                if sort_desc { cmp.reverse() } else { cmp }
            }),
            SortColumn::Name => indices.sort_by(|&a, &b| {
                let cmp = processes[a].name.cmp(&processes[b].name);
                if sort_desc { cmp.reverse() } else { cmp }
            }),
            SortColumn::Cpu => indices.sort_by(|&a, &b| {
                let cmp = processes[a].cpu_usage.total_cmp(&processes[b].cpu_usage);
                if sort_desc { cmp.reverse() } else { cmp }
            }),
            SortColumn::Memory => indices.sort_by(|&a, &b| {
                let cmp = processes[a].memory.cmp(&processes[b].memory);
                if sort_desc { cmp.reverse() } else { cmp }
            }),
        }

        let text_height = egui::TextStyle::Body.resolve(ui.style()).size + 2.0;

        //buton
        let mut next_sort_col = sort_col;
        let mut next_sort_desc = sort_desc;

        let header_btn = |ui: &mut egui::Ui,
                          label: &str,
                          col: SortColumn,
                          next_col: &mut SortColumn,
                          next_desc: &mut bool| {
            let is_sorted = sort_col == col;
            let text = if is_sorted {
                format!("{} {}", label, if sort_desc { "⬇" } else { "⬆" })
            } else {
                label.to_string()
            };

            if ui.selectable_label(is_sorted, text).clicked() {
                if is_sorted {
                    *next_desc = !*next_desc;
                } else {
                    *next_col = col;
                    *next_desc = true; //afiseaza default descrescator
                }
            }
        };

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
                header.col(|ui| {
                    header_btn(
                        ui,
                        "PID",
                        SortColumn::Pid,
                        &mut next_sort_col,
                        &mut next_sort_desc,
                    )
                });
                header.col(|ui| {
                    header_btn(
                        ui,
                        "Name",
                        SortColumn::Name,
                        &mut next_sort_col,
                        &mut next_sort_desc,
                    )
                });
                header.col(|ui| {
                    header_btn(
                        ui,
                        "CPU %",
                        SortColumn::Cpu,
                        &mut next_sort_col,
                        &mut next_sort_desc,
                    )
                });
                header.col(|ui| {
                    header_btn(
                        ui,
                        "Mem",
                        SortColumn::Memory,
                        &mut next_sort_col,
                        &mut next_sort_desc,
                    )
                });
                header.col(|ui| {
                    ui.strong("User");
                });
                header.col(|ui| {
                    ui.strong("Path");
                });
            })
            .body(|body| {
                body.rows(text_height, indices.len(), |mut row| {
                    let p = &processes[indices[row.index()]];
                    row.col(|ui| {
                        ui.label(p.pid.to_string());
                    });
                    row.col(|ui| {
                        ui.label(&p.name);
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.1}", p.cpu_usage));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.2} MB", bytes_to_mb(p.memory)));
                    });
                    row.col(|ui| {
                        ui.label(&p.username);
                    });
                    row.col(|ui| {
                        ui.label(&p.path);
                    });
                });
            });

        self.sort_col = next_sort_col;
        self.sort_desc = next_sort_desc;
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
            let has_children;
            if let Some(v) = children {
                has_children = !v.is_empty();
            } else {
                has_children = false;
            }

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

    fn draw_circle_gauge(
        &self,
        ui: &mut egui::Ui,
        value_percent: f32,
        color: Color32,
        label: &str,
    ) -> egui::Response {
        let size = 120.0;
        let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

        let center = rect.center();
        let radius = size / 2.0 - 5.0;
        let start_angle = std::f32::consts::PI * 0.75; //STANGA JOS
        let end_angle = std::f32::consts::PI * 2.25; //DREAPTA SUS
        let total_angle = end_angle - start_angle;

        let current_angle = start_angle + (total_angle * (value_percent / 100.0));

        let painter = ui.painter();

        let bg_color = color.linear_multiply(0.2);
        self.stroke_arc(
            painter,
            center,
            radius,
            start_angle,
            end_angle,
            Stroke::new(4.0, bg_color),
        );

        self.stroke_arc(
            painter,
            center,
            radius,
            start_angle,
            current_angle,
            Stroke::new(6.0, color),
        );

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

    fn stroke_arc(
        &self,
        painter: &egui::Painter,
        center: Pos2,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        stroke: Stroke,
    ) {
        let n_points = 30;
        let points: Vec<Pos2> = (0..=n_points)
            .map(|i| {
                let angle = start_angle + (end_angle - start_angle) * (i as f32 / n_points as f32);
                Pos2::new(
                    center.x + radius * angle.cos(),
                    center.y + radius * angle.sin(),
                )
            })
            .collect();
        painter.add(PathShape::line(points, stroke));
    }

    fn draw_graph(&self, ui: &mut egui::Ui, history: &VecDeque<f32>, color: Color32) {
        let height = 60.0;
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
        let painter = ui.painter();

        //background box
        painter.rect_filled(rect, 5.0, Color32::from_black_alpha(100));
        painter.rect_stroke(
            rect,
            5.0,
            Stroke::new(1.0, color.linear_multiply(0.3)),
            egui::StrokeKind::Inside,
        );

        if history.len() < 2 {
            return;
        }

        let points: Vec<Pos2> = history
            .iter()
            .enumerate()
            .map(|(i, &val)| {
                let x = rect.min.x + (i as f32 / (history.len() - 1) as f32) * rect.width();
                //jos devine sus la grafic
                let y = rect.max.y - (val / 100.0) * rect.height();
                Pos2::new(x, y)
            })
            .collect();

        //linie
        painter.add(PathShape::line(points, Stroke::new(2.0, color)));
    }

    //neofetch
    fn render_overview(&mut self, ui: &mut egui::Ui, global: &Stats) {
        let color_cpu = Color32::from_rgb(0, 255, 255); //turcoaz
        let color_mem = Color32::from_rgb(255, 0, 255); //mov
        let color_swap = Color32::from_rgb(255, 165, 0); //mandariniu

        egui::ScrollArea::vertical().show(ui, |ui| {
            let host;
            let os;
            let version;
            if let Some(s) = global.host_name.as_ref() {
                host = s.as_str();
            } else {
                host = "";
            }
            if let Some(s) = global.system_name.as_ref() {
                os = s.as_ref();
            } else {
                os = "";
            }
            if let Some(s) = global.os_version.as_ref() {
                version = s.to_string();
            } else {
                version = String::default();
            }

            ui.add_space(10.0);


            egui::Frame::group(ui.style())
                .fill(Color32::from_black_alpha(50))
                .stroke(Stroke::new(1.0, Color32::from_white_alpha(30)))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.heading(
                        egui::RichText::new(&global.distribution_id)
                            .monospace()
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.add_space(10.0);

                    egui::Grid::new("sys_info_grid")
                        .spacing([40.0, 12.0])
                        .striped(false)
                        .show(ui, |ui| {
                            let label_fmt = |s: &str| {
                                egui::RichText::new(s)
                                    .color(Color32::from_gray(180))
                                    .size(14.0)
                            };
                            let val_fmt = |s: &str| {
                                egui::RichText::new(s)
                                    .code()
                                    .color(Color32::WHITE)
                                    .size(14.0)
                            };

                            ui.label(label_fmt("💻 Host"));
                            ui.label(val_fmt(host));
                            ui.label(label_fmt("💾 Arch"));
                            ui.label(val_fmt(&global.cpu_architecture));
                            ui.end_row();

                            ui.label(label_fmt("💿 OS"));
                            ui.label(val_fmt(&format!("{} {}", os, version)));
                            ui.label(label_fmt("📊 Cores"));
                            ui.label(val_fmt(global.cores.to_string().as_str()));
                            ui.end_row();

                            ui.label(label_fmt("🐧 Kernel"));
                            ui.label(val_fmt(&global.kernel_long_version));
                            ui.label(label_fmt("⌚ Uptime"));
                            ui.label(val_fmt(global.uptime.to_string().as_str()));
                            ui.end_row();
                            //ui.label(label_fmt("🌐 ID")); ui.label(val_fmt(&global.distribution_id));
                            //ui.end_row();
                        });
                });

            ui.add_space(30.0);

            //cercuri

            //CPU
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("PROCESSOR").strong().color(color_cpu));
                    if self
                        .draw_circle_gauge(ui, global.global_cpu_usage, color_cpu, "CPU")
                        .clicked()
                    {
                        self.show_cpu_graph = !self.show_cpu_graph;
                    }
                });
                if self.show_cpu_graph {
                    ui.add_space(20.0);
                    self.draw_graph(ui, &global.cpu_history, color_cpu);
                }
            });
            ui.add_space(20.0);

            //memory
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let mem_perc = (global.used_memory as f32 / global.total_memory as f32) * 100.0;
                    ui.label(egui::RichText::new("MEMORY MOD").strong().color(color_mem));
                    if self
                        .draw_circle_gauge(ui, mem_perc, color_mem, "RAM")
                        .clicked()
                    {
                        self.show_mem_graph = !self.show_mem_graph;
                    }
                    ui.label(
                        egui::RichText::new(format!(
                            "{:.1}/{:.1} GB",
                            bytes_to_gb(global.used_memory),
                            bytes_to_gb(global.total_memory)
                        ))
                        .size(10.0)
                        .weak(),
                    );
                });
                if self.show_mem_graph {
                    ui.add_space(20.0);
                    self.draw_graph(ui, &global.mem_history, color_mem);
                }
            });
            ui.add_space(20.0);

            //swap
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let swap_perc = if global.total_swap > 0 {
                        (global.used_swap as f32 / global.total_swap as f32) * 100.0
                    } else {
                        0.0
                    };

                    ui.label(
                        egui::RichText::new("VIRTUAL MEM")
                            .strong()
                            .color(color_swap),
                    );
                    if self
                        .draw_circle_gauge(ui, swap_perc, color_swap, "SWAP")
                        .clicked()
                    {
                        self.show_swap_graph = !self.show_swap_graph;
                    }
                    ui.label(
                        egui::RichText::new(format!(
                            "{:.1}/{:.1} GB",
                            bytes_to_gb(global.used_swap),
                            bytes_to_gb(global.total_swap)
                        ))
                        .size(10.0)
                        .weak(),
                    );
                });
                if self.show_swap_graph {
                    ui.add_space(20.0);
                    self.draw_graph(ui, &global.swap_history, color_swap);
                }
            });
        });
    }
}

fn bytes_to_mb(bytes: u64) -> f64 {
    bytes as f64 / 1048576.0
}
fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / 1073741824.0
}

fn backend(tx: Sender<OverallData>, ctx: egui::Context) {
    let mut system = System::new();
    let mut users = Users::new_with_refreshed_list();
    let mut cpu_history = VecDeque::from(vec![0.0; 60]); // Init with zeros
    let mut mem_history = VecDeque::from(vec![0.0; 60]);
    let mut swap_history = VecDeque::from(vec![0.0; 60]);
    loop {
        system.refresh_cpu_all();
        system.refresh_memory();
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_exe(sysinfo::UpdateKind::Always)
                .with_user(sysinfo::UpdateKind::Always)
                .with_memory(),
        );

        users.refresh();
        let mut processes: Vec<ProcessData> = system
            .processes()
            .iter()
            .map(|(pid, proc)| {
                let usr = if let Some(uid) = proc.user_id() {
                    users
                        .get_user_by_id(uid)
                        .map(|u| u.name().to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                } else {
                    "system".to_string()
                };
                let path_str: String = if let Some(pth) = proc.exe() {
                    pth.to_string_lossy().to_string()
                } else {
                    "Unknown".to_string()
                };
                let proc_name = proc.name().to_string_lossy().to_string();
                
                ProcessData {
                    pid: pid.as_u32(),
                    name: proc_name,
                    ppid: proc.parent().map(|p| p.as_u32()),
                    cpu_usage: proc.cpu_usage() / system.cpus().len() as f32,
                    memory: proc.memory(),
                    path: path_str,
                    username: usr,
                    is_thread: proc.thread_kind(),
                }
            })
            .collect();

        processes.sort_by(|a, b| b.cpu_usage.total_cmp(&a.cpu_usage));

        cpu_history.push_back(system.global_cpu_usage());
        if cpu_history.len() > 60 {
            cpu_history.pop_front();
        }

        mem_history.push_back((system.used_memory() as f32 / system.total_memory() as f32) * 100.0);
        if mem_history.len() > 60 {
            mem_history.pop_front();
        }

        let swap_p = if system.total_swap() > 0 {
            (system.used_swap() as f32 / system.total_swap() as f32) * 100.0
        } else {
            0.0
        };
        swap_history.push_back(swap_p);
        if swap_history.len() > 60 {
            swap_history.pop_front();
        }

        let stat = Stats {
            host_name: System::host_name(),
            system_name: System::name(),
            cpu_architecture: System::cpu_arch(),
            os_version: System::long_os_version(),
            global_cpu_usage: system.global_cpu_usage(),
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
            swap_history: swap_history.clone(),
        };
        ctx.request_repaint();
        if tx
            .send(OverallData {
                procese: processes,
                stats: stat,
            })
            .is_err()
        {
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
                        self.render_list(ui);
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

fn main() -> Result<(), eframe::Error> {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([750.0, 750.0])
        .with_min_inner_size([750.0, 750.0]);
    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Rust Process Monitor",
        native_options,
        Box::new(|cc| Ok(Box::new(TaskManager::new(cc)))),
    )
}
