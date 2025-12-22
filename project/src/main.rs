use std::{collections::{HashMap, HashSet}, sync::mpsc::{Receiver, Sender, channel}, thread, time::Duration};

use eframe::egui;
use egui_extras::{TableBuilder,Column};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, Users};
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
    uptime: u64
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
    process_map: HashMap<u32,ProcessData>
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
            process_map: HashMap::new()
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
        let mut parinti: HashSet<u32> = HashSet::new();
        let mut toate_nodurile: HashSet<u32> = HashSet::new();
        let mut frunze_to_parinti: HashMap<u32,u32> = HashMap::new();
        for p in &data.procese {
            if let Some(ppid) = p.ppid {
                parinti.insert(ppid);
                toate_nodurile.insert(ppid);
                frunze_to_parinti.insert(p.pid, ppid);
                if self.process_map.contains_key(&ppid) {
                    self.tree_cache.entry(ppid).or_default().push(p.pid);
                } else { //parinte fantoma
                self.radacini.push(p.pid);
                toate_nodurile.insert(p.pid);
                }
            } else {
                self.radacini.push(p.pid);
                toate_nodurile.insert(p.pid);
            }
        }
        //propagare cpu_usage de la fii la parinti
        let mut frunze: HashSet<u32> = toate_nodurile.difference(&parinti).cloned().collect();
        while !frunze.is_empty() {
            let mut urm_frunze = HashSet::new();
            
            for pid in frunze {
                let child_cpu = if let Some(proc) = self.process_map.get(&pid) {
                    proc.cpu_usage
                } else {
                    0.0
                };  
                if let Some(&ppid) = frunze_to_parinti.get(&pid) {
                    if let Some(parent) = self.process_map.get_mut(&ppid) {
                        parent.cpu_usage += child_cpu;
                        urm_frunze.insert(ppid);
                    }
                }
            }
            frunze = urm_frunze;
        }
        self.cur_data = Some(data);
    }

}


fn backend(tx: Sender<Overall_Data>, ctx: egui::Context) {
        let mut system = System::new();
        let mut users = Users::new_with_refreshed_list();

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
                uptime: System::uptime()
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


        });
    }
   }


fn main() -> Result<(),eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Rust Process Monitor", native_options, Box::new(|cc| Ok(Box::new(TaskManager::new(cc)))))
}
