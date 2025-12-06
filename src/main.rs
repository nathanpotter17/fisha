//! Fisha - Knowledge Management Application (iced 0.12, single file)

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row, scrollable,
    text, text_input, vertical_rule, vertical_space, Column,
};
use iced::{executor, Application, Background, Border, Color, Command, Element, Length, Settings, Theme};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

fn main() -> iced::Result {
    Fisha::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            min_size: Some(iced::Size::new(800.0, 600.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}

// ============================================================================
// Data Model
// ============================================================================

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

#[derive(Debug, Serialize, Deserialize)]
struct CsvRow {
    #[serde(rename = "Category")]
    category: String,
    #[serde(rename = "Subcategory")]
    subcategory: String,
    #[serde(rename = "Concept")]
    concept: String,
    #[serde(rename = "Note")]
    note: String,
}

#[derive(Debug, Clone, Default)]
struct Microfiche {
    categories: HashMap<String, Category>,
}

#[derive(Debug, Clone)]
struct SearchResult {
    category: String,
    subcategory: String,
    concept: String,
    note: String,
}

#[derive(Debug, Clone, Default)]
struct Stats {
    categories: usize,
    subcategories: usize,
    concepts: usize,
    notes: usize,
}

impl Microfiche {
    fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let mut fiche = Self::default();
        let mut rdr = csv::Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let row: CsvRow = result?;
            fiche.add_entry(&row.category, &row.subcategory, &row.concept, row.note);
        }
        Ok(fiche)
    }

    fn to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut wtr = csv::Writer::from_path(path)?;
        wtr.write_record(["Category", "Subcategory", "Concept", "Note"])?;
        let mut cat_names: Vec<_> = self.categories.keys().collect();
        cat_names.sort();
        for cat_name in cat_names {
            let category = &self.categories[cat_name];
            for subcat in &category.subcategories {
                for concept in &subcat.concepts {
                    for note in &concept.notes {
                        wtr.write_record([cat_name, &subcat.name, &concept.name, note])?;
                    }
                }
            }
        }
        wtr.flush()?;
        Ok(())
    }

    fn add_entry(&mut self, cat: &str, sub: &str, con: &str, note: String) {
        let category = self.categories.entry(cat.to_string()).or_insert_with(|| Category {
            name: cat.to_string(),
            subcategories: Vec::new(),
        });
        let subcat = match category.subcategories.iter_mut().find(|s| s.name == sub) {
            Some(s) => s,
            None => {
                category.subcategories.push(Subcategory {
                    name: sub.to_string(),
                    concepts: Vec::new(),
                });
                category.subcategories.last_mut().unwrap()
            }
        };
        let concept = match subcat.concepts.iter_mut().find(|c| c.name == con) {
            Some(c) => c,
            None => {
                subcat.concepts.push(Concept {
                    name: con.to_string(),
                    notes: Vec::new(),
                });
                subcat.concepts.last_mut().unwrap()
            }
        };
        concept.notes.push(note);
    }

    fn delete_note(&mut self, cat: &str, sub: &str, con: &str, note: &str) -> bool {
        let Some(category) = self.categories.get_mut(cat) else { return false };
        let Some(subcat) = category.subcategories.iter_mut().find(|s| s.name == sub) else { return false };
        let Some(concept) = subcat.concepts.iter_mut().find(|c| c.name == con) else { return false };
        let Some(pos) = concept.notes.iter().position(|n| n == note) else { return false };
        concept.notes.remove(pos);
        if concept.notes.is_empty() { subcat.concepts.retain(|c| !c.notes.is_empty()); }
        if subcat.concepts.is_empty() { category.subcategories.retain(|s| !s.concepts.is_empty()); }
        if category.subcategories.is_empty() { self.categories.remove(cat); }
        true
    }

    fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let parts: Vec<&str> = query_lower.split_whitespace().collect();
        
        if parts.is_empty() {
            return Vec::new();
        }
        
        let mut results = Vec::with_capacity(100);
        
        // Check for exact category name match
        for (cat_name, category) in &self.categories {
            let cat_lower = cat_name.to_lowercase();
            
            if parts.len() == 1 && cat_lower == parts[0] {
                for subcat in &category.subcategories {
                    for concept in &subcat.concepts {
                        for note in &concept.notes {
                            results.push(SearchResult {
                                category: cat_name.clone(),
                                subcategory: subcat.name.clone(),
                                concept: concept.name.clone(),
                                note: note.clone(),
                            });
                        }
                    }
                }
                return results;
            }
        }
        
        // Check for exact subcategory name match (handles multi-word)
        let query_joined = parts.join(" ");
        for (cat_name, category) in &self.categories {
            for subcat in &category.subcategories {
                if subcat.name.to_lowercase() == query_joined {
                    for concept in &subcat.concepts {
                        for note in &concept.notes {
                            results.push(SearchResult {
                                category: cat_name.clone(),
                                subcategory: subcat.name.clone(),
                                concept: concept.name.clone(),
                                note: note.clone(),
                            });
                        }
                    }
                    return results;
                }
            }
        }
        
        // Check for exact concept name match
        for (cat_name, category) in &self.categories {
            for subcat in &category.subcategories {
                for concept in &subcat.concepts {
                    if concept.name.to_lowercase() == query_joined {
                        for note in &concept.notes {
                            results.push(SearchResult {
                                category: cat_name.clone(),
                                subcategory: subcat.name.clone(),
                                concept: concept.name.clone(),
                                note: note.clone(),
                            });
                        }
                        return results;
                    }
                }
            }
        }
        
        // Content search - ALL terms must match
        for (cat_name, category) in &self.categories {
            let cat_lower = cat_name.to_lowercase();
            
            for subcat in &category.subcategories {
                let sub_lower = subcat.name.to_lowercase();
                
                for concept in &subcat.concepts {
                    let con_lower = concept.name.to_lowercase();
                    
                    for note in &concept.notes {
                        let note_lower = note.to_lowercase();
                        
                        // Check if ALL parts match somewhere
                        let all_match = parts.iter().all(|part| {
                            cat_lower.contains(part) 
                                || sub_lower.contains(part) 
                                || con_lower.contains(part) 
                                || note_lower.contains(part)
                        });
                        
                        if all_match {
                            results.push(SearchResult {
                                category: cat_name.clone(),
                                subcategory: subcat.name.clone(),
                                concept: concept.name.clone(),
                                note: note.clone(),
                            });
                            
                            if results.len() >= 200 { return results; }
                        }
                    }
                }
            }
        }
        
        results
    }

    fn stats(&self) -> Stats {
        let mut s = Stats { categories: self.categories.len(), ..Default::default() };
        for category in self.categories.values() {
            s.subcategories += category.subcategories.len();
            for subcat in &category.subcategories {
                s.concepts += subcat.concepts.len();
                for concept in &subcat.concepts {
                    s.notes += concept.notes.len();
                }
            }
        }
        s
    }

    fn sorted_category_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.categories.keys().cloned().collect();
        names.sort();
        names
    }
}

// ============================================================================
// Theme
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
struct Palette {
    background: Color,
    surface: Color,
    surface_hover: Color,
    surface_active: Color,
    border: Color,
    text_primary: Color,
    text_secondary: Color,
    accent: Color,
    accent_hover: Color,
    success: Color,
    warning: Color,
    error: Color,
}

const MONOKAI: Palette = Palette {
    background: Color::from_rgb(0.153, 0.157, 0.133),
    surface: Color::from_rgb(0.192, 0.196, 0.173),
    surface_hover: Color::from_rgb(0.235, 0.239, 0.216),
    surface_active: Color::from_rgb(0.294, 0.298, 0.267),
    border: Color::from_rgb(0.353, 0.357, 0.322),
    text_primary: Color::from_rgb(0.973, 0.973, 0.949),
    text_secondary: Color::from_rgb(0.647, 0.647, 0.627),
    accent: Color::from_rgb(0.976, 0.149, 0.447),
    accent_hover: Color::from_rgb(0.400, 0.851, 0.937),
    success: Color::from_rgb(0.651, 0.886, 0.314),
    warning: Color::from_rgb(0.902, 0.859, 0.455),
    error: Color::from_rgb(0.976, 0.149, 0.447),
};

const TOMORROW_BLUE: Palette = Palette {
    background: Color::from_rgb(0.000, 0.114, 0.200),
    surface: Color::from_rgb(0.000, 0.169, 0.278),
    surface_hover: Color::from_rgb(0.027, 0.216, 0.341),
    surface_active: Color::from_rgb(0.047, 0.275, 0.451),
    border: Color::from_rgb(0.067, 0.333, 0.529),
    text_primary: Color::from_rgb(0.906, 0.910, 0.922),
    text_secondary: Color::from_rgb(0.600, 0.651, 0.702),
    accent: Color::from_rgb(0.490, 0.682, 0.776),
    accent_hover: Color::from_rgb(0.600, 0.780, 0.860),
    success: Color::from_rgb(0.710, 0.808, 0.659),
    warning: Color::from_rgb(1.000, 0.800, 0.400),
    error: Color::from_rgb(1.000, 0.400, 0.400),
};

const DARK_PLUS: Palette = Palette {
    background: Color::from_rgb(0.118, 0.118, 0.118),
    surface: Color::from_rgb(0.145, 0.145, 0.149),
    surface_hover: Color::from_rgb(0.176, 0.176, 0.176),
    surface_active: Color::from_rgb(0.235, 0.235, 0.235),
    border: Color::from_rgb(0.294, 0.294, 0.294),
    text_primary: Color::from_rgb(0.831, 0.831, 0.831),
    text_secondary: Color::from_rgb(0.600, 0.600, 0.600),
    accent: Color::from_rgb(0.306, 0.635, 0.902),
    accent_hover: Color::from_rgb(0.400, 0.720, 0.950),
    success: Color::from_rgb(0.353, 0.788, 0.353),
    warning: Color::from_rgb(0.808, 0.569, 0.471),
    error: Color::from_rgb(0.957, 0.278, 0.278),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum FishaTheme {
    #[default]
    DarkPlus,
    Monokai,
    TomorrowBlue,
}

impl FishaTheme {
    const ALL: [Self; 3] = [Self::Monokai, Self::TomorrowBlue, Self::DarkPlus];
    fn palette(&self) -> Palette {
        match self {
            Self::Monokai => MONOKAI,
            Self::TomorrowBlue => TOMORROW_BLUE,
            Self::DarkPlus => DARK_PLUS,
        }
    }
    fn name(&self) -> &'static str {
        match self {
            Self::Monokai => "Monokai",
            Self::TomorrowBlue => "Tomorrow Blue",
            Self::DarkPlus => "Dark+",
        }
    }
}

impl std::fmt::Display for FishaTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

fn alpha(c: Color, a: f32) -> Color { Color { a, ..c } }

// ============================================================================
// Style Sheets
// ============================================================================

struct ContainerStyle(Palette, bool); // bool = is_card
impl container::StyleSheet for ContainerStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(if self.1 { self.0.surface } else { self.0.background })),
            text_color: Some(self.0.text_primary),
            border: if self.1 { Border { color: self.0.border, width: 1.0, radius: 4.0.into() } } else { Border::default() },
            ..Default::default()
        }
    }
}

struct ButtonStyle(Palette, ButtonKind);
#[derive(Clone, Copy)]
enum ButtonKind { Primary, Secondary, Nav(bool), Danger, ListItem(bool) }

impl button::StyleSheet for ButtonStyle {
    type Style = Theme;
    fn active(&self, _: &Self::Style) -> button::Appearance {
        let p = &self.0;
        let (bg, text, border_color, border_width) = match self.1 {
            ButtonKind::Primary => (p.accent, p.text_primary, Color::TRANSPARENT, 0.0),
            ButtonKind::Secondary => (p.surface, p.text_primary, p.border, 1.0),
            ButtonKind::Nav(sel) => (if sel { p.surface_active } else { Color::TRANSPARENT }, if sel { p.accent } else { p.text_primary }, Color::TRANSPARENT, 0.0),
            ButtonKind::Danger => (alpha(p.error, 0.2), p.error, p.error, 1.0),
            ButtonKind::ListItem(sel) => (if sel { alpha(p.accent, 0.2) } else { Color::TRANSPARENT }, p.text_primary, if sel { p.accent } else { Color::TRANSPARENT }, if sel { 1.0 } else { 0.0 }),
        };
        button::Appearance {
            background: Some(Background::Color(bg)),
            text_color: text,
            border: Border { color: border_color, width: border_width, radius: 4.0.into() },
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        let p = &self.0;
        let (bg, text, border_color) = match self.1 {
            ButtonKind::Primary => (p.accent_hover, p.text_primary, Color::TRANSPARENT),
            ButtonKind::Secondary => (p.surface_hover, p.text_primary, p.accent),
            ButtonKind::Nav(_) => (p.surface_hover, p.text_primary, Color::TRANSPARENT),
            ButtonKind::Danger => (alpha(p.error, 0.4), p.error, p.error),
            ButtonKind::ListItem(_) => (p.surface_hover, p.text_primary, Color::TRANSPARENT),
        };
        button::Appearance {
            background: Some(Background::Color(bg)),
            text_color: text,
            border: Border { color: border_color, width: if matches!(self.1, ButtonKind::Secondary | ButtonKind::Danger) { 1.0 } else { 0.0 }, radius: 4.0.into() },
            ..Default::default()
        }
    }
    fn pressed(&self, s: &Self::Style) -> button::Appearance {
        let mut a = self.hovered(s);
        a.background = Some(Background::Color(self.0.surface_active));
        a
    }
}

struct InputStyle(Palette);
impl text_input::StyleSheet for InputStyle {
    type Style = Theme;
    fn active(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(self.0.surface),
            border: Border { color: self.0.border, width: 1.0, radius: 4.0.into() },
            icon_color: self.0.text_secondary,
        }
    }
    fn focused(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(self.0.surface_hover),
            border: Border { color: self.0.accent, width: 1.0, radius: 4.0.into() },
            icon_color: self.0.text_secondary,
        }
    }
    fn hovered(&self, s: &Self::Style) -> text_input::Appearance {
        let mut a = self.active(s);
        a.border.color = self.0.accent;
        a
    }
    fn disabled(&self, s: &Self::Style) -> text_input::Appearance { self.active(s) }
    fn placeholder_color(&self, _: &Self::Style) -> Color { self.0.text_secondary }
    fn value_color(&self, _: &Self::Style) -> Color { self.0.text_primary }
    fn selection_color(&self, _: &Self::Style) -> Color { alpha(self.0.accent, 0.3) }
    fn disabled_color(&self, _: &Self::Style) -> Color { self.0.text_secondary }
}

struct ScrollStyle(Palette);
impl scrollable::StyleSheet for ScrollStyle {
    type Style = Theme;
    fn active(&self, _: &Self::Style) -> scrollable::Appearance {
        scrollable::Appearance {
            container: Default::default(),
            scrollbar: scrollable::Scrollbar {
                background: Some(Background::Color(self.0.surface)),
                border: Border::default(),
                scroller: scrollable::Scroller { color: self.0.surface_hover, border: Border { radius: 4.0.into(), ..Default::default() } },
            },
            gap: None,
        }
    }
    fn hovered(&self, _: &Self::Style, _: bool) -> scrollable::Appearance {
        let mut a = self.active(&Theme::Dark);
        a.scrollbar.scroller.color = self.0.surface_active;
        a
    }
}

struct PickStyle(Palette);
impl pick_list::StyleSheet for PickStyle {
    type Style = Theme;
    fn active(&self, _: &Self::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            background: Background::Color(self.0.surface),
            border: Border { color: self.0.border, width: 1.0, radius: 4.0.into() },
            text_color: self.0.text_primary,
            placeholder_color: self.0.text_secondary,
            handle_color: self.0.text_secondary,
        }
    }
    fn hovered(&self, _: &Self::Style) -> pick_list::Appearance {
        let mut a = self.active(&Theme::Dark);
        a.border.color = self.0.accent;
        a.background = Background::Color(self.0.surface_hover);
        a
    }
}

struct MenuStyle(Palette);
impl iced::overlay::menu::StyleSheet for MenuStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> iced::overlay::menu::Appearance {
        iced::overlay::menu::Appearance {
            background: Background::Color(self.0.surface),
            border: Border { color: self.0.border, width: 1.0, radius: 4.0.into() },
            text_color: self.0.text_primary,
            selected_background: Background::Color(alpha(self.0.accent, 0.3)),
            selected_text_color: self.0.accent,
        }
    }
}

// ============================================================================
// Application
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ViewMode { #[default] Browse, Search, Create, Stats }

#[derive(Debug, Clone)]
enum Message {
    OpenFile, FileOpened(Result<(String, Microfiche), String>),
    SaveFile, SaveFileAs, FileSaved(Result<String, String>),
    SetViewMode(ViewMode), SetTheme(FishaTheme),
    SelectCategory(String), SelectSubcategory(String),
    SearchQueryChanged(String),
    FormCategoryChanged(String), FormSubcategoryChanged(String),
    FormConceptChanged(String), FormNoteChanged(String),
    SubmitEntry, ClearForm,
    DeleteNote { category: String, subcategory: String, concept: String, note: String },
    EditNote { category: String, subcategory: String, concept: String, note: String },
    UseAsTemplate { category: String, subcategory: String, concept: String },
}

struct Fisha {
    microfiche: Microfiche,
    current_file: Option<String>,
    view_mode: ViewMode,
    theme: FishaTheme,
    selected_category: Option<String>,
    selected_subcategory: Option<String>,
    search_query: String,
    search_results: Vec<SearchResult>,
    form_category: String,
    form_subcategory: String,
    form_concept: String,
    form_note: String,
    status_message: String,
}

impl Application for Fisha {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_: ()) -> (Self, Command<Message>) {
        (Self {
            microfiche: Microfiche::from_csv("microfiche.csv").unwrap_or_default(),
            current_file: Some("microfiche.csv".to_string()),
            view_mode: ViewMode::default(),
            theme: FishaTheme::default(),
            selected_category: None,
            selected_subcategory: None,
            search_query: String::new(),
            search_results: Vec::new(),
            form_category: String::new(),
            form_subcategory: String::new(),
            form_concept: String::new(),
            form_note: String::new(),
            status_message: String::new(),
        }, Command::none())
    }

    fn title(&self) -> String { "Fisha".into() }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::OpenFile => return Command::perform(async {
                match rfd::AsyncFileDialog::new().add_filter("CSV", &["csv"]).pick_file().await {
                    Some(f) => {
                        let p = f.path().to_string_lossy().to_string();
                        Microfiche::from_csv(&p).map(|m| (p, m)).map_err(|e| e.to_string())
                    }
                    None => Err("Cancelled".into())
                }
            }, Message::FileOpened),
            Message::FileOpened(Ok((path, m))) => {
                self.microfiche = m;
                self.current_file = Some(path.clone());
                self.status_message = format!("Loaded: {}", path);
                self.selected_category = None;
                self.selected_subcategory = None;
            }
            Message::FileOpened(Err(e)) => if e != "Cancelled" { self.status_message = e; },
            Message::SaveFile => if let Some(p) = self.current_file.clone() {
                let m = self.microfiche.clone();
                return Command::perform(async move {
                    m.to_csv(&p).map(|_| p).map_err(|e| e.to_string())
                }, Message::FileSaved);
            } else { return self.update(Message::SaveFileAs); },
            Message::SaveFileAs => {
                let m = self.microfiche.clone();
                return Command::perform(async move {
                    match rfd::AsyncFileDialog::new().add_filter("CSV", &["csv"]).save_file().await {
                        Some(f) => {
                            let p = f.path().to_string_lossy().to_string();
                            m.to_csv(&p).map(|_| p).map_err(|e| e.to_string())
                        }
                        None => Err("Cancelled".into())
                    }
                }, Message::FileSaved);
            }
            Message::FileSaved(Ok(p)) => { self.current_file = Some(p.clone()); self.status_message = format!("Saved: {}", p); }
            Message::FileSaved(Err(e)) => if e != "Cancelled" { self.status_message = e; },
            Message::SetViewMode(m) => self.view_mode = m,
            Message::SetTheme(t) => self.theme = t,
            Message::SelectCategory(c) => { self.selected_category = Some(c); self.selected_subcategory = None; }
            Message::SelectSubcategory(s) => self.selected_subcategory = Some(s),
            Message::SearchQueryChanged(q) => { self.search_query = q; self.search_results = self.microfiche.search(&self.search_query); }
            Message::FormCategoryChanged(v) => self.form_category = v,
            Message::FormSubcategoryChanged(v) => self.form_subcategory = v,
            Message::FormConceptChanged(v) => self.form_concept = v,
            Message::FormNoteChanged(v) => self.form_note = v,
            Message::SubmitEntry => {
                if self.form_category.is_empty() || self.form_subcategory.is_empty() || self.form_concept.is_empty() || self.form_note.trim().is_empty() {
                    self.status_message = "All fields required".into();
                } else {
                    self.microfiche.add_entry(&self.form_category, &self.form_subcategory, &self.form_concept, self.form_note.trim().into());
                    self.status_message = "Entry created".into();
                    self.form_category.clear(); self.form_subcategory.clear(); self.form_concept.clear(); self.form_note.clear();
                }
            }
            Message::ClearForm => { self.form_category.clear(); self.form_subcategory.clear(); self.form_concept.clear(); self.form_note.clear(); }
            Message::DeleteNote { category, subcategory, concept, note } => {
                if self.microfiche.delete_note(&category, &subcategory, &concept, &note) {
                    self.status_message = "Deleted".into();
                    if self.view_mode == ViewMode::Search { self.search_results = self.microfiche.search(&self.search_query); }
                    if !self.microfiche.categories.contains_key(&category) { self.selected_category = None; self.selected_subcategory = None; }
                }
            }
            Message::EditNote { category, subcategory, concept, note } => {
                if self.microfiche.delete_note(&category, &subcategory, &concept, &note) {
                    self.form_category = category; self.form_subcategory = subcategory; self.form_concept = concept; self.form_note = note;
                    self.view_mode = ViewMode::Create;
                    self.status_message = "Edit and submit".into();
                }
            }
            Message::UseAsTemplate { category, subcategory, concept } => {
                self.form_category = category; self.form_subcategory = subcategory; self.form_concept = concept; self.form_note.clear();
                self.view_mode = ViewMode::Create;
                self.status_message = "Template loaded".into();
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let p = self.theme.palette();
        let content: Element<Message> = match self.view_mode {
            ViewMode::Browse => self.view_browse(),
            ViewMode::Search => self.view_search(),
            ViewMode::Create => self.view_create(),
            ViewMode::Stats => self.view_stats(),
        };
        
        let layout: Element<Message> = column![
            self.view_top_bar(),
            container(content).width(Length::Fill).height(Length::Fill),
            self.view_status_bar()
        ].into();
        
        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, false))))
            .into()
    }
}

impl Fisha {
    fn view_top_bar(&self) -> Element<Message> {
        let p = self.theme.palette();
        let btn = |label: &str, msg: Message| -> Element<Message> {
            button(text(label).size(13)).on_press(msg)
                .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))).into()
        };
        let nav = |label: &str, mode: ViewMode| -> Element<Message> {
            button(text(label).size(13)).on_press(Message::SetViewMode(mode))
                .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Nav(self.view_mode == mode))))).into()
        };
        container(row![
            row![btn("Open", Message::OpenFile), btn("Save", Message::SaveFile), btn("Save As", Message::SaveFileAs)].spacing(4),
            vertical_rule(1),
            row![nav("Browse", ViewMode::Browse), nav("Search", ViewMode::Search), nav("Create", ViewMode::Create), nav("Stats", ViewMode::Stats)].spacing(4),
            horizontal_space(),
            pick_list(&FishaTheme::ALL[..], Some(self.theme), Message::SetTheme).text_size(13)
                .style(iced::theme::PickList::Custom(std::rc::Rc::new(PickStyle(p)), std::rc::Rc::new(MenuStyle(p)))),
        ].spacing(16).padding(8).align_items(iced::Alignment::Center))
        .width(Length::Fill)
        .height(Length::Fixed(45.0))
        .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, true)))).into()
    }

    fn view_status_bar(&self) -> Element<Message> {
        let p = self.theme.palette();
        container(row![
            text(&self.status_message).size(12).style(p.text_secondary),
            horizontal_space(),
            text(self.theme.name()).size(12).style(p.text_secondary),
        ].padding(8))
        .width(Length::Fill)
        .height(Length::Fixed(30.0))
        .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, true)))).into()
    }

    fn view_browse(&self) -> Element<Message> {
        let p = self.theme.palette();
        let cats = self.microfiche.sorted_category_names();
        let cat_list: Element<Message> = if cats.is_empty() {
            text("No categories").size(13).style(p.text_secondary).into()
        } else {
            Column::with_children(cats.iter().map(|n| {
                let sel = self.selected_category.as_ref() == Some(n);
                button(text(n).size(13)).on_press(Message::SelectCategory(n.clone())).width(Length::Fill)
                    .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::ListItem(sel))))).into()
            }).collect::<Vec<_>>()).spacing(2).into()
        };
        let cat_panel = container(column![
            text("Categories").size(14).style(p.accent), horizontal_rule(1),
            scrollable(container(cat_list).padding(4)).height(Length::Fill)
                .style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p)))),
        ].spacing(8).padding(12)).width(200).height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, true))));

        let sub_panel: Element<Message> = if let Some(cat_name) = &self.selected_category {
            if let Some(cat) = self.microfiche.categories.get(cat_name) {
                let sub_list: Vec<Element<Message>> = cat.subcategories.iter().map(|s| {
                    let sel = self.selected_subcategory.as_ref() == Some(&s.name);
                    button(text(&s.name).size(13)).on_press(Message::SelectSubcategory(s.name.clone())).width(Length::Fill)
                        .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::ListItem(sel))))).into()
                }).collect();
                container(column![
                    text("Subcategories").size(14).style(p.accent), horizontal_rule(1),
                    scrollable(Column::with_children(sub_list).spacing(2).padding(4)).height(Length::Fill)
                        .style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p)))),
                ].spacing(8).padding(12)).width(200).height(Length::Fill)
                    .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, true)))).into()
            } else { horizontal_space().into() }
        } else { horizontal_space().into() };

        let main: Element<Message> = match (&self.selected_category, &self.selected_subcategory) {
            (Some(cn), Some(sn)) => {
                if let Some(cat) = self.microfiche.categories.get(cn) {
                    if let Some(sub) = cat.subcategories.iter().find(|s| &s.name == sn) {
                        let cards: Vec<Element<Message>> = sub.concepts.iter().map(|c| self.view_concept(cn, sn, c)).collect();
                        column![
                            text(format!("{} > {}", cn, sn)).size(16).style(p.accent), horizontal_rule(1),
                            scrollable(Column::with_children(cards).spacing(12).padding(8)).height(Length::Fill)
                                .style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p)))),
                        ].spacing(8).into()
                    } else { self.empty_state("Not found") }
                } else { self.empty_state("Not found") }
            }
            (Some(_), None) => self.empty_state("Select subcategory"),
            _ => if self.microfiche.categories.is_empty() {
                column![vertical_space(), text("No data").size(16).style(p.text_secondary), vertical_space().height(16),
                    button(text("Open File").size(14)).on_press(Message::OpenFile)
                        .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Primary)))), vertical_space(),
                ].align_items(iced::Alignment::Center).width(Length::Fill).height(Length::Fill).into()
            } else { self.empty_state("Select category") }
        };

        row![cat_panel, sub_panel, container(main).width(Length::Fill).height(Length::Fill).padding(12)
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, true))))
        ].spacing(8).padding(8).height(Length::Fill).into()
    }

    fn view_concept(&self, cat: &str, sub: &str, concept: &Concept) -> Element<Message> {
        let p = self.theme.palette();
        let notes: Vec<Element<Message>> = concept.notes.iter().map(|n| {
            let (c1, s1, co1) = (cat.to_string(), sub.to_string(), concept.name.clone());
            let (c2, s2, co2, n2) = (c1.clone(), s1.clone(), co1.clone(), n.clone());
            let (c3, s3, co3, n3) = (c1.clone(), s1.clone(), co1.clone(), n.clone());
            container(column![
                text(n).size(13), vertical_space().height(8),
                row![
                    button(text("Template").size(11)).on_press(Message::UseAsTemplate { category: c1, subcategory: s1, concept: co1 })
                        .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
                    button(text("Edit").size(11)).on_press(Message::EditNote { category: c2, subcategory: s2, concept: co2, note: n2 })
                        .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
                    button(text("Delete").size(11)).on_press(Message::DeleteNote { category: c3, subcategory: s3, concept: co3, note: n3 })
                        .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Danger)))),
                ].spacing(8),
            ].padding(8)).style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, true)))).into()
        }).collect();
        container(column![
            text(&concept.name).size(14).style(p.accent), horizontal_rule(1),
            Column::with_children(notes).spacing(8),
        ].spacing(8).padding(8)).style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, true)))).into()
    }

    fn view_search(&self) -> Element<Message> {
        let p = self.theme.palette();
        let results: Vec<Element<Message>> = self.search_results.iter().map(|r| {
            let (c1, s1, co1) = (r.category.clone(), r.subcategory.clone(), r.concept.clone());
            let (c2, s2, co2, n2) = (c1.clone(), s1.clone(), co1.clone(), r.note.clone());
            let (c3, s3, co3, n3) = (c1.clone(), s1.clone(), co1.clone(), r.note.clone());
            container(column![
                text(format!("{} > {} > {}", r.category, r.subcategory, r.concept)).size(12).style(p.accent),
                text(&r.note).size(13), vertical_space().height(8),
                row![
                    button(text("Template").size(11)).on_press(Message::UseAsTemplate { category: c1, subcategory: s1, concept: co1 })
                        .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
                    button(text("Edit").size(11)).on_press(Message::EditNote { category: c2, subcategory: s2, concept: co2, note: n2 })
                        .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
                    button(text("Delete").size(11)).on_press(Message::DeleteNote { category: c3, subcategory: s3, concept: co3, note: n3 })
                        .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Danger)))),
                ].spacing(8),
            ].spacing(4).padding(12)).style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, true)))).into()
        }).collect();
        container(column![
            row![text("Search:").size(14), text_input("Type to search...", &self.search_query).on_input(Message::SearchQueryChanged)
                .style(iced::theme::TextInput::Custom(Box::new(InputStyle(p)))).width(Length::Fill)].spacing(12).align_items(iced::Alignment::Center),
            vertical_space().height(8),
            text(format!("Found {} results", self.search_results.len())).size(13).style(p.text_secondary),
            horizontal_rule(1),
            scrollable(Column::with_children(results).spacing(8).padding(8)).height(Length::Fill)
                .style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p)))),
        ].spacing(8).padding(16)).width(Length::Fill).height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, false)))).into()
    }

    fn view_create(&self) -> Element<Message> {
        let p = self.theme.palette();
        let field = |label: &str, val: &str, msg: fn(String) -> Message| -> Element<Message> {
            column![text(label).size(13).style(p.text_secondary),
                text_input("", val).on_input(msg).style(iced::theme::TextInput::Custom(Box::new(InputStyle(p))))
            ].spacing(4).into()
        };
        container(scrollable(container(column![
            text("Create New Entry").size(18).style(p.accent), vertical_space().height(16),
            field("Category", &self.form_category, Message::FormCategoryChanged),
            field("Subcategory", &self.form_subcategory, Message::FormSubcategoryChanged),
            field("Concept", &self.form_concept, Message::FormConceptChanged),
            field("Note", &self.form_note, Message::FormNoteChanged),
            vertical_space().height(16),
            row![
                button(text("Create").size(14)).on_press(Message::SubmitEntry)
                    .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Primary)))),
                button(text("Clear").size(14)).on_press(Message::ClearForm)
                    .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
            ].spacing(12),
        ].spacing(12).max_width(600)).padding(24).center_x()).height(Length::Fill)
            .style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p)))))
            .width(Length::Fill).height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, false)))).into()
    }

    fn view_stats(&self) -> Element<Message> {
        let p = self.theme.palette();
        let s = self.microfiche.stats();
        let stat = |label: &str, val: usize, color: Color| -> Element<Message> {
            row![text(label).size(14).width(Length::FillPortion(1)), text(val.to_string()).size(16).style(color).width(Length::FillPortion(1))].padding(8).into()
        };
        let overview = container(column![
            text("Overview").size(16).style(p.accent), horizontal_rule(1),
            stat("Categories", s.categories, p.accent),
            stat("Subcategories", s.subcategories, p.warning),
            stat("Concepts", s.concepts, p.success),
            stat("Total Notes", s.notes, p.error),
        ].spacing(4).padding(16)).style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, true))));

        let mut cat_stats: Vec<_> = self.microfiche.categories.iter().map(|(n, c)| {
            let notes: usize = c.subcategories.iter().flat_map(|s| &s.concepts).map(|c| c.notes.len()).sum();
            (n.clone(), c.subcategories.len(), notes)
        }).collect();
        cat_stats.sort_by(|a, b| b.2.cmp(&a.2));
        let cat_rows: Vec<Element<Message>> = cat_stats.iter().take(10).map(|(n, subs, notes)| {
            row![text(n).size(13).width(Length::FillPortion(2)),
                text(format!("{} subs", subs)).size(12).style(p.text_secondary).width(Length::FillPortion(1)),
                text(format!("{} notes", notes)).size(12).style(p.accent).width(Length::FillPortion(1)),
            ].padding(4).into()
        }).collect();
        let breakdown = container(column![
            text("Top Categories").size(16).style(p.accent), horizontal_rule(1),
            Column::with_children(cat_rows).spacing(2),
        ].spacing(8).padding(16)).style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, true))));

        container(scrollable(column![overview, vertical_space().height(16), breakdown].spacing(8).padding(16).max_width(800))
            .height(Length::Fill).style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p)))))
            .width(Length::Fill).height(Length::Fill).center_x()
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, false)))).into()
    }

    fn empty_state(&self, msg: &str) -> Element<Message> {
        let p = self.theme.palette();
        container(text(msg).size(14).style(p.text_secondary))
            .width(Length::Fill).height(Length::Fill).center_x().center_y().into()
    }
}