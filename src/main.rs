#![windows_subsystem = "windows"]

use eframe::egui;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use csv::{Reader, Writer};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Notes {
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct KeyDetails {
    name: String,
    notes: Vec<Notes>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Concept {
    name: String,
    details: Vec<KeyDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Subcategory {
    name: String,
    concepts: Vec<Concept>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Category {
    name: String,
    subcategories: Vec<Subcategory>,
}

#[derive(Serialize, Deserialize)]
struct Microfiche {
    categories: HashMap<String, Category>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FicheRow {
    #[serde(rename = "Category")]
    category: String,
    #[serde(rename = "Subcategory")]
    subcategory: String,
    #[serde(rename = "Concept")]
    concept: String,
    #[serde(rename = "KeyDetail")]
    key_detail: String,
    #[serde(rename = "Note")]
    note: String,
}

impl Microfiche {
    fn new() -> Self {
        Microfiche {
            categories: HashMap::new(),
        }
    }
    
    fn from_csv(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut fiche = Microfiche::new();
        let mut rdr = Reader::from_path(path)?;
        
        for result in rdr.deserialize() {
            let row: FicheRow = result?;
            fiche.add_row(row);
        }
        
        Ok(fiche)
    }
    
    fn to_csv(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_path(path)?;
        wtr.write_record(&["Category", "Subcategory", "Concept", "KeyDetail", "Note"])?;
        
        for (cat_name, category) in &self.categories {
            for subcat in &category.subcategories {
                for concept in &subcat.concepts {
                    for detail in &concept.details {
                        for note in &detail.notes {
                            wtr.write_record(&[
                                &cat_name,
                                &subcat.name,
                                &concept.name,
                                &detail.name,
                                &note.content,
                            ])?;
                        }
                    }
                }
            }
        }
        
        wtr.flush()?;
        Ok(())
    }
    
    fn add_row(&mut self, row: FicheRow) {
        let category = self.categories.entry(row.category.clone())
            .or_insert_with(|| Category {
                name: row.category.clone(),
                subcategories: Vec::new(),
            });
        
        if !category.subcategories.iter().any(|s| s.name == row.subcategory) {
            category.subcategories.push(Subcategory {
                name: row.subcategory.clone(),
                concepts: Vec::new(),
            });
        }
        let subcat = category.subcategories.iter_mut()
            .find(|s| s.name == row.subcategory)
            .unwrap();
        
        if !subcat.concepts.iter().any(|c| c.name == row.concept) {
            subcat.concepts.push(Concept {
                name: row.concept.clone(),
                details: Vec::new(),
            });
        }
        let concept = subcat.concepts.iter_mut()
            .find(|c| c.name == row.concept)
            .unwrap();
        
        if !concept.details.iter().any(|d| d.name == row.key_detail) {
            concept.details.push(KeyDetails {
                name: row.key_detail.clone(),
                notes: Vec::new(),
            });
        }
        let detail = concept.details.iter_mut()
            .find(|d| d.name == row.key_detail)
            .unwrap();
        
        detail.notes.push(Notes {
            content: row.note,
        });
    }
    
    fn search(&self, query: &str) -> Vec<(String, String, String, String, String)> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        if query_lower.is_empty() {
            return results;
        }
        
        for (cat_name, category) in &self.categories {
            for subcat in &category.subcategories {
                for concept in &subcat.concepts {
                    for detail in &concept.details {
                        for note in &detail.notes {
                            let full_text = format!("{} {} {} {} {}", 
                                cat_name, subcat.name, concept.name, detail.name, note.content)
                                .to_lowercase();
                            
                            if full_text.contains(&query_lower) {
                                results.push((
                                    cat_name.clone(),
                                    subcat.name.clone(),
                                    concept.name.clone(),
                                    detail.name.clone(),
                                    note.content.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        
        results
    }
    
    fn delete_note(&mut self, cat: &str, sub: &str, con: &str, det: &str, note_content: &str) -> bool {
        if let Some(category) = self.categories.get_mut(cat) {
            if let Some(subcat) = category.subcategories.iter_mut().find(|s| s.name == sub) {
                if let Some(concept) = subcat.concepts.iter_mut().find(|c| c.name == con) {
                    if let Some(detail) = concept.details.iter_mut().find(|d| d.name == det) {
                        if let Some(pos) = detail.notes.iter().position(|n| n.content == note_content) {
                            detail.notes.remove(pos);
                            
                            // Cleanup empty structures
                            if detail.notes.is_empty() {
                                concept.details.retain(|d| !d.notes.is_empty());
                            }
                            if concept.details.is_empty() {
                                subcat.concepts.retain(|c| !c.details.is_empty());
                            }
                            if subcat.concepts.is_empty() {
                                category.subcategories.retain(|s| !s.concepts.is_empty());
                            }
                            if category.subcategories.is_empty() {
                                self.categories.remove(cat);
                            }
                            
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    
    fn stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        let mut total_subcats = 0;
        let mut total_concepts = 0;
        let mut total_details = 0;
        let mut total_notes = 0;
        
        stats.insert("categories".to_string(), self.categories.len());
        
        for (_, category) in &self.categories {
            total_subcats += category.subcategories.len();
            for subcat in &category.subcategories {
                total_concepts += subcat.concepts.len();
                for concept in &subcat.concepts {
                    total_details += concept.details.len();
                    for detail in &concept.details {
                        total_notes += detail.notes.len();
                    }
                }
            }
        }
        
        stats.insert("subcategories".to_string(), total_subcats);
        stats.insert("concepts".to_string(), total_concepts);
        stats.insert("key_details".to_string(), total_details);
        stats.insert("total_notes".to_string(), total_notes);
        
        stats
    }
}

struct MicroficheApp {
    microfiche: Microfiche,
    current_file: Option<String>,
    
    // UI State
    search_query: String,
    search_results: Vec<(String, String, String, String, String)>,
    
    // Create form
    new_category: String,
    new_subcategory: String,
    new_concept: String,
    new_key_detail: String,
    new_note: String,
    
    // Selected for viewing
    selected_category: Option<String>,
    selected_subcategory: Option<String>,
    selected_concept: Option<String>,
    
    // Messages
    status_message: String,
    
    // View mode
    view_mode: ViewMode,
    
    // Theme
    current_theme: Theme,
    show_theme_selector: bool,
}

#[derive(PartialEq, Clone, Copy)]
enum Theme {
    Monokai,
    TomorrowBlueHour,
    DarkPlus,
}

impl Theme {
    fn name(&self) -> &str {
        match self {
            Theme::Monokai => "Monokai",
            Theme::TomorrowBlueHour => "Tomorrow (Blue Hour)",
            Theme::DarkPlus => "Dark+",
        }
    }
    
    fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        
        match self {
            Theme::Monokai => {
                // Monokai - warm dark theme with purple/pink accents
                visuals.window_fill = egui::Color32::from_rgb(39, 40, 34);
                visuals.panel_fill = egui::Color32::from_rgb(39, 40, 34);
                visuals.faint_bg_color = egui::Color32::from_rgb(49, 50, 44);
                
                visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(49, 50, 44);
                visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 61, 54);
                visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(75, 76, 68);
                visuals.widgets.active.bg_fill = egui::Color32::from_rgb(90, 91, 82);
                
                visuals.selection.bg_fill = egui::Color32::from_rgb(73, 72, 62);
                visuals.selection.stroke.color = egui::Color32::from_rgb(249, 38, 114);
                
                visuals.override_text_color = Some(egui::Color32::from_rgb(248, 248, 242));
                visuals.hyperlink_color = egui::Color32::from_rgb(102, 217, 239);
                visuals.warn_fg_color = egui::Color32::from_rgb(230, 219, 116);
                visuals.error_fg_color = egui::Color32::from_rgb(249, 38, 114);
            },
            Theme::TomorrowBlueHour => {
                // Tomorrow Night Blue - cool blue theme
                visuals.window_fill = egui::Color32::from_rgb(0, 29, 51);
                visuals.panel_fill = egui::Color32::from_rgb(0, 29, 51);
                visuals.faint_bg_color = egui::Color32::from_rgb(0, 43, 71);
                
                visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0, 43, 71);
                visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(0, 56, 92);
                visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(7, 70, 115);
                visuals.widgets.active.bg_fill = egui::Color32::from_rgb(17, 85, 135);
                
                visuals.selection.bg_fill = egui::Color32::from_rgb(0, 72, 119);
                visuals.selection.stroke.color = egui::Color32::from_rgb(125, 174, 198);
                
                visuals.override_text_color = Some(egui::Color32::from_rgb(231, 232, 235));
                visuals.hyperlink_color = egui::Color32::from_rgb(125, 174, 198);
                visuals.warn_fg_color = egui::Color32::from_rgb(255, 204, 102);
                visuals.error_fg_color = egui::Color32::from_rgb(255, 102, 102);
            },
            Theme::DarkPlus => {
                // Dark+ - VS Code default dark theme
                visuals.window_fill = egui::Color32::from_rgb(30, 30, 30);
                visuals.panel_fill = egui::Color32::from_rgb(30, 30, 30);
                visuals.faint_bg_color = egui::Color32::from_rgb(37, 37, 38);
                
                visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 45, 45);
                visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 60, 60);
                visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(75, 75, 75);
                visuals.widgets.active.bg_fill = egui::Color32::from_rgb(90, 90, 90);
                
                visuals.selection.bg_fill = egui::Color32::from_rgb(38, 79, 120);
                visuals.selection.stroke.color = egui::Color32::from_rgb(14, 99, 156);
                
                visuals.override_text_color = Some(egui::Color32::from_rgb(212, 212, 212));
                visuals.hyperlink_color = egui::Color32::from_rgb(78, 162, 230);
                visuals.warn_fg_color = egui::Color32::from_rgb(206, 145, 120);
                visuals.error_fg_color = egui::Color32::from_rgb(244, 71, 71);
            },
        }
        
        ctx.set_visuals(visuals);
    }
}

#[derive(PartialEq)]
enum ViewMode {
    Browse,
    Search,
    Create,
    Stats,
}

impl Default for MicroficheApp {
    fn default() -> Self {
        Self {
            microfiche: Microfiche::new(),
            current_file: None,
            search_query: String::new(),
            search_results: Vec::new(),
            new_category: String::new(),
            new_subcategory: String::new(),
            new_concept: String::new(),
            new_key_detail: String::new(),
            new_note: String::new(),
            selected_category: None,
            selected_subcategory: None,
            selected_concept: None,
            status_message: String::from("Ready"),
            view_mode: ViewMode::Browse,
            current_theme: Theme::Monokai,
            show_theme_selector: false,
        }
    }
}

impl MicroficheApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app = Self::default();
        app.current_theme.apply(&cc.egui_ctx);
        app
    }
    
    fn load_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .pick_file()
        {
            match Microfiche::from_csv(path.to_str().unwrap()) {
                Ok(fiche) => {
                    self.microfiche = fiche;
                    self.current_file = Some(path.to_str().unwrap().to_string());
                    self.status_message = format!("Loaded: {}", path.file_name().unwrap().to_str().unwrap());
                    self.search_results.clear();
                },
                Err(e) => {
                    self.status_message = format!("Error loading file: {}", e);
                }
            }
        }
    }
    
    fn save_file(&mut self) {
        if let Some(ref path) = self.current_file {
            match self.microfiche.to_csv(path) {
                Ok(_) => {
                    self.status_message = "File saved successfully".to_string();
                },
                Err(e) => {
                    self.status_message = format!("Error saving: {}", e);
                }
            }
        } else {
            self.save_file_as();
        }
    }
    
    fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .save_file()
        {
            match self.microfiche.to_csv(path.to_str().unwrap()) {
                Ok(_) => {
                    self.current_file = Some(path.to_str().unwrap().to_string());
                    self.status_message = format!("Saved: {}", path.file_name().unwrap().to_str().unwrap());
                },
                Err(e) => {
                    self.status_message = format!("Error saving: {}", e);
                }
            }
        }
    }
    
    fn render_top_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            if ui.button("Open").clicked() {
                self.load_file();
            }
            if ui.button("Save").clicked() {
                self.save_file();
            }
            if ui.button("Save As...").clicked() {
                self.save_file_as();
            }
            
            ui.separator();
            
            if ui.selectable_label(self.view_mode == ViewMode::Browse, "Browse").clicked() {
                self.view_mode = ViewMode::Browse;
            }
            if ui.selectable_label(self.view_mode == ViewMode::Search, "Search").clicked() {
                self.view_mode = ViewMode::Search;
            }
            if ui.selectable_label(self.view_mode == ViewMode::Create, "Create").clicked() {
                self.view_mode = ViewMode::Create;
            }
            if ui.selectable_label(self.view_mode == ViewMode::Stats, "Stats").clicked() {
                self.view_mode = ViewMode::Stats;
            }
            
            ui.separator();
            
            if ui.button("Theme").clicked() {
                self.show_theme_selector = !self.show_theme_selector;
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(&self.status_message);
            });
        });
        
        // Theme selector window
        if self.show_theme_selector {
            egui::Window::new("Theme Selection")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        if ui.selectable_label(
                            self.current_theme == Theme::Monokai, 
                            "Monokai"
                        ).clicked() {
                            self.current_theme = Theme::Monokai;
                            self.current_theme.apply(ctx);
                            self.show_theme_selector = false;
                        }
                        
                        if ui.selectable_label(
                            self.current_theme == Theme::TomorrowBlueHour, 
                            "Tomorrow (Blue Hour)"
                        ).clicked() {
                            self.current_theme = Theme::TomorrowBlueHour;
                            self.current_theme.apply(ctx);
                            self.show_theme_selector = false;
                        }
                        
                        if ui.selectable_label(
                            self.current_theme == Theme::DarkPlus, 
                            "Dark+"
                        ).clicked() {
                            self.current_theme = Theme::DarkPlus;
                            self.current_theme.apply(ctx);
                            self.show_theme_selector = false;
                        }
                    });
                    
                    ui.separator();
                    
                    if ui.button("Close").clicked() {
                        self.show_theme_selector = false;
                    }
                });
        }
    }
    
    fn render_browse_view(&mut self, ui: &mut egui::Ui) {
        egui::SidePanel::left("categories_panel")
            .resizable(true)
            .default_width(200.0)
            .show_inside(ui, |ui| {
                ui.heading("Categories");
                ui.separator();
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut categories: Vec<_> = self.microfiche.categories.keys().collect();
                    categories.sort();
                    
                    for cat_name in categories {
                        let is_selected = self.selected_category.as_ref() == Some(cat_name);
                        if ui.selectable_label(is_selected, cat_name).clicked() {
                            self.selected_category = Some(cat_name.clone());
                            self.selected_subcategory = None;
                            self.selected_concept = None;
                        }
                    }
                });
            });
        
        if let Some(ref cat_name) = self.selected_category.clone() {
            if let Some(category) = self.microfiche.categories.get(cat_name) {
                egui::SidePanel::left("subcategories_panel")
                    .resizable(true)
                    .default_width(200.0)
                    .show_inside(ui, |ui| {
                        ui.heading("Subcategories");
                        ui.separator();
                        
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for subcat in &category.subcategories {
                                let is_selected = self.selected_subcategory.as_ref() == Some(&subcat.name);
                                if ui.selectable_label(is_selected, &subcat.name).clicked() {
                                    self.selected_subcategory = Some(subcat.name.clone());
                                    self.selected_concept = None;
                                }
                            }
                        });
                    });
            }
        }
        
        // Collect data before rendering to avoid borrow issues
        let display_data: Option<(String, String, Vec<(String, Vec<(String, Vec<String>)>)>)> = 
            if let Some(ref cat_name) = self.selected_category {
                if let Some(category) = self.microfiche.categories.get(cat_name) {
                    if let Some(ref sub_name) = self.selected_subcategory {
                        if let Some(subcat) = category.subcategories.iter().find(|s| &s.name == sub_name) {
                            let concepts: Vec<_> = subcat.concepts.iter().map(|concept| {
                                let details: Vec<_> = concept.details.iter().map(|detail| {
                                    let notes: Vec<_> = detail.notes.iter().map(|n| n.content.clone()).collect();
                                    (detail.name.clone(), notes)
                                }).collect();
                                (concept.name.clone(), details)
                            }).collect();
                            Some((cat_name.clone(), sub_name.clone(), concepts))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
        
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some((cat_name, sub_name, concepts)) = display_data {
                ui.heading(format!("{} > {}", cat_name, sub_name));
                ui.separator();
                
                let mut to_delete: Option<(String, String, String, String, String)> = None;
                let mut to_edit: Option<(String, String, String, String, String)> = None;
                let mut to_template: Option<(String, String, String, String)> = None;
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (concept_name, details) in concepts {
                        ui.group(|ui| {
                            ui.strong(&concept_name);
                            ui.separator();
                            
                            for (detail_name, notes) in details {
                                ui.label(egui::RichText::new(&detail_name).color(egui::Color32::from_rgb(100, 149, 237)));
                                
                                for note in notes {
                                    ui.horizontal(|ui| {
                                        ui.label("  •");
                                        ui.label(&note);
                                        
                                        if ui.button("Template").clicked() {
                                            to_template = Some((
                                                cat_name.clone(),
                                                sub_name.clone(),
                                                concept_name.clone(),
                                                detail_name.clone(),
                                            ));
                                        }
                                        
                                        if ui.button("Edit").clicked() {
                                            to_edit = Some((
                                                cat_name.clone(),
                                                sub_name.clone(),
                                                concept_name.clone(),
                                                detail_name.clone(),
                                                note.clone(),
                                            ));
                                        }
                                        
                                        if ui.button("Delete").clicked() {
                                            to_delete = Some((
                                                cat_name.clone(),
                                                sub_name.clone(),
                                                concept_name.clone(),
                                                detail_name.clone(),
                                                note.clone(),
                                            ));
                                        }
                                    });
                                }
                                ui.add_space(5.0);
                            }
                        });
                        ui.add_space(10.0);
                    }
                });
                
                // Handle actions after the scroll area
                if let Some((cat, sub, con, det, note)) = to_delete {
                    if self.microfiche.delete_note(&cat, &sub, &con, &det, &note) {
                        self.status_message = "Entry deleted".to_string();
                    }
                }
                
                if let Some((cat, sub, con, det, note)) = to_edit {
                    // Delete the old entry
                    if self.microfiche.delete_note(&cat, &sub, &con, &det, &note) {
                        // Populate the create form with the old data
                        self.new_category = cat;
                        self.new_subcategory = sub;
                        self.new_concept = con;
                        self.new_key_detail = det;
                        self.new_note = note;
                        
                        // Switch to create view
                        self.view_mode = ViewMode::Create;
                        self.status_message = "Entry loaded for editing. Modify and click Create to save.".to_string();
                    }
                }
                
                if let Some((cat, sub, con, det)) = to_template {
                    // Populate the create form but leave note empty
                    self.new_category = cat;
                    self.new_subcategory = sub;
                    self.new_concept = con;
                    self.new_key_detail = det;
                    self.new_note.clear();
                    
                    // Switch to create view
                    self.view_mode = ViewMode::Create;
                    self.status_message = "Template loaded. Add your new note and click Create.".to_string();
                }
            } else if self.selected_category.is_some() && self.selected_subcategory.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a subcategory to view its contents");
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a category from the left panel");
                });
            }
        });
    }
    
    fn render_search_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            let response = ui.text_edit_singleline(&mut self.search_query);
            
            if response.changed() || ui.button("Search").clicked() {
                self.search_results = self.microfiche.search(&self.search_query);
            }
        });
        
        ui.separator();
        
        ui.label(format!("Found {} results", self.search_results.len()));
        
        // Clone results to avoid borrow issues
        let results = self.search_results.clone();
        let mut to_delete: Option<(String, String, String, String, String)> = None;
        let mut to_edit: Option<(String, String, String, String, String)> = None;
        let mut to_template: Option<(String, String, String, String)> = None;
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (cat, sub, con, det, note) in &results {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(format!("{}.{}.{}.{}", cat, sub, con, det));
                            ui.label(note);
                        });
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Delete").clicked() {
                                to_delete = Some((cat.clone(), sub.clone(), con.clone(), det.clone(), note.clone()));
                            }
                            
                            if ui.button("Edit").clicked() {
                                to_edit = Some((cat.clone(), sub.clone(), con.clone(), det.clone(), note.clone()));
                            }
                            
                            if ui.button("Template").clicked() {
                                to_template = Some((cat.clone(), sub.clone(), con.clone(), det.clone()));
                            }
                        });
                    });
                });
                ui.add_space(5.0);
            }
        });
        
        // Handle actions after the scroll area
        if let Some((cat, sub, con, det, note)) = to_delete {
            if self.microfiche.delete_note(&cat, &sub, &con, &det, &note) {
                self.search_results = self.microfiche.search(&self.search_query);
                self.status_message = "Entry deleted".to_string();
            }
        }
        
        if let Some((cat, sub, con, det, note)) = to_edit {
            // Delete the old entry
            if self.microfiche.delete_note(&cat, &sub, &con, &det, &note) {
                // Populate the create form with the old data
                self.new_category = cat;
                self.new_subcategory = sub;
                self.new_concept = con;
                self.new_key_detail = det;
                self.new_note = note;
                
                // Switch to create view
                self.view_mode = ViewMode::Create;
                self.status_message = "Entry loaded for editing. Modify and click Create to save.".to_string();
                
                // Refresh search results
                self.search_results = self.microfiche.search(&self.search_query);
            }
        }
        
        if let Some((cat, sub, con, det)) = to_template {
            // Populate the create form but leave note empty
            self.new_category = cat;
            self.new_subcategory = sub;
            self.new_concept = con;
            self.new_key_detail = det;
            self.new_note.clear();
            
            // Switch to create view
            self.view_mode = ViewMode::Create;
            self.status_message = "Template loaded. Add your new note and click Create.".to_string();
        }
    }
    
    fn render_create_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Create New Entry");
        ui.separator();
        
        egui::Grid::new("create_grid")
            .num_columns(2)
            .spacing([10.0, 10.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Category:");
                ui.text_edit_singleline(&mut self.new_category);
                ui.end_row();
                
                ui.label("Subcategory:");
                ui.text_edit_singleline(&mut self.new_subcategory);
                ui.end_row();
                
                ui.label("Concept:");
                ui.text_edit_singleline(&mut self.new_concept);
                ui.end_row();
                
                ui.label("Key Detail:");
                ui.text_edit_singleline(&mut self.new_key_detail);
                ui.end_row();
                
                ui.label("Note:");
                ui.text_edit_multiline(&mut self.new_note);
                ui.end_row();
            });
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            let can_create = !self.new_category.is_empty() 
                && !self.new_subcategory.is_empty()
                && !self.new_concept.is_empty()
                && !self.new_key_detail.is_empty()
                && !self.new_note.is_empty();
            
            if ui.add_enabled(can_create, egui::Button::new("Create Entry")).clicked() {
                self.microfiche.add_row(FicheRow {
                    category: self.new_category.clone(),
                    subcategory: self.new_subcategory.clone(),
                    concept: self.new_concept.clone(),
                    key_detail: self.new_key_detail.clone(),
                    note: self.new_note.clone(),
                });
                
                self.status_message = "Entry created successfully".to_string();
                
                // Clear form
                self.new_note.clear();
            }
            
            if ui.button("Clear Form").clicked() {
                self.new_category.clear();
                self.new_subcategory.clear();
                self.new_concept.clear();
                self.new_key_detail.clear();
                self.new_note.clear();
            }
        });
        
        ui.add_space(20.0);
        ui.separator();
        
        ui.label("Tip: Leave fields the same to add multiple notes to the same path");
    }

    fn estimate_memory_usage(&self) -> usize {
        let mut total = 0;
        
        for (cat_name, category) in &self.microfiche.categories {
            total += cat_name.len();
            for subcat in &category.subcategories {
                total += subcat.name.len();
                for concept in &subcat.concepts {
                    total += concept.name.len();
                    for detail in &concept.details {
                        total += detail.name.len();
                        for note in &detail.notes {
                            total += note.content.len();
                        }
                    }
                }
            }
        }
        
        // Add overhead for structure (rough estimate)
        total * 2
    }

    fn find_deepest_category(&self) -> String {
        let mut deepest_name = "N/A".to_string();
        let mut max_depth = 0;
        
        for (cat_name, category) in &self.microfiche.categories {
            let subcats = category.subcategories.len();
            if subcats > max_depth {
                max_depth = subcats;
                deepest_name = cat_name.clone();
            }
        }
        
        if max_depth > 0 {
            format!("{} ({} subcats)", deepest_name, max_depth)
        } else {
            deepest_name
        }
    }

    fn find_largest_category(&self) -> String {
        let mut largest_name = "N/A".to_string();
        let mut max_notes = 0;
        
        for (cat_name, category) in &self.microfiche.categories {
            let note_count: usize = category.subcategories.iter()
                .flat_map(|s| &s.concepts)
                .flat_map(|c| &c.details)
                .map(|d| d.notes.len())
                .sum();
            
            if note_count > max_notes {
                max_notes = note_count;
                largest_name = cat_name.clone();
            }
        }
        
        if max_notes > 0 {
            format!("{} ({} notes)", largest_name, max_notes)
        } else {
            largest_name
        }
    }
    
    fn render_stats_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Knowledge Base Statistics");
        ui.separator();
        
        let stats = self.microfiche.stats();
        let total_notes = *stats.get("total_notes").unwrap_or(&0);
        let total_cats = *stats.get("categories").unwrap_or(&0);
        let total_concepts = *stats.get("concepts").unwrap_or(&0);
        let total_details = *stats.get("key_details").unwrap_or(&0);
        
        // Calculate estimated memory usage
        let estimated_bytes = self.estimate_memory_usage();
        let max_reasonable_bytes = 100 * 1024 * 1024; // 100 MB as a reasonable limit
        
        // Calculate averages
        let avg_notes_per_cat = if total_cats > 0 { total_notes as f32 / total_cats as f32 } else { 0.0 };
        let avg_concepts_per_cat = if total_cats > 0 { total_concepts as f32 / total_cats as f32 } else { 0.0 };
        
        // Get theme colors
        let (accent_color, secondary_color, tertiary_color) = match self.current_theme {
            Theme::Monokai => (
                egui::Color32::from_rgb(249, 38, 114),  // Pink
                egui::Color32::from_rgb(102, 217, 239), // Cyan
                egui::Color32::from_rgb(230, 219, 116), // Yellow
            ),
            Theme::TomorrowBlueHour => (
                egui::Color32::from_rgb(125, 174, 198), // Light blue
                egui::Color32::from_rgb(255, 204, 102), // Yellow
                egui::Color32::from_rgb(255, 102, 102), // Red
            ),
            Theme::DarkPlus => (
                egui::Color32::from_rgb(78, 162, 230),  // Blue
                egui::Color32::from_rgb(206, 145, 120), // Orange
                egui::Color32::from_rgb(244, 71, 71),   // Red
            ),
        };
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            let panel_height = 280.0;
            
            // Top row - 2x1 grid
            egui::Grid::new("stats_top_grid")
                .num_columns(2)
                .spacing([10.0, 10.0])
                .min_col_width((ui.available_width() - 10.0) / 2.0)
                .show(ui, |ui| {
                    // Left panel: Storage Usage
                    ui.group(|ui| {
                        ui.set_height(panel_height);
                        ui.vertical(|ui| {
                            ui.heading("Storage Usage");
                            ui.separator();
                            ui.add_space(5.0);
                            
                            let usage_percent = (estimated_bytes as f32 / max_reasonable_bytes as f32 * 100.0).min(100.0);
                            
                            ui.label(egui::RichText::new(format!("Current: {} KB", estimated_bytes / 1024)).size(14.0));
                            ui.label(egui::RichText::new(format!("Limit: {} MB", max_reasonable_bytes / (1024 * 1024))).size(14.0));
                            ui.label(egui::RichText::new(format!("Usage: {:.1}%", usage_percent)).size(14.0).strong());
                            
                            ui.add_space(15.0);
                            
                            // Storage bar
                            let bar_width = ui.available_width();
                            let bar_height = 40.0;
                            let filled_width = bar_width * (usage_percent / 100.0);
                            
                            let (rect, _response) = ui.allocate_exact_size(
                                egui::vec2(bar_width, bar_height),
                                egui::Sense::hover()
                            );
                            
                            // Draw background
                            ui.painter().rect_filled(
                                rect,
                                4.0,
                                ui.style().visuals.faint_bg_color
                            );
                            
                            // Draw filled portion
                            if filled_width > 0.0 {
                                let filled_rect = egui::Rect::from_min_size(
                                    rect.min,
                                    egui::vec2(filled_width, bar_height)
                                );
                                
                                let color = if usage_percent < 50.0 {
                                    secondary_color
                                } else if usage_percent < 80.0 {
                                    tertiary_color
                                } else {
                                    accent_color
                                };
                                
                                ui.painter().rect_filled(filled_rect, 4.0, color);
                            }
                            
                            ui.add_space(15.0);
                            ui.label(egui::RichText::new(format!("Total Entries: {}", total_notes)).size(16.0).strong().color(accent_color));
                            
                            ui.add_space(10.0);
                            
                            // Quick facts
                            ui.separator();
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new("Status:").strong());
                            ui.add_space(3.0);
                            
                            if total_notes == 0 {
                                ui.label("• Knowledge base is empty");
                                ui.label("• Start by creating your first entry");
                            } else if total_notes < 50 {
                                ui.label("• Building your knowledge base");
                                ui.label(&format!("• {} more to reach 50 notes", 50 - total_notes));
                            } else if total_notes < 100 {
                                ui.label("• Solid progress!");
                                ui.label(&format!("• {} more to reach 100 notes", 100 - total_notes));
                            } else {
                                ui.label("• Impressive collection!");
                                ui.label(&format!("• {} notes organized", total_notes));
                            }
                        });
                    });
                    
                    // Right panel: Hierarchy Overview & Insights
                    ui.group(|ui| {
                        ui.set_height(panel_height);
                        ui.vertical(|ui| {
                            ui.heading("Hierarchy & Insights");
                            ui.separator();
                            ui.add_space(5.0);
                            
                            egui::Grid::new("hierarchy_grid")
                                .num_columns(2)
                                .spacing([20.0, 10.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("Categories:").strong());
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(total_cats.to_string()).strong().size(15.0).color(accent_color));
                                    });
                                    ui.end_row();
                                    
                                    ui.label(egui::RichText::new("Subcategories:").strong());
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(stats.get("subcategories").unwrap_or(&0).to_string()).size(15.0).color(secondary_color));
                                    });
                                    ui.end_row();
                                    
                                    ui.label(egui::RichText::new("Concepts:").strong());
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(total_concepts.to_string()).size(15.0).color(tertiary_color));
                                    });
                                    ui.end_row();
                                    
                                    ui.label(egui::RichText::new("Key Details:").strong());
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(total_details.to_string()).size(15.0).color(accent_color));
                                    });
                                    ui.end_row();
                                    
                                    ui.label(egui::RichText::new("Total Notes:").strong());
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(total_notes.to_string()).strong().size(15.0).color(secondary_color));
                                    });
                                    ui.end_row();
                                });
                            
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(5.0);
                            
                            egui::Grid::new("insights_grid")
                                .num_columns(2)
                                .spacing([20.0, 8.0])
                                .show(ui, |ui| {
                                    ui.label("Avg Notes/Category:");
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(format!("{:.1}", avg_notes_per_cat)).strong());
                                    });
                                    ui.end_row();
                                    
                                    ui.label("Avg Concepts/Category:");
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(format!("{:.1}", avg_concepts_per_cat)).strong());
                                    });
                                    ui.end_row();
                                    
                                    ui.label("Deepest Category:");
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let deepest = self.find_deepest_category();
                                        ui.label(egui::RichText::new(deepest).strong());
                                    });
                                    ui.end_row();
                                    
                                    ui.label("Largest Category:");
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let largest = self.find_largest_category();
                                        ui.label(egui::RichText::new(largest).strong());
                                    });
                                    ui.end_row();
                                });
                        });
                    });
                    
                    ui.end_row();
                });
            
            ui.add_space(10.0);
            
            // Bottom panel: Full-width Category Distribution Bar Graph
            ui.group(|ui| {
                ui.set_min_height(300.0);
                ui.vertical(|ui| {
                    ui.heading("Category Distribution");
                    ui.separator();
                    ui.add_space(10.0);
                    
                    let mut cat_data: Vec<_> = self.microfiche.categories.iter()
                        .map(|(name, category)| {
                            let note_count: usize = category.subcategories.iter()
                                .flat_map(|s| &s.concepts)
                                .flat_map(|c| &c.details)
                                .map(|d| d.notes.len())
                                .sum();
                            (name.clone(), note_count)
                        })
                        .collect();
                    
                    cat_data.sort_by(|a, b| b.1.cmp(&a.1));
                    
                    if cat_data.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("No categories yet...").size(16.0).color(egui::Color32::GRAY));
                        });
                    } else {
                        let max_notes = cat_data.iter().map(|(_, n)| *n).max().unwrap_or(1);
                        let available_width = ui.available_width() - 300.0;
                        
                        egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                            for (i, (cat_name, note_count)) in cat_data.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    // Category name with fixed width
                                    ui.add_sized([200.0, 24.0], egui::Label::new(
                                        egui::RichText::new(cat_name.as_str()).size(14.0)
                                    ));
                                    
                                    // Bar
                                    let bar_width = (available_width * (*note_count as f32 / max_notes as f32)).max(5.0);
                                    let (rect, _response) = ui.allocate_exact_size(
                                        egui::vec2(bar_width, 28.0),
                                        egui::Sense::hover()
                                    );
                                    
                                    // Use theme colors
                                    let color = if i % 3 == 0 {
                                        accent_color
                                    } else if i % 3 == 1 {
                                        secondary_color
                                    } else {
                                        tertiary_color
                                    };
                                    
                                    ui.painter().rect_filled(rect, 4.0, color);
                                    
                                    // Note count
                                    ui.label(egui::RichText::new(format!("{} notes", note_count)).strong().size(14.0));
                                });
                                ui.add_space(8.0);
                            }
                        });
                    }
                });
            });
        });
    }
}

impl eframe::App for MicroficheApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_top_bar(ui, ctx);
            ui.separator();
            
            match self.view_mode {
                ViewMode::Browse => self.render_browse_view(ui),
                ViewMode::Search => self.render_search_view(ui),
                ViewMode::Create => self.render_create_view(ui),
                ViewMode::Stats => self.render_stats_view(ui),
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Fisha GUI",
        options,
        Box::new(|cc| Ok(Box::new(MicroficheApp::new(cc)))),
    )
}