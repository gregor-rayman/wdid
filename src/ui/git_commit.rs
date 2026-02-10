use egui::{Color32, Ui};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use super::state::DiaryViewState;
use crate::db::GitCommit;

/// Render a single git commit message.
pub fn render_git_commit(
    ui: &mut Ui,
    state: &mut DiaryViewState,
    entry: &GitCommit,
    cache: &mut CommonMarkCache,
) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(entry.time.clone())
                    .small()
                    .color(Color32::from_rgb(100, 149, 237)),
            );
            ui.label(egui::RichText::new(entry.folder.clone()).strong());
            ui.label(egui::RichText::new(format!("{:.7}", entry.id)).small());
        });
        //ui.label(entry.description.clone());
        CommonMarkViewer::new().show(ui, cache, &entry.description);
    });
}
