use crate::schedule::{DoseKind, Vaccine, VaccineAppointment, VaccineRecord};
use chrono::{Datelike, NaiveDate};
use egui::TextWrapMode;
use egui_dnd::dnd;
use itertools::Itertools;
use jiff::{civil::date as jiffdate, tz::TimeZone, Zoned};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(default)]
pub struct VaccineConfig {
    name: String,
    enabled: bool,
}

// Configuration for the scheduling process.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Profile {
    vaccines: Vec<VaccineConfig>,
    end_plan_year: i16,
    records: Vec<VaccineRecord>,
    schedule: Vec<VaccineAppointment>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            vaccines: Vaccine::get_vaccines()
                .values()
                .sorted()
                .map(|v| VaccineConfig {
                    name: v.name().to_owned(),
                    enabled: v.recommended(),
                })
                .collect(),
            end_plan_year: Zoned::now().year() + 55,
            records: vec![],
            schedule: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct VaccineHelperApp {
    active_profile: String,
    profiles: HashMap<String, Profile>,

    // Window state
    show_profiles: bool,
    show_preferences: bool,
    show_about: bool,

    // Add record widget
    add_record: Option<VaccineRecord>,

    // Add profile widget
    add_profile_name: String,
}

impl Default for VaccineHelperApp {
    fn default() -> Self {
        // let mut profiles = ;
        // profiles.insert("Default".to_owned(), Profile::default());
        Self {
            active_profile: "Default".to_owned(),
            profiles: HashMap::from_iter([("Default".to_owned(), Profile::default())]),
            show_profiles: false,
            show_preferences: false,
            show_about: false,
            add_record: None,
            add_profile_name: "".to_owned(),
        }
    }
}

impl VaccineHelperApp {
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

impl eframe::App for VaccineHelperApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // Menu Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Profiles...").clicked() {
                        self.show_profiles = true;
                        ui.close_menu();
                    }
                    if ui.button("Preferences...").clicked() {
                        self.show_preferences = true;
                        ui.close_menu();
                    }

                    // NOTE: no File->Quit on web pages!
                    let is_web = cfg!(target_arch = "wasm32");
                    if !is_web {
                        ui.separator();
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About...").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Vaccine Records");
                ui.label(
                    "Put immunizations you've already received here to remove them from the schedule.",
                );
                egui::Grid::new("records_grid").num_columns(5).show(ui, |ui| {
                    let mut deletions = vec![];
                    for (i, record) in self.profiles[&self.active_profile].records.iter().enumerate() {
                        ui.label(record.date().strftime("%d %b %y").to_string());
                        ui.label(record.vaccine());
                        ui.label(record.kind().to_string());
                        ui.label(record.notes());
                        if ui.button("Delete").clicked() {
                            deletions.push(i);
                        }
                        ui.end_row();
                    }
                    for deletion in deletions.iter().rev() {
                        self.profiles.get_mut(&self.active_profile).unwrap().records.remove(*deletion);
                    }
                });
                if let Some(mut record) = self.add_record.take() {
                    egui::Grid::new("record_entry_grid").num_columns(2).show(ui, |ui| {
                        let vaccine_names = Vaccine::get_vaccines().keys().map(|v| v.to_string()).sorted().collect_vec();
                        let mut current_index = vaccine_names.iter().position(|v| v == record.vaccine()).unwrap_or(0);
                        ui.label("Vaccine:");
                        egui::ComboBox::from_id_salt("record_entry_vaccine").wrap_mode(TextWrapMode::Extend).show_index(
                            ui,
                            &mut current_index,
                            vaccine_names.len(),
                            |i| &vaccine_names[i],
                        );
                        *record.vaccine_mut() = vaccine_names[current_index].clone();
                        ui.end_row();

                        let kind_names = DoseKind::all_kinds();
                        let mut current_index = kind_names.iter().position(|(_, k)| k == record.kind()).unwrap();
                        ui.label("Kind:");
                        egui::ComboBox::from_id_salt("record_entry_kind").wrap_mode(TextWrapMode::Extend).show_index(
                            ui,
                            &mut current_index,
                            kind_names.len(),
                            |i| kind_names[i].0,
                        );
                        *record.kind_mut() = kind_names[current_index].1;
                        ui.end_row();

                        ui.label("Date:");
                        let mut date = NaiveDate::from_ymd_opt(record.date().year().into(), record.date().month() as u32, record.date().day() as u32).unwrap();
                        ui.add(egui_extras::DatePickerButton::new(&mut date).show_icon(true));
                        *record.date_mut() = jiffdate(date.year() as i16, date.month() as i8, date.day() as i8).to_zoned(TimeZone::system()).expect("a valid date");
                        ui.end_row();

                        ui.label("Notes:");
                        ui.text_edit_singleline(record.notes_mut());
                        ui.end_row();
                    });
                    if ui.button("Add Record").clicked() {
                        // Note: always keep the records sorted by receipt date, not entry time.
                        self.profiles.get_mut(&self.active_profile).unwrap().records.push(record);
                        self.profiles.get_mut(&self.active_profile).unwrap().records.sort();
                        self.add_record = None;
                    } else {
                        self.add_record = Some(record);
                    }
                } else if ui.button("New Record").clicked() {
                    self.add_record = Some(VaccineRecord::default());
                }
                ui.label("");

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

                // Select end plan year
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

                // Re-compute the schedule
                // TODO: only do this if something changed? Probably not worth bothering.
                profile.schedule = Vaccine::schedule(
                    &Zoned::now(),
                    profile
                        .vaccines
                        .iter()
                        .filter(|v| v.enabled)
                        .map(|v| v.name.clone()),
                    profile.end_plan_year,
                    &profile.records
                )
                    .unwrap();

                // Show the current schedule
                let now = Zoned::now();
                let year = now.year();
                let month = now.month();
                for y in year..year + 50 {
                    if profile.schedule.iter().any(|appt| appt.year() == y) {
                        ui.heading(egui::RichText::new(format!("{}", y)).underline().strong());
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
                                ui.label(format!("    {} {}", appt.vaccine(), appt.kind()));
                            }
                        }
                    }
                }

                // Show sub-windows
                self.show_profile_list(ctx);
                self.show_preferences(ctx);
                self.show_about(ctx);

                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    // ui.with_layout(egui::Layout::left_to_right(egui::Align::RIGHT), |ui| {
                    powered_by_egui_and_eframe(ui);
                    // });
                    egui::warn_if_debug_build(ui);
                });
            });
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

impl VaccineHelperApp {
    fn show_profile_list(&mut self, ctx: &egui::Context) {
        egui::Window::new("Profiles")
            .open(&mut self.show_profiles)
            .show(ctx, |ui| {
                ui.heading("Profile Management");
                ui.label("Profile data is saved to your machine locally. Deletion is immediate, irreversible, and has no confirmation prompt.");
                ui.separator();
                let profile_names = self.profiles.keys().cloned().collect_vec();
                for name in profile_names {
                    let is_active_row = name == self.active_profile;
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(!is_active_row, |ui| {
                            if ui.button("Activate").clicked() {
                                self.active_profile = name.clone();
                            }
                        });
                        ui.add_enabled_ui(!is_active_row, |ui| {
                            if ui.button("Delete").clicked() {
                                self.profiles.remove(&name);
                            }
                        });
                        let mut content = egui::RichText::new(name);
                        if is_active_row {
                            content = content.strong();
                        }
                        ui.label(content);
                    });
                }
                ui.separator();
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
            });
    }

    fn show_preferences(&mut self, ctx: &egui::Context) {
        egui::Window::new("Preferences")
            .open(&mut self.show_preferences)
            .show(ctx, |ui| {
                egui::Grid::new("preferences_grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Night Mode:");
                        egui::widgets::global_theme_preference_buttons(ui);
                        ui.end_row();
                    });
            });
    }

    fn show_about(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .open(&mut self.show_about)
            .show(ctx, |ui| {
                ui.heading("Warning");
                ui.separator();
                ui.label("Usage of this (extremely simple) tool does not constitute medical advice. Please consult a doctor or pharmacist before putting anything in your body.");
                ui.label("");

                ui.heading("General Safety Information");
                ui.separator();
                ui.label("However, there is overwhelming evidence that vaccines are both safe and effective. Vaccines are so effective that we can't study the long-term effectiveness because there's nobody sick left to study.");
                ui.label("");
                ui.label("The odds of a severe or worse vaccine reaction are, like shark attacks and lightning strikes, low enough that computing accurate odds is impossible. Many vaccines are safe to the limits of our ability to detect, with no reported severe reactions on record.");
                ui.hyperlink_to("WHO Reaction Rates Info Sheets", "https://www.who.int/teams/regulation-prequalification/regulation-and-safety/pharmacovigilance/guidance/reaction-rates-information-sheets");
                ui.hyperlink_to("CDC Vaccine Information", "https://www.cdc.gov/vaccines/index.html");
                ui.label("");

                ui.heading("About this Tool");
                ui.separator();
                ui.label("Vaccine helper is there to give adults (with no a competent vaccine clinic in their area) a simple way to track and schedule immunizations with all the incredible new vaccines that are available since we were children.");
                ui.label("");
                ui.label("The source for this tool is available on GitHub:");
                ui.hyperlink_to("https://github.com/terrence2/vaccine_helper", "https://github.com/jimmycuadra/vaccine_helper");
            });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    // Note: right alignment, so add in opposite order.
    ui.horizontal(|ui| {
        ui.label(".");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(" and ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label("Powered by ");
    });
}
