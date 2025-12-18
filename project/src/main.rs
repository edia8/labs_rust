use std::{collections::HashMap};

use eframe::egui::{self};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, Users};
#[derive(PartialEq)]
enum ViewMode {
    List=0,
    Tree=1
}

struct TaskManager {
    system: System,
    users: Users,
    view: ViewMode,
    tree_cache: HashMap<Option<Pid>, Vec<Pid>>
}



impl TaskManager {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let ctx = &cc.egui_ctx;
        ctx.set_visuals(egui::Visuals::dark());
        let refreshes = RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()).with_memory(MemoryRefreshKind::nothing().with_ram());
        let mut system: System =  System::new_with_specifics(refreshes);
        let users = Users::new_with_refreshed_list();
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::everything());

        Self {
            system,
            users,
            view: ViewMode::List,
            tree_cache: HashMap::new()
        }
    }
}

fn toggle_ui_compact(ui: &mut egui::Ui, view_mode: &mut ViewMode) -> egui::Response {
    let mut is_tree_mode = *view_mode == ViewMode::Tree;
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        is_tree_mode = !is_tree_mode;
        match view_mode {
            ViewMode::List => {
                *view_mode = ViewMode::Tree
            }
            ViewMode::Tree => {
                *view_mode = ViewMode::List
            }
        }
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), is_tree_mode, "")
    });

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, is_tree_mode);
        let visuals = ui.style().interact_selectable(&response, is_tree_mode);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter().rect(
            rect,
            radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}

impl eframe::App for TaskManager {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
        //self.system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::everything());
        self.system.refresh_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()).with_memory(MemoryRefreshKind::nothing().with_ram()));
        egui::CentralPanel::default().show(ctx, |ui| {
            // --- Header: Global Stats ---
            ui.heading("Rust Process Monitor (Linux)");
            ui.separator();

            ui.horizontal(|ui| {
                let total_mem = self.system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
                let used_mem = self.system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
                ui.label(format!(
                    "Total CPU: {:.1}%",
                    self.system.global_cpu_usage()
                ));
                ui.separator();
                ui.label(format!("Memory: {:.2} GB / {:.2} GB", used_mem, total_mem));
            });

            ui.separator();

            // --- Controls ---
            ui.horizontal(|ui| {
                ui.label("List");
                toggle_ui_compact(ui, &mut self.view);
                ui.label("Tree");
            });

            ui.separator();
   });
    }
   }

fn main() -> Result<(),eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("My egui App", native_options, Box::new(|cc| Ok(Box::new(TaskManager::new(cc)))))
}
