#![windows_subsystem = "windows"]

use eframe::egui;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use csv::{Reader, Writer, StringRecord};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Concept {
    name: String,
    notes: Vec<String>,
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
        wtr.write_record(&["Category", "Subcategory", "Concept", "Note"])?;
        
        for (cat_name, category) in &self.categories {
            for subcat in &category.subcategories {
                for concept in &subcat.concepts {
                    for note in &concept.notes {
                        wtr.write_record(&[
                            &cat_name,
                            &subcat.name,
                            &concept.name,
                            note,
                        ])?;
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
                notes: Vec::new(),
            });
        }
        let concept = subcat.concepts.iter_mut()
            .find(|c| c.name == row.concept)
            .unwrap();
        
        concept.notes.push(row.note);
    }
    
    fn search(&self, query: &str) -> Vec<(String, String, String, String)> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        if query_lower.is_empty() {
            return results;
        }
        
        for (cat_name, category) in &self.categories {
            for subcat in &category.subcategories {
                for concept in &subcat.concepts {
                    for note in &concept.notes {
                        let full_text = format!("{} {} {} {}", 
                            cat_name, subcat.name, concept.name, note)
                            .to_lowercase();
                        
                        if full_text.contains(&query_lower) {
                            results.push((
                                cat_name.clone(),
                                subcat.name.clone(),
                                concept.name.clone(),
                                note.clone(),
                            ));
                        }
                    }
                }
            }
        }
        
        results
    }
    
    fn delete_note(&mut self, cat: &str, sub: &str, con: &str, note_content: &str) -> bool {
        if let Some(category) = self.categories.get_mut(cat) {
            if let Some(subcat) = category.subcategories.iter_mut().find(|s| s.name == sub) {
                if let Some(concept) = subcat.concepts.iter_mut().find(|c| c.name == con) {
                    if let Some(pos) = concept.notes.iter().position(|n| n == note_content) {
                        concept.notes.remove(pos);
                        
                        // Cleanup empty structures
                        if concept.notes.is_empty() {
                            subcat.concepts.retain(|c| !c.notes.is_empty());
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
        false
    }
    
    fn stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        let mut total_subcats = 0;
        let mut total_concepts = 0;
        let mut total_notes = 0;
        
        stats.insert("categories".to_string(), self.categories.len());
        
        for (_, category) in &self.categories {
            total_subcats += category.subcategories.len();
            for subcat in &category.subcategories {
                total_concepts += subcat.concepts.len();
                for concept in &subcat.concepts {
                    total_notes += concept.notes.len();
                }
            }
        }
        
        stats.insert("subcategories".to_string(), total_subcats);
        stats.insert("concepts".to_string(), total_concepts);
        stats.insert("total_notes".to_string(), total_notes);
        
        stats
    }
}

struct MicroficheApp {
    microfiche: Microfiche,
    current_file: Option<String>,
    
    // UI State
    search_query: String,
    search_results: Vec<(String, String, String, String)>,
    
    // Create form
    new_category: String,
    new_subcategory: String,
    new_concept: String,
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

    // Pagination
    cooccurrence_page: usize,
    category_page: usize,
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
        let microfiche = Microfiche::from_csv("microfiche.csv")
            .unwrap_or_else(|_| Microfiche::new());
        
        let mut app = MicroficheApp {
            microfiche,
            current_file: Some("microfiche.csv".to_string()),
            search_query: String::new(),
            search_results: Vec::new(),
            new_category: String::new(),
            new_subcategory: String::new(),
            new_concept: String::new(),
            new_note: String::new(),
            selected_category: None,
            selected_subcategory: None,
            selected_concept: None,
            status_message: String::new(),
            view_mode: ViewMode::Browse,
            current_theme: Theme::Monokai,
            show_theme_selector: false,
            cooccurrence_page: 0,
            category_page: 0,
        };
        
        app
    }
}

impl MicroficheApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
    
    fn save_file(&mut self) {
        if let Some(ref path) = self.current_file {
            match self.microfiche.to_csv(path) {
                Ok(_) => self.status_message = format!("Saved to {}", path),
                Err(e) => self.status_message = format!("Error saving: {}", e),
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
            let path_str = path.to_string_lossy().to_string();
            match self.microfiche.to_csv(&path_str) {
                Ok(_) => {
                    self.current_file = Some(path_str.clone());
                    self.status_message = format!("Saved to {}", path_str);
                },
                Err(e) => self.status_message = format!("Error saving: {}", e),
            }
        }
    }
    
    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .pick_file()
        {
            let path_str = path.to_string_lossy().to_string();
            match Microfiche::from_csv(&path_str) {
                Ok(fiche) => {
                    self.microfiche = fiche;
                    self.current_file = Some(path_str.clone());
                    self.status_message = format!("Loaded {}", path_str);
                },
                Err(e) => self.status_message = format!("Error loading: {}", e),
            }
        }
    }
    
    fn render_top_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open...").clicked() {
                    self.open_file();
                    ui.close_menu();
                }
                if ui.button("Save").clicked() {
                    self.save_file();
                    ui.close_menu();
                }
                if ui.button("Save As...").clicked() {
                    self.save_file_as();
                    ui.close_menu();
                }
            });
            
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
                
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
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
                        
                        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
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
        let display_data: Option<(String, String, Vec<(String, Vec<String>)>)> = 
            if let Some(ref cat_name) = self.selected_category {
                if let Some(category) = self.microfiche.categories.get(cat_name) {
                    if let Some(ref sub_name) = self.selected_subcategory {
                        if let Some(subcat) = category.subcategories.iter().find(|s| &s.name == sub_name) {
                            let concepts: Vec<_> = subcat.concepts.iter().map(|concept| {
                                (concept.name.clone(), concept.notes.clone())
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
                
                let mut to_delete: Option<(String, String, String, String)> = None;
                let mut to_edit: Option<(String, String, String, String)> = None;
                let mut to_template: Option<(String, String, String)> = None;
                
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for (concept_name, notes) in concepts {
                        ui.group(|ui| {
                            ui.strong(egui::RichText::new(&concept_name).color(egui::Color32::from_rgb(100, 149, 237)));
                            ui.separator();
                            
                            for note in notes {
                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        ui.add(egui::Label::new(&note).wrap());
                                        ui.horizontal(|ui| {
                                            if ui.button("Template").clicked() {
                                                to_template = Some((
                                                    cat_name.clone(),
                                                    sub_name.clone(),
                                                    concept_name.clone(),
                                                ));
                                            }
                                            
                                            if ui.button("Edit").clicked() {
                                                to_edit = Some((
                                                    cat_name.clone(),
                                                    sub_name.clone(),
                                                    concept_name.clone(),
                                                    note.clone(),
                                                ));
                                            }
                                            
                                            if ui.button("Delete").clicked() {
                                                to_delete = Some((
                                                    cat_name.clone(),
                                                    sub_name.clone(),
                                                    concept_name.clone(),
                                                    note.clone(),
                                                ));
                                            }
                                        });
                                    });
                                });
                            }
                            ui.add_space(5.0);
                        });
                        ui.add_space(10.0);
                    }
                });
                
                // Handle actions after the scroll area
                if let Some((cat, sub, con, note)) = to_delete {
                    if self.microfiche.delete_note(&cat, &sub, &con, &note) {
                        self.status_message = "Entry deleted".to_string();
                    }
                }
                
                if let Some((cat, sub, con, note)) = to_edit {
                    // Delete the old entry
                    if self.microfiche.delete_note(&cat, &sub, &con, &note) {
                        // Populate the create form with the old data
                        self.new_category = cat;
                        self.new_subcategory = sub;
                        self.new_concept = con;
                        self.new_note = note;
                        
                        // Switch to create view
                        self.view_mode = ViewMode::Create;
                        self.status_message = "Entry loaded for editing. Modify and click Create to save.".to_string();
                    }
                }
                
                if let Some((cat, sub, con)) = to_template {
                    // Populate the create form but leave note empty
                    self.new_category = cat;
                    self.new_subcategory = sub;
                    self.new_concept = con;
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
                if self.microfiche.categories.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() / 2.0 - 50.0);
                        ui.label(egui::RichText::new("No data loaded").size(14.0));
                        ui.add_space(10.0);
                        if ui.button(egui::RichText::new("Open File").size(12.0)).clicked() {
                            self.open_file();
                        }
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select a category from the left panel");
                    });
                }
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
        let mut to_delete: Option<(String, String, String, String)> = None;
        let mut to_edit: Option<(String, String, String, String)> = None;
        let mut to_template: Option<(String, String, String)> = None;
        
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for (cat, sub, con, note) in &results {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.strong(format!("{} > {} > {}", cat, sub, con));
                        ui.add(egui::Label::new(note).wrap());
                        ui.horizontal(|ui| {
                            if ui.button("Delete").clicked() {
                                to_delete = Some((cat.clone(), sub.clone(), con.clone(), note.clone()));
                            }
                            
                            if ui.button("Edit").clicked() {
                                to_edit = Some((cat.clone(), sub.clone(), con.clone(), note.clone()));
                            }
                            
                            if ui.button("Template").clicked() {
                                to_template = Some((cat.clone(), sub.clone(), con.clone()));
                            }
                        });
                    });
                });
                ui.add_space(5.0);
            }
        });
        
        // Handle actions after the scroll area
        if let Some((cat, sub, con, note)) = to_delete {
            if self.microfiche.delete_note(&cat, &sub, &con, &note) {
                self.search_results = self.microfiche.search(&self.search_query);
                self.status_message = "Entry deleted".to_string();
            }
        }
        
        if let Some((cat, sub, con, note)) = to_edit {
            // Delete the old entry
            if self.microfiche.delete_note(&cat, &sub, &con, &note) {
                // Populate the create form with the old data
                self.new_category = cat;
                self.new_subcategory = sub;
                self.new_concept = con;
                self.new_note = note;
                
                // Switch to create view
                self.view_mode = ViewMode::Create;
                self.status_message = "Entry loaded for editing. Modify and click Create to save.".to_string();
                
                // Refresh search results
                self.search_results = self.microfiche.search(&self.search_query);
            }
        }
        
        if let Some((cat, sub, con)) = to_template {
            // Populate the create form but leave note empty
            self.new_category = cat;
            self.new_subcategory = sub;
            self.new_concept = con;
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
            .show(ui, |ui| {
                ui.label("Category:");
                ui.add(egui::TextEdit::singleline(&mut self.new_category).desired_width(f32::INFINITY));
                ui.end_row();
                
                ui.label("Subcategory:");
                ui.add(egui::TextEdit::singleline(&mut self.new_subcategory).desired_width(f32::INFINITY));
                ui.end_row();
                
                ui.label("Concept:");
                ui.add(egui::TextEdit::singleline(&mut self.new_concept).desired_width(f32::INFINITY));
                ui.end_row();
            });
        
        ui.separator();
        ui.label("Note:");
        ui.add(
            egui::TextEdit::multiline(&mut self.new_note)
                .desired_width(f32::INFINITY)
                .desired_rows(10)
        );
        
        ui.separator();
        
        if ui.button("Create").clicked() {
            if !self.new_category.is_empty() 
                && !self.new_subcategory.is_empty() 
                && !self.new_concept.is_empty() 
                && !self.new_note.is_empty() 
            {
                self.microfiche.add_row(FicheRow {
                    category: self.new_category.clone(),
                    subcategory: self.new_subcategory.clone(),
                    concept: self.new_concept.clone(),
                    note: self.new_note.clone(),
                });
                
                self.status_message = "Entry created successfully".to_string();
                
                // Clear form
                self.new_category.clear();
                self.new_subcategory.clear();
                self.new_concept.clear();
                self.new_note.clear();
            } else {
                self.status_message = "All fields are required".to_string();
            }
        }
    }
    
    fn render_stats_view(&mut self, ui: &mut egui::Ui) {
        use std::collections::{HashMap, HashSet};
        
        // Helper function to extract words from text
        fn extract_words(text: &str) -> Vec<String> {
            let stop_words: HashSet<&str> = [
                "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
                "of", "with", "by", "from", "as", "is", "was", "are", "were", "be",
                "been", "being", "have", "has", "had", "do", "does", "did", "will",
                "would", "should", "could", "may", "might", "must", "can", "this",
                "that", "these", "those", "i", "you", "he", "she", "it", "we", "they",
                "what", "which", "who", "when", "where", "why", "how", "all", "each",
                "every", "both", "few", "more", "most", "other", "some", "such", "no",
                "not", "only", "own", "same", "so", "than", "too", "very", "just",
                "www", "youtube", "https", "com", "github", "http", "watch", "conference",
                "commit", "src", "main"
            ].iter().cloned().collect();
            
            text.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 2 && !stop_words.contains(w))
                .map(|w| w.to_string())
                .collect()
        }
        
        // Analyze all text content
        let mut word_freq: HashMap<String, usize> = HashMap::new();
        let mut category_terms: HashMap<String, HashSet<String>> = HashMap::new();
        let mut term_categories: HashMap<String, HashSet<String>> = HashMap::new();
        let mut co_occurrences: HashMap<(String, String), usize> = HashMap::new();
        
        for (cat_name, category) in &self.microfiche.categories {
            let mut cat_words = HashSet::new();
            
            for subcat in &category.subcategories {
                for concept in &subcat.concepts {
                    // Extract words from concept name
                    for word in extract_words(&concept.name) {
                        *word_freq.entry(word.clone()).or_insert(0) += 1;
                        cat_words.insert(word.clone());
                        term_categories.entry(word.clone())
                            .or_insert_with(HashSet::new)
                            .insert(cat_name.clone());
                    }
                    
                    // Extract words from all notes
                    for note in &concept.notes {
                        let words = extract_words(note);
                        for word in &words {
                            *word_freq.entry(word.clone()).or_insert(0) += 1;
                            cat_words.insert(word.clone());
                            term_categories.entry(word.clone())
                                .or_insert_with(HashSet::new)
                                .insert(cat_name.clone());
                        }
                        
                        // Calculate co-occurrences
                        for i in 0..words.len() {
                            for j in (i + 1)..words.len() {
                                if words[i] != words[j] {
                                    let pair = if words[i] < words[j] {
                                        (words[i].clone(), words[j].clone())
                                    } else {
                                        (words[j].clone(), words[i].clone())
                                    };
                                    *co_occurrences.entry(pair).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }
            }
            
            category_terms.insert(cat_name.clone(), cat_words);
        }
        
        // Get top co-occurrences with stable sorting
        let mut top_cooccur: Vec<_> = co_occurrences.iter()
            .map(|(pair, count)| (pair.clone(), *count))
            .collect();
        top_cooccur.sort_by(|a, b| {
            match b.1.cmp(&a.1) {
                std::cmp::Ordering::Equal => a.0.cmp(&b.0),
                other => other,
            }
        });
        
        // Pagination constants
        const ITEMS_PER_PAGE: usize = 10;
        let total_cooccur = top_cooccur.len();
        let total_cooccur_pages = (total_cooccur + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
        
        let stats = self.microfiche.stats();
        let visuals = ui.ctx().style().visuals.clone();
        let accent_color = visuals.hyperlink_color;
        let secondary_color = visuals.selection.stroke.color;
        let tertiary_color = visuals.warn_fg_color;
        let error_color = visuals.error_fg_color;
        
        // Main container
        ui.vertical(|ui| {
            // Header
            ui.heading("Knowledge Statistics & Word Associations");
            ui.separator();
            ui.add_space(5.0);
            
            // Overview panel - this establishes our width
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.heading("Overview");
                ui.separator();
                ui.add_space(5.0);
                
                egui::Grid::new("hierarchy_grid")
                    .num_columns(2)
                    .spacing([20.0, 10.0])
                    .striped(true)
                    .min_col_width(ui.available_width() / 2.0 - 10.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Categories:").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(stats.get("categories").unwrap_or(&0).to_string())
                                .strong().size(15.0).color(accent_color));
                        });
                        ui.end_row();
                        
                        ui.label(egui::RichText::new("Subcategories:").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(stats.get("subcategories").unwrap_or(&0).to_string())
                                .size(15.0).color(secondary_color));
                        });
                        ui.end_row();
                        
                        ui.label(egui::RichText::new("Concepts:").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(stats.get("concepts").unwrap_or(&0).to_string())
                                .size(15.0).color(tertiary_color));
                        });
                        ui.end_row();
                        
                        ui.label(egui::RichText::new("Total Notes:").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(stats.get("total_notes").unwrap_or(&0).to_string())
                                .strong().size(15.0).color(error_color));
                        });
                        ui.end_row();
                        
                        ui.label(egui::RichText::new("Unique Terms:").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(word_freq.len().to_string())
                                .size(15.0).color(accent_color));
                        });
                        ui.end_row();
                    });
            });
            
            ui.add_space(10.0);
            
            // Calculate available height for the two panels
            let available_height = ui.available_height() - 20.0;
            let total_width = ui.available_width();
            let panel_spacing = 10.0;
            let panel_width = (total_width - panel_spacing) / 2.0;
            
            // Side by side panels - using columns for exact sizing
            ui.columns(2, |columns| {
                // Left panel - Term Co-occurrence
                columns[0].vertical(|ui| {
                    ui.set_height(available_height);
                    
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.set_height(available_height);
                        
                        ui.vertical(|ui| {
                            ui.heading("Term Co-occurrences");
                            ui.label("Pairs appearing together");
                            
                            if top_cooccur.is_empty() {
                                ui.separator();
                                ui.centered_and_justified(|ui| {
                                    ui.label(egui::RichText::new("No co-occurrences found")
                                        .size(14.0).color(egui::Color32::GRAY));
                                });
                            } else {
                                ui.separator();
                                
                                // Pagination controls
                                ui.horizontal(|ui| {
                                    if ui.button("◀ Prev").clicked() && self.cooccurrence_page > 0 {
                                        self.cooccurrence_page -= 1;
                                    }
                                    ui.label(format!("Page {} / {}", self.cooccurrence_page + 1, total_cooccur_pages.max(1)));
                                    if ui.button("Next ▶").clicked() && self.cooccurrence_page < total_cooccur_pages.saturating_sub(1) {
                                        self.cooccurrence_page += 1;
                                    }
                                });
                                
                                ui.separator();
                                
                                // Clamp page number
                                if self.cooccurrence_page >= total_cooccur_pages {
                                    self.cooccurrence_page = total_cooccur_pages.saturating_sub(1);
                                }
                                
                                let start_idx = self.cooccurrence_page * ITEMS_PER_PAGE;
                                let end_idx = (start_idx + ITEMS_PER_PAGE).min(total_cooccur);
                                
                                egui::ScrollArea::vertical()
                                    .id_source("cooccurrence_scroll")
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        for ((term1, term2), count) in &top_cooccur[start_idx..end_idx] {
                                            ui.group(|ui| {
                                                ui.set_width(ui.available_width());
                                                ui.horizontal(|ui| {
                                                    ui.strong(egui::RichText::new(term1.as_str()).color(accent_color));
                                                    ui.label("↔");
                                                    ui.strong(egui::RichText::new(term2.as_str()).color(secondary_color));
                                                });
                                                ui.label(egui::RichText::new(format!("{} occurrences", count))
                                                    .size(11.0)
                                                    .color(tertiary_color));
                                                
                                                // Show shared categories in a compact way
                                                let mut pair_categories: HashSet<String> = HashSet::new();
                                                if let Some(cats1) = term_categories.get(term1) {
                                                    if let Some(cats2) = term_categories.get(term2) {
                                                        pair_categories = cats1.intersection(cats2).cloned().collect();
                                                    }
                                                }
                                                
                                                if !pair_categories.is_empty() {
                                                    let mut cat_list: Vec<_> = pair_categories.iter().collect();
                                                    cat_list.sort();
                                                    let cat_display = cat_list.iter().take(3)
                                                        .map(|s| s.as_str())
                                                        .collect::<Vec<_>>()
                                                        .join(", ");
                                                    ui.label(egui::RichText::new(cat_display)
                                                        .size(10.0)
                                                        .color(egui::Color32::GRAY));
                                                }
                                            });
                                            ui.add_space(3.0);
                                        }
                                    });
                            }
                        });
                    });
                });
                
                // Right panel - Category-Term Distribution
                columns[1].vertical(|ui| {
                    ui.set_height(available_height);
                    
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.set_height(available_height);
                        
                        ui.vertical(|ui| {
                            ui.heading("Category-Term Distribution");
                            ui.label("Top terms per category");
                            
                            if category_terms.is_empty() {
                                ui.separator();
                                ui.centered_and_justified(|ui| {
                                    ui.label(egui::RichText::new("No categories yet")
                                        .size(14.0).color(egui::Color32::GRAY));
                                });
                            } else {
                                ui.separator();
                                
                                let mut sorted_cats: Vec<_> = category_terms.iter().collect();
                                sorted_cats.sort_by(|a, b| a.0.cmp(b.0));
                                
                                let total_cats = sorted_cats.len();
                                let total_cat_pages = (total_cats + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
                                
                                // Pagination controls
                                ui.horizontal(|ui| {
                                    if ui.button("◀ Prev").clicked() && self.category_page > 0 {
                                        self.category_page -= 1;
                                    }
                                    ui.label(format!("Page {} / {}", self.category_page + 1, total_cat_pages.max(1)));
                                    if ui.button("Next ▶").clicked() && self.category_page < total_cat_pages.saturating_sub(1) {
                                        self.category_page += 1;
                                    }
                                });
                                
                                ui.separator();
                                
                                // Clamp page number
                                if self.category_page >= total_cat_pages {
                                    self.category_page = total_cat_pages.saturating_sub(1);
                                }
                                
                                let start_idx = self.category_page * ITEMS_PER_PAGE;
                                let end_idx = (start_idx + ITEMS_PER_PAGE).min(total_cats);
                                
                                egui::ScrollArea::vertical()
                                    .id_source("category_terms_scroll")
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        for (cat_name, terms) in &sorted_cats[start_idx..end_idx] {
                                            ui.group(|ui| {
                                                ui.set_width(ui.available_width());
                                                ui.strong(egui::RichText::new(cat_name.as_str()).color(accent_color));
                                                ui.label(egui::RichText::new(format!("{} unique terms", terms.len()))
                                                    .size(11.0)
                                                    .color(egui::Color32::GRAY));
                                                ui.separator();
                                                
                                                // Get top terms for this category with stable sorting
                                                let mut cat_terms: Vec<_> = terms.iter()
                                                    .filter_map(|t| word_freq.get(t).map(|f| (t.clone(), *f)))
                                                    .collect();
                                                cat_terms.sort_by(|a, b| {
                                                    match b.1.cmp(&a.1) {
                                                        std::cmp::Ordering::Equal => a.0.cmp(&b.0),
                                                        other => other,
                                                    }
                                                });
                                                
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.set_max_width(ui.available_width());
                                                    for (term, freq) in cat_terms.iter().take(12) {
                                                        let tag = format!("{} ({})", term, freq);
                                                        ui.label(egui::RichText::new(tag)
                                                            .size(11.0)
                                                            .color(secondary_color)
                                                            .background_color(egui::Color32::from_rgba_unmultiplied(
                                                                secondary_color.r(),
                                                                secondary_color.g(),
                                                                secondary_color.b(),
                                                                40
                                                            )));
                                                    }
                                                });
                                            });
                                            ui.add_space(3.0);
                                        }
                                    });
                            }
                        });
                    });
                });
            });
        });
    }
}

impl eframe::App for MicroficheApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.current_theme.apply(ctx);
        
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.render_top_bar(ui, ctx);
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
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
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };
    
    eframe::run_native(
        "Fisha GUI",
        options,
        Box::new(|cc| Ok(Box::new(MicroficheApp::new(cc)))),
    )
}

fn load_icon() -> egui::IconData {
    const ICON_DATA: &str = include_str!("../assets/icon_rgba.txt");
    
    let mut lines = ICON_DATA.lines();
    
    // Parse dimensions
    let dims_line = lines.next().expect("Missing dimensions line");
    let mut dims = dims_line.split(',');
    let width: u32 = dims.next().unwrap().trim().parse().unwrap();
    let height: u32 = dims.next().unwrap().trim().parse().unwrap();
    
    // Parse RGBA data
    let rgba_line = lines.next().expect("Missing RGBA data line");
    let rgba: Vec<u8> = rgba_line
        .split(',')
        .map(|s| s.trim().parse().expect("Invalid RGBA value"))
        .collect();
    
    // Verify size
    assert_eq!(rgba.len(), (width * height * 4) as usize, "RGBA data size mismatch");
    
    egui::IconData {
        rgba,
        width,
        height,
    }
}