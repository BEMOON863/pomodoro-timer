#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::{Duration, Instant};

use eframe::egui;

const WORK_SECS: f64 = 25.0 * 60.0;
const REST_SECS: f64 = 5.0 * 60.0;

const BG: egui::Color32 = egui::Color32::from_rgb(20, 20, 26);
const CARD: egui::Color32 = egui::Color32::from_rgb(31, 31, 40);
const TEXT: egui::Color32 = egui::Color32::from_rgb(236, 236, 243);
const MUTED: egui::Color32 = egui::Color32::from_rgb(138, 138, 156);
const WORK_ACCENT: egui::Color32 = egui::Color32::from_rgb(255, 122, 89);
const REST_ACCENT: egui::Color32 = egui::Color32::from_rgb(78, 201, 176);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Work,
    Rest,
}

impl Mode {
    fn duration_secs(self) -> f64 {
        match self {
            Self::Work => WORK_SECS,
            Self::Rest => REST_SECS,
        }
    }

    fn toggled(self) -> Self {
        match self {
            Self::Work => Self::Rest,
            Self::Rest => Self::Work,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Work => "Работа",
            Self::Rest => "Отдых",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Self::Work => "25 мин",
            Self::Rest => "5 мин",
        }
    }

    fn accent(self) -> egui::Color32 {
        match self {
            Self::Work => WORK_ACCENT,
            Self::Rest => REST_ACCENT,
        }
    }
}

struct Task {
    title: String,
    done: bool,
}

struct PomodoroApp {
    mode: Mode,
    remaining: f64,
    running: bool,
    last_tick: Instant,
    tasks: Vec<Task>,
    new_task: String,
}

impl PomodoroApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_theme(egui::ThemePreference::Dark);
        cc.egui_ctx.all_styles_mut(|style| {
            style.visuals = theme_visuals();
            style.spacing.item_spacing = egui::vec2(8.0, 10.0);
            style.spacing.button_padding = egui::vec2(14.0, 8.0);
            style
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
            style
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
            style
                .text_styles
                .insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
        });

        Self {
            mode: Mode::Work,
            remaining: WORK_SECS,
            running: false,
            last_tick: Instant::now(),
            tasks: Vec::new(),
            new_task: String::new(),
        }
    }

    fn tick(&mut self) {
        if !self.running {
            return;
        }

        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_tick).as_secs_f64();
        self.last_tick = now;
        self.remaining = (self.remaining - elapsed).max(0.0);

        if self.remaining <= 0.0 {
            self.finish_session();
        }
    }

    fn finish_session(&mut self) {
        self.running = false;
        self.mode = self.mode.toggled();
        self.remaining = self.mode.duration_secs();
    }

    fn start(&mut self) {
        if self.remaining <= 0.0 {
            self.remaining = self.mode.duration_secs();
        }
        self.running = true;
        self.last_tick = Instant::now();
    }

    fn pause(&mut self) {
        self.running = false;
    }

    fn reset(&mut self) {
        self.running = false;
        self.remaining = self.mode.duration_secs();
    }

    fn switch_mode(&mut self, mode: Mode) {
        if self.mode != mode {
            self.mode = mode;
            self.reset();
        }
    }

    fn add_task(&mut self) {
        let title = self.new_task.trim();
        if title.is_empty() {
            return;
        }
        self.tasks.push(Task {
            title: title.to_owned(),
            done: false,
        });
        self.new_task.clear();
    }

    fn time_text(&self) -> String {
        let total_seconds = self.remaining.ceil().max(0.0) as u64;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{minutes:02}:{seconds:02}")
    }

    fn progress(&self) -> f32 {
        let duration = self.mode.duration_secs();
        if duration <= 0.0 {
            return 0.0;
        }
        (1.0 - self.remaining / duration).clamp(0.0, 1.0) as f32
    }

    fn ui_header(&mut self, ui: &mut egui::Ui) {
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("ПОМОДОРО · ТАЙМЕР И ЗАДАЧИ")
                .size(11.0)
                .color(MUTED),
        );
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            let total = ui.available_width();
            let spacing = ui.spacing().item_spacing.x;
            let button_width = ((total - spacing) / 2.0).max(80.0);
            let side = ((total - (button_width * 2.0 + spacing)) / 2.0).max(0.0);
            ui.add_space(side);

            for mode in [Mode::Work, Mode::Rest] {
                let selected = self.mode == mode;
                let fill = if selected { mode.accent() } else { CARD };
                let text_color = if selected { BG } else { MUTED };
                let label = format!("{} · {}", mode.title(), mode.hint());
                let button = egui::Button::new(
                    egui::RichText::new(label)
                        .size(14.0)
                        .color(text_color)
                        .strong(),
                )
                .fill(fill)
                .stroke(egui::Stroke::NONE)
                .corner_radius(egui::CornerRadius::same(11));

                if ui.add_sized([button_width, 36.0], button).clicked() {
                    self.switch_mode(mode);
                }
            }
        });
    }

    fn ui_timer(&self, ui: &mut egui::Ui) {
        ui.add_space(24.0);
        ui.label(
            egui::RichText::new(self.time_text())
                .monospace()
                .size(82.0)
                .color(self.mode.accent())
                .strong(),
        );
        ui.add_space(2.0);

        let status = if self.running {
            "идёт"
        } else if self.remaining < self.mode.duration_secs() {
            "на паузе"
        } else {
            "готов к старту"
        };
        ui.label(
            egui::RichText::new(format!("{} · {status}", self.mode.title()))
                .size(13.0)
                .color(MUTED),
        );

        ui.add_space(16.0);
        ui.add(
            egui::ProgressBar::new(self.progress())
                .desired_width(300.0)
                .fill(self.mode.accent())
                .corner_radius(egui::CornerRadius::same(4)),
        );
    }

    fn ui_controls(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.0);

        let widths = [116.0, 104.0, 96.0];
        let spacing = ui.spacing().item_spacing.x;
        let row_width = widths.iter().sum::<f32>() + spacing * 2.0;
        let side = ((ui.available_width() - row_width) / 2.0).max(0.0);

        ui.horizontal(|ui| {
            ui.add_space(side);

            let can_start = !self.running;
            let (start_fill, start_text) = if can_start {
                (self.mode.accent(), BG)
            } else {
                (CARD, MUTED)
            };
            let start = egui::Button::new(
                egui::RichText::new("Старт")
                    .size(15.0)
                    .color(start_text)
                    .strong(),
            )
            .fill(start_fill)
            .stroke(egui::Stroke::NONE)
            .corner_radius(egui::CornerRadius::same(11));
            if can_start && ui.add_sized([widths[0], 40.0], start).clicked() {
                self.start();
            }

            let can_pause = self.running;
            let (pause_fill, pause_text) = if can_pause { (CARD, TEXT) } else { (BG, MUTED) };
            let pause = egui::Button::new(
                egui::RichText::new("Пауза")
                    .size(15.0)
                    .color(pause_text)
                    .strong(),
            )
            .fill(pause_fill)
            .stroke(egui::Stroke::NONE)
            .corner_radius(egui::CornerRadius::same(11));
            if can_pause && ui.add_sized([widths[1], 40.0], pause).clicked() {
                self.pause();
            }

            let reset = egui::Button::new(
                egui::RichText::new("Сброс")
                    .size(15.0)
                    .color(MUTED)
                    .strong(),
            )
            .fill(CARD)
            .stroke(egui::Stroke::NONE)
            .corner_radius(egui::CornerRadius::same(11));
            if ui.add_sized([widths[2], 40.0], reset).clicked() {
                self.reset();
            }
        });
    }

    fn ui_tasks(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Задачи")
                    .size(17.0)
                    .strong()
                    .color(TEXT),
            );
            let done = self.tasks.iter().filter(|task| task.done).count();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("выполнено {done} из {}", self.tasks.len()))
                        .size(12.0)
                        .color(MUTED),
                );
            });
        });

        ui.add_space(10.0);

        let mut submitted = false;
        let button_width = 108.0;
        ui.horizontal(|ui| {
            let spacing = ui.spacing().item_spacing.x;
            let edit_width = (ui.available_width() - button_width - spacing).max(120.0);
            let edit = ui.add_sized(
                [edit_width, 34.0],
                egui::TextEdit::singleline(&mut self.new_task)
                    .hint_text("Новая задача…")
                    .margin(egui::Margin::symmetric(10, 8)),
            );
            if edit.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                submitted = true;
            }

            let add = egui::Button::new(
                egui::RichText::new("Добавить")
                    .size(14.0)
                    .color(BG)
                    .strong(),
            )
            .fill(self.mode.accent())
            .stroke(egui::Stroke::NONE)
            .corner_radius(egui::CornerRadius::same(10));
            if ui.add_sized([button_width, 34.0], add).clicked() {
                submitted = true;
            }
        });
        if submitted {
            self.add_task();
        }

        ui.add_space(12.0);

        if self.tasks.is_empty() {
            ui.label(
                egui::RichText::new("Пока пусто — добавьте первую задачу.")
                    .size(13.0)
                    .color(MUTED)
                    .italics(),
            );
            return;
        }

        let mut remove_index = None;
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (index, task) in self.tasks.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut task.done, "");
                        let text = if task.done {
                            egui::RichText::new(task.title.as_str())
                                .size(15.0)
                                .strikethrough()
                                .color(MUTED)
                        } else {
                            egui::RichText::new(task.title.as_str())
                                .size(15.0)
                                .color(TEXT)
                        };
                        ui.label(text);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let delete =
                                egui::Button::new(egui::RichText::new("×").size(15.0).color(MUTED))
                                    .frame(false);
                            if ui.add(delete).on_hover_text("Удалить задачу").clicked()
                            {
                                remove_index = Some(index);
                            }
                        });
                    });
                }
            });

        if let Some(index) = remove_index {
            self.tasks.remove(index);
        }
    }
}

impl eframe::App for PomodoroApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.tick();
        if self.running {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(BG)
                    .inner_margin(egui::Margin::same(22)),
            )
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    self.ui_header(ui);
                    self.ui_timer(ui);
                    self.ui_controls(ui);
                });
                self.ui_tasks(ui);
            });
    }
}

fn theme_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = BG;
    visuals.extreme_bg_color = CARD;
    visuals.override_text_color = Some(TEXT);
    visuals.selection.bg_fill = WORK_ACCENT.gamma_multiply(0.35);
    visuals.selection.stroke = egui::Stroke::new(1.0, TEXT);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, CARD);

    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.bg_fill = CARD;
        widget.weak_bg_fill = CARD;
        widget.corner_radius = egui::CornerRadius::same(10);
    }

    visuals
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([430.0, 640.0])
            .with_min_inner_size([380.0, 540.0])
            .with_title("Помодоро — таймер и задачи"),
        ..Default::default()
    };

    eframe::run_native(
        "Помодоро — таймер и задачи",
        options,
        Box::new(|cc| Ok(Box::new(PomodoroApp::new(cc)))),
    )
}
