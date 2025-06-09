use crate::schedule::{Vaccine, VaccineAppointment};
use egui_dnd::dnd;
use itertools::Itertools;
use jiff::Zoned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(default)]
pub struct VaccineConfig {
    name: String,
    enabled: bool,
}

// Configuration for the scheduling process.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Profile {
    vaccines: Vec<VaccineConfig>,
    shots_per_visit: u8,
    end_plan_year: i16,
    schedule: Vec<VaccineAppointment>,
    schedule_base: Zoned,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct TemplateApp {
    active_profile: String,
    profiles: HashMap<String, Profile>,
    show_profiles: bool,
    show_about: bool,
    add_profile_name: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert(
            "Default".to_owned(),
            Profile {
                vaccines: Vaccine::get_vaccines()
                    .keys()
                    .sorted()
                    .map(|v| VaccineConfig {
                        name: v.to_string(),
                        enabled: true,
                    })
                    .collect(),
                shots_per_visit: 3,
                end_plan_year: Zoned::now().year() + 55,
                schedule: vec![],
                schedule_base: Zoned::now(),
            },
        );
        Self {
            active_profile: "Default".to_owned(),
            profiles,
            show_profiles: false,
            show_about: false,
            add_profile_name: "".to_owned(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Profiles...").clicked() {
                            self.show_profiles = true;
                        }
                        ui.separator();
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("About...").clicked() {
                            self.show_about = true;
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Schedule Configuration");
            ui.label("Select and prioritize the vaccines you want to get");

            // Order the vaccines and select which ones to enable.
            let profile = self.profiles.get_mut(&self.active_profile).unwrap();
            let response = dnd(ui, "dnd_vaccines").show(
                profile.vaccines.iter_mut(),
                |ui, vaccine_cfg, handle, _state| {
                    let vaccine = Vaccine::get_vaccines()
                        .get(vaccine_cfg.name.as_str())
                        .expect("valid vaccine name");
                    handle.ui(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(egui::include_image!(
                                "../assets/icons8-drag-handle-30.png"
                            )));
                            ui.checkbox(&mut vaccine_cfg.enabled, "");
                            let resp = ui.add_enabled(
                                vaccine_cfg.enabled,
                                egui::Label::new(format!(
                                    "{} ({})",
                                    vaccine.name(),
                                    vaccine.treats_str(),
                                )),
                            );
                            if resp.hovered() {
                                resp.show_tooltip_text(format!(
                                    "Dose: {}\nBoost: {}\nNotes: {}",
                                    vaccine.dosage_schedule(),
                                    vaccine.booster_schedule(),
                                    vaccine.notes()
                                ));
                            }
                        });
                    });
                },
            );
            if let Some(update) = response.update {
                profile.vaccines.swap(update.from, update.to);
            }

            // Select concurrency
            ui.horizontal(|ui| {
                let r0 = ui.label("Max Shots per visit:");
                let r1 = ui.add(egui::Slider::new(&mut profile.shots_per_visit, 1..=10));
                for resp in [r0, r1].iter() {
                    if resp.hovered() {
                        resp.show_tooltip_text("Don't schedule more than this many shots per day.")
                    }
                }
            });
            ui.horizontal(|ui| {
                let year = Zoned::now().year();
                let r0 = ui.label("End plan year:");
                let r1 = ui.add(egui::Slider::new(
                    &mut profile.end_plan_year,
                    year..=year + 100,
                ));
                for resp in [r0, r1].iter() {
                    if resp.hovered() {
                        resp.show_tooltip_text("When to stop scheduling vaccines.")
                    }
                }
            });

            ui.separator();

            // Compute the schedule
            ui.horizontal(|ui| {
                if ui.button("Propose Schedule").clicked() {
                    profile.schedule_base = Zoned::now();
                    profile.schedule = Vaccine::schedule(
                        &profile.schedule_base,
                        profile
                            .vaccines
                            .iter()
                            .filter(|v| v.enabled)
                            .map(|v| v.name.clone()),
                        profile.shots_per_visit,
                        profile.end_plan_year,
                        vec![],
                    )
                    .unwrap();
                }
                ui.label(format!("Last computed at: {}", profile.schedule_base));
            });

            // Show the current schedule
            let now = Zoned::now();
            let year = now.year();
            let month = now.month();
            for y in year..year + 50 {
                if profile.schedule.iter().any(|appt| appt.year() == y) {
                    ui.heading(format!("{}", y));
                    ui.separator();
                }
                for mo in month..month + 12 {
                    if profile
                        .schedule
                        .iter()
                        .any(|appt| appt.year() == y && appt.month() == mo)
                    {
                        let tmp = jiff::civil::date(y, mo, 1);
                        ui.heading(format!("{}", tmp.strftime("%B")));
                    }
                    for appt in &profile.schedule {
                        if appt.year() == y && appt.month() == mo {
                            ui.label(appt.vaccine());
                        }
                    }
                }
            }

            // Show sub-windows
            if self.show_profiles {
                self.show_profile_list(ctx);
            }
            if self.show_about {
                self.show_about(ctx);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

impl TemplateApp {
    fn show_profile_list(&mut self, ctx: &egui::Context) {
        egui::Window::new("Profiles").show(ctx, |ui| {
            let profile_names = self.profiles.keys().cloned().collect_vec();
            for name in profile_names {
                let is_active_row = name == self.active_profile;
                ui.add_enabled_ui(!is_active_row, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Activate").clicked() {
                            self.active_profile = name.clone();
                        }
                        if ui.button("Delete").clicked() {
                            self.profiles.remove(&name);
                        }
                        ui.label(name);
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("Add a profile:");
                    ui.text_edit_singleline(&mut self.add_profile_name);
                    if ui.button("Add").clicked() {
                        self.profiles
                            .insert(self.add_profile_name.clone(), Profile::default());
                        self.active_profile = self.add_profile_name.clone();
                        self.add_profile_name = "".to_owned();
                    }
                });
            }
            ui.separator();
            if ui.button("Close").clicked() {
                self.show_profiles = false;
            }
        });
    }

    fn show_about(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .show(ctx, |ui| {
                ui.label("Usage of this (extremely simple) tool does not constitute medical advice. Please consult a doctor or pharmacist before taking any vaccines.");
                ui.label("However, there is overwhelming evidence that vaccines are both safe and effective. Vaccines are so effective that we can't study the long-term effectiveness because there's nobody sick left to study.");
                ui.label("The source for this tool is available on GitHub:");
                ui.hyperlink_to("https://github.com/terrence2/vaccine_helper", "https://github.com/jimmycuadra/vaccine_helper");
                ui.separator();
                if ui.button("Close").clicked() {
                    self.show_about = false;
                }
            });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
