#![windows_subsystem = "windows"]
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row, scrollable,
    text, text_input, vertical_space, Column,
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
        let query_joined = parts.join(" ");
        
        for (cat_name, category) in &self.categories {
            let cat_lower = cat_name.to_lowercase();
            
            for subcat in &category.subcategories {
                let sub_lower = subcat.name.to_lowercase();
                
                for concept in &subcat.concepts {
                    let con_lower = concept.name.to_lowercase();
                    
                    for note in &concept.notes {
                        let note_lower = note.to_lowercase();
                        
                        let all_match = parts.iter().all(|part| {
                            cat_lower.contains(part) 
                                || sub_lower.contains(part) 
                                || con_lower.contains(part) 
                                || note_lower.contains(part)
                        });
                        
                        if all_match || cat_lower == query_joined || sub_lower == query_joined || con_lower == query_joined {
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
    text_muted: Color,
    accent: Color,
    accent_hover: Color,
    accent_secondary: Color,
    accent_tertiary: Color,
    success: Color,
    warning: Color,
    error: Color,
}

// EMBER Theme
const EMBER: Palette = Palette {
    background:       Color::from_rgb(0.11, 0.08, 0.06),
    surface:          Color::from_rgb(0.15, 0.10, 0.06),
    surface_hover:    Color::from_rgb(0.24, 0.17, 0.10),
    surface_active:   Color::from_rgb(0.29, 0.19, 0.13),
    border:           Color::from_rgb(0.24, 0.17, 0.10),
    text_primary:     Color::from_rgb(1.00, 0.84, 0.67),
    text_secondary:   Color::from_rgb(0.79, 0.66, 0.54),
    text_muted:       Color::from_rgb(0.49, 0.33, 0.23),
    accent:           Color::from_rgb(0.98, 0.45, 0.09),
    accent_hover:     Color::from_rgb(0.99, 0.73, 0.46),
    accent_secondary: Color::from_rgb(0.92, 0.35, 0.05),
    accent_tertiary:  Color::from_rgb(0.76, 0.26, 0.05),
    success:          Color::from_rgb(0.13, 0.77, 0.37),
    warning:          Color::from_rgb(0.92, 0.35, 0.05),
    error:            Color::from_rgb(0.60, 0.18, 0.07),
};

// MONOCHROME Theme
const MONOCHROME: Palette = Palette {
    background:       Color::from_rgb(0.04, 0.04, 0.04),
    surface:          Color::from_rgb(0.07, 0.07, 0.07),
    surface_hover:    Color::from_rgb(0.10, 0.10, 0.10),
    surface_active:   Color::from_rgb(0.13, 0.13, 0.13),
    border:           Color::from_rgb(0.20, 0.20, 0.20),
    text_primary:     Color::from_rgb(1.00, 1.00, 1.00),
    text_secondary:   Color::from_rgb(0.60, 0.60, 0.60),
    text_muted:       Color::from_rgb(0.40, 0.40, 0.40),
    accent:           Color::from_rgb(1.00, 1.00, 1.00),
    accent_hover:     Color::from_rgb(0.80, 0.80, 0.80),
    accent_secondary: Color::from_rgb(0.70, 0.70, 0.70),
    accent_tertiary:  Color::from_rgb(0.53, 0.53, 0.53),
    success:          Color::from_rgb(0.60, 0.60, 0.60),
    warning:          Color::from_rgb(0.70, 0.70, 0.70),
    error:            Color::from_rgb(0.53, 0.53, 0.53),
};

// DEEP OCEAN Theme
const DEEP_OCEAN: Palette = Palette {
    background:       Color::from_rgb(0.04, 0.09, 0.16),
    surface:          Color::from_rgb(0.05, 0.12, 0.24),
    surface_hover:    Color::from_rgb(0.07, 0.17, 0.30),
    surface_active:   Color::from_rgb(0.12, 0.29, 0.46),
    border:           Color::from_rgb(0.12, 0.29, 0.46),
    text_primary:     Color::from_rgb(0.88, 0.95, 0.99),
    text_secondary:   Color::from_rgb(0.58, 0.77, 0.99),
    text_muted:       Color::from_rgb(0.39, 0.46, 0.55),
    accent:           Color::from_rgb(0.23, 0.51, 0.97),
    accent_hover:     Color::from_rgb(0.38, 0.65, 0.98),
    accent_secondary: Color::from_rgb(0.15, 0.39, 0.92),
    accent_tertiary:  Color::from_rgb(0.11, 0.31, 0.85),
    success:          Color::from_rgb(0.58, 0.79, 0.98),
    warning:          Color::from_rgb(0.15, 0.31, 0.54),
    error:            Color::from_rgb(0.40, 0.60, 0.80),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum FishaTheme {
    #[default]
    Ember,
    DeepOcean,
    Monochrome,
}

impl FishaTheme {
    const ALL: [Self; 3] = [Self::Ember, Self::DeepOcean, Self::Monochrome];
    
    fn palette(&self) -> Palette {
        match self {
            Self::Ember => EMBER,
            Self::DeepOcean => DEEP_OCEAN,
            Self::Monochrome => MONOCHROME,
        }
    }
    
    fn name(&self) -> &'static str {
        match self {
            Self::Ember => "Ember",
            Self::DeepOcean => "Deep Ocean",
            Self::Monochrome => "Monochrome",
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

struct ContainerStyle(Palette, ContainerKind);

#[derive(Clone, Copy)]
enum ContainerKind {
    Background,
    Panel,
    Card,
}

impl container::StyleSheet for ContainerStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        let p = &self.0;
        match self.1 {
            ContainerKind::Background => container::Appearance {
                background: Some(Background::Color(p.background)),
                text_color: Some(p.text_primary),
                border: Border::default(),
                ..Default::default()
            },
            ContainerKind::Panel => container::Appearance {
                background: Some(Background::Color(p.surface)),
                text_color: Some(p.text_primary),
                border: Border { color: p.border, width: 1.0, radius: 12.0.into() },
                ..Default::default()
            },
            ContainerKind::Card => container::Appearance {
                background: Some(Background::Color(p.surface)),
                text_color: Some(p.text_primary),
                border: Border { color: p.border, width: 1.0, radius: 8.0.into() },
                ..Default::default()
            },
        }
    }
}

struct ButtonStyle(Palette, ButtonKind);

#[derive(Clone, Copy)]
enum ButtonKind { 
    Primary, 
    Secondary, 
    Nav(bool), 
    Danger, 
    ListItem(bool),
}

impl button::StyleSheet for ButtonStyle {
    type Style = Theme;
    fn active(&self, _: &Self::Style) -> button::Appearance {
        let p = &self.0;
        let (bg, txt, border_color, border_width) = match self.1 {
            ButtonKind::Primary => (p.accent, p.background, Color::TRANSPARENT, 0.0),
            ButtonKind::Secondary => (Color::TRANSPARENT, p.text_muted, p.border, 1.0),
            ButtonKind::Nav(sel) => {
                if sel { (p.accent, p.background, Color::TRANSPARENT, 0.0) }
                else { (Color::TRANSPARENT, p.text_muted, p.border, 1.0) }
            },
            ButtonKind::Danger => (Color::TRANSPARENT, p.error, p.error, 1.0),
            ButtonKind::ListItem(sel) => {
                if sel { (p.surface_hover, p.text_primary, Color::TRANSPARENT, 0.0) }
                else { (Color::TRANSPARENT, p.text_secondary, Color::TRANSPARENT, 0.0) }
            },
        };
        button::Appearance {
            background: Some(Background::Color(bg)),
            text_color: txt,
            border: Border { color: border_color, width: border_width, radius: 6.0.into() },
            ..Default::default()
        }
    }
    
    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        let p = &self.0;
        let (bg, txt, border_color, border_width) = match self.1 {
            ButtonKind::Primary => (p.accent_hover, p.background, Color::TRANSPARENT, 0.0),
            ButtonKind::Secondary => (p.surface_hover, p.text_primary, p.accent, 1.0),
            ButtonKind::Nav(sel) => {
                if sel { (p.accent_hover, p.background, Color::TRANSPARENT, 0.0) }
                else { (p.surface_hover, p.text_primary, p.border, 1.0) }
            },
            ButtonKind::Danger => (alpha(p.error, 0.2), p.error, p.error, 1.0),
            ButtonKind::ListItem(_) => (p.surface_hover, p.text_primary, Color::TRANSPARENT, 0.0),
        };
        button::Appearance {
            background: Some(Background::Color(bg)),
            text_color: txt,
            border: Border { color: border_color, width: border_width, radius: 6.0.into() },
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
            border: Border { color: self.0.border, width: 1.0, radius: 6.0.into() },
            icon_color: self.0.text_muted,
        }
    }
    fn focused(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(self.0.surface_hover),
            border: Border { color: self.0.accent, width: 2.0, radius: 6.0.into() },
            icon_color: self.0.text_muted,
        }
    }
    fn hovered(&self, s: &Self::Style) -> text_input::Appearance {
        let mut a = self.active(s);
        a.border.color = self.0.accent;
        a
    }
    fn disabled(&self, s: &Self::Style) -> text_input::Appearance { self.active(s) }
    fn placeholder_color(&self, _: &Self::Style) -> Color { self.0.text_muted }
    fn value_color(&self, _: &Self::Style) -> Color { self.0.text_primary }
    fn selection_color(&self, _: &Self::Style) -> Color { alpha(self.0.accent, 0.3) }
    fn disabled_color(&self, _: &Self::Style) -> Color { self.0.text_muted }
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
                scroller: scrollable::Scroller { 
                    color: self.0.surface_hover, 
                    border: Border { radius: 4.0.into(), ..Default::default() } 
                },
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
            border: Border { color: self.0.border, width: 1.0, radius: 6.0.into() },
            text_color: self.0.text_primary,
            placeholder_color: self.0.text_muted,
            handle_color: self.0.text_muted,
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
            border: Border { color: self.0.border, width: 1.0, radius: 8.0.into() },
            text_color: self.0.text_primary,
            selected_background: Background::Color(alpha(self.0.accent, 0.3)),
            selected_text_color: self.0.accent,
        }
    }
}

struct RuleStyle(Palette);
impl iced::widget::rule::StyleSheet for RuleStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::rule::Appearance {
        iced::widget::rule::Appearance {
            color: self.0.border,
            width: 1,
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Full,
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
            Message::FileSaved(Ok(p)) => { 
                self.current_file = Some(p.clone()); 
                self.status_message = format!("Saved: {}", p); 
            }
            Message::FileSaved(Err(e)) => if e != "Cancelled" { self.status_message = e; },
            Message::SetViewMode(m) => self.view_mode = m,
            Message::SetTheme(t) => self.theme = t,
            Message::SelectCategory(c) => { 
                self.selected_category = Some(c); 
                self.selected_subcategory = None; 
            }
            Message::SelectSubcategory(s) => self.selected_subcategory = Some(s),
            Message::SearchQueryChanged(q) => { 
                self.search_query = q; 
                self.search_results = self.microfiche.search(&self.search_query); 
            }
            Message::FormCategoryChanged(v) => self.form_category = v,
            Message::FormSubcategoryChanged(v) => self.form_subcategory = v,
            Message::FormConceptChanged(v) => self.form_concept = v,
            Message::FormNoteChanged(v) => self.form_note = v,
            Message::SubmitEntry => {
                if self.form_category.is_empty() || self.form_subcategory.is_empty() 
                    || self.form_concept.is_empty() || self.form_note.trim().is_empty() 
                {
                    self.status_message = "All fields required".into();
                } else {
                    self.microfiche.add_entry(
                        &self.form_category, 
                        &self.form_subcategory, 
                        &self.form_concept, 
                        self.form_note.trim().into()
                    );
                    self.status_message = "Entry created".into();
                    self.form_category.clear(); 
                    self.form_subcategory.clear(); 
                    self.form_concept.clear(); 
                    self.form_note.clear();
                }
            }
            Message::ClearForm => { 
                self.form_category.clear(); 
                self.form_subcategory.clear(); 
                self.form_concept.clear(); 
                self.form_note.clear(); 
            }
            Message::DeleteNote { category, subcategory, concept, note } => {
                if self.microfiche.delete_note(&category, &subcategory, &concept, &note) {
                    self.status_message = "Deleted".into();
                    if self.view_mode == ViewMode::Search { 
                        self.search_results = self.microfiche.search(&self.search_query); 
                    }
                    if !self.microfiche.categories.contains_key(&category) { 
                        self.selected_category = None; 
                        self.selected_subcategory = None; 
                    }
                }
            }
            Message::EditNote { category, subcategory, concept, note } => {
                if self.microfiche.delete_note(&category, &subcategory, &concept, &note) {
                    self.form_category = category; 
                    self.form_subcategory = subcategory; 
                    self.form_concept = concept; 
                    self.form_note = note;
                    self.view_mode = ViewMode::Create;
                    self.status_message = "Edit and submit".into();
                }
            }
            Message::UseAsTemplate { category, subcategory, concept } => {
                self.form_category = category; 
                self.form_subcategory = subcategory; 
                self.form_concept = concept; 
                self.form_note.clear();
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
            container(content).width(Length::Fill).height(Length::Fill).padding(16),
            self.view_status_bar()
        ].into();
        
        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Background))))
            .into()
    }
}

impl Fisha {
    fn view_top_bar(&self) -> Element<Message> {
        let p = self.theme.palette();
        
        let nav_btn = |label: &str, mode: ViewMode, is_active: bool| -> Element<Message> {
            button(text(label).size(14))
                .on_press(Message::SetViewMode(mode))
                .padding([8, 16])
                .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Nav(is_active)))))
                .into()
        };
        
        container(
            row![
                text("Fisha").size(20).style(p.text_primary),
                horizontal_space().width(24),
                
                row![
                    nav_btn("Browse", ViewMode::Browse, self.view_mode == ViewMode::Browse),
                    nav_btn("Search", ViewMode::Search, self.view_mode == ViewMode::Search),
                    nav_btn("Create", ViewMode::Create, self.view_mode == ViewMode::Create),
                    nav_btn("Stats", ViewMode::Stats, self.view_mode == ViewMode::Stats),
                ].spacing(4),
                
                horizontal_space().width(24),
                
                row![
                    button(text("Open").size(12))
                        .on_press(Message::OpenFile)
                        .padding([6, 12])
                        .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
                    button(text("Save").size(12))
                        .on_press(Message::SaveFile)
                        .padding([6, 12])
                        .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
                ].spacing(4),
                
                horizontal_space(),
                
                pick_list(&FishaTheme::ALL[..], Some(self.theme), Message::SetTheme)
                    .text_size(14)
                    .padding([6, 12])
                    .style(iced::theme::PickList::Custom(
                        std::rc::Rc::new(PickStyle(p)), 
                        std::rc::Rc::new(MenuStyle(p))
                    )),
            ]
            .spacing(8)
            .padding(12)
            .align_items(iced::Alignment::Center)
        )
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Panel))))
        .into()
    }

    fn view_status_bar(&self) -> Element<Message> {
        let p = self.theme.palette();
        let stats = self.microfiche.stats();
        let file_name = self.current_file.as_deref().unwrap_or("No file");
        
        container(
            row![
                text(file_name).size(12).style(p.text_muted),
                horizontal_space().width(24),
                text(format!(
                    "{} categories, {} subcategories, {} concepts, {} notes",
                    stats.categories, stats.subcategories, stats.concepts, stats.notes
                )).size(12).style(p.text_muted),
                horizontal_space(),
                text(self.theme.name()).size(12).style(p.text_muted),
            ]
            .spacing(8)
            .padding([8, 16])
            .align_items(iced::Alignment::Center)
        )
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Background))))
        .into()
    }

    fn view_browse(&self) -> Element<Message> {
        let p = self.theme.palette();
        
        // Categories panel
        let cats = self.microfiche.sorted_category_names();
        let cat_list: Element<Message> = if cats.is_empty() {
            text("No categories").size(14).style(p.text_muted).into()
        } else {
            Column::with_children(cats.iter().map(|name| {
                let is_selected = self.selected_category.as_ref() == Some(name);
                button(text(name).size(14))
                    .on_press(Message::SelectCategory(name.clone()))
                    .width(Length::Fill)
                    .padding([8, 12])
                    .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::ListItem(is_selected)))))
                    .into()
            }).collect::<Vec<_>>())
            .spacing(4)
            .into()
        };
        
        let cat_panel = container(
            column![
                text("Categories").size(14).style(p.accent),
                horizontal_rule(1).style(iced::theme::Rule::Custom(Box::new(RuleStyle(p)))),
                scrollable(container(cat_list).padding([8, 12, 8, 8]))
                    .height(Length::Fill)
                    .style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p)))),
            ]
            .spacing(12)
            .padding(16)
        )
        .width(200)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Panel))));
        
        // Subcategories panel
        let sub_panel: Element<Message> = if let Some(cat_name) = &self.selected_category {
            if let Some(cat) = self.microfiche.categories.get(cat_name) {
                let sub_list: Vec<Element<Message>> = cat.subcategories.iter().map(|sub| {
                    let is_selected = self.selected_subcategory.as_ref() == Some(&sub.name);
                    button(text(&sub.name).size(14))
                        .on_press(Message::SelectSubcategory(sub.name.clone()))
                        .width(Length::Fill)
                        .padding([8, 12])
                        .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::ListItem(is_selected)))))
                        .into()
                }).collect();
                
                container(
                    column![
                        text("Subcategories").size(14).style(p.accent),
                        horizontal_rule(1).style(iced::theme::Rule::Custom(Box::new(RuleStyle(p)))),
                        scrollable(Column::with_children(sub_list).spacing(4).padding([8, 12, 8, 8]))
                            .height(Length::Fill)
                            .style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p)))),
                    ]
                    .spacing(12)
                    .padding(16)
                )
                .width(200)
                .height(Length::Fill)
                .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Panel))))
                .into()
            } else {
                horizontal_space().width(200).into()
            }
        } else {
            horizontal_space().width(200).into()
        };
        
        // Main content
        let main: Element<Message> = match (&self.selected_category, &self.selected_subcategory) {
            (Some(cn), Some(sn)) => {
                if let Some(cat) = self.microfiche.categories.get(cn) {
                    if let Some(sub) = cat.subcategories.iter().find(|s| &s.name == sn) {
                        let cards: Vec<Element<Message>> = sub.concepts.iter()
                            .map(|concept| self.view_concept_card(cn, sn, concept))
                            .collect();
                        
                        column![
                            text(format!("{} / {}", cn, sn)).size(16).style(p.accent),
                            horizontal_rule(1).style(iced::theme::Rule::Custom(Box::new(RuleStyle(p)))),
                            scrollable(Column::with_children(cards).spacing(12).padding([8, 12, 8, 8]))
                                .height(Length::Fill)
                                .style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p)))),
                        ]
                        .spacing(8)
                        .into()
                    } else {
                        self.empty_state("Not found")
                    }
                } else {
                    self.empty_state("Not found")
                }
            }
            (Some(_), None) => self.empty_state("Select a subcategory"),
            _ => {
                if self.microfiche.categories.is_empty() {
                    column![
                        vertical_space(),
                        text("No data loaded").size(16).style(p.text_muted),
                        vertical_space().height(16),
                        button(text("Open File").size(14))
                            .on_press(Message::OpenFile)
                            .padding([12, 24])
                            .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Primary)))),
                        vertical_space(),
                    ]
                    .align_items(iced::Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else {
                    self.empty_state("Select a category")
                }
            }
        };
        
        row![
            cat_panel,
            sub_panel,
            container(main)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20)
                .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Panel)))),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    }

    fn view_concept_card(&self, cat: &str, sub: &str, concept: &Concept) -> Element<Message> {
        let p = self.theme.palette();
        
        let notes: Vec<Element<Message>> = concept.notes.iter().map(|note| {
            let (c1, s1, co1) = (cat.to_string(), sub.to_string(), concept.name.clone());
            let (c2, s2, co2, n2) = (c1.clone(), s1.clone(), co1.clone(), note.clone());
            let (c3, s3, co3, n3) = (c1.clone(), s1.clone(), co1.clone(), note.clone());
            
            container(
                column![
                    text(note).size(14).style(p.text_secondary).width(Length::Fill),
                    vertical_space().height(8),
                    row![
                        button(text("Template").size(12))
                            .on_press(Message::UseAsTemplate { category: c1, subcategory: s1, concept: co1 })
                            .padding([4, 8])
                            .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
                        button(text("Edit").size(12))
                            .on_press(Message::EditNote { category: c2, subcategory: s2, concept: co2, note: n2 })
                            .padding([4, 8])
                            .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
                        button(text("Delete").size(12))
                            .on_press(Message::DeleteNote { category: c3, subcategory: s3, concept: co3, note: n3 })
                            .padding([4, 8])
                            .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Danger)))),
                    ].spacing(8),
                ]
                .padding(12)
                .width(Length::Fill)
            )
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Card))))
            .into()
        }).collect();
        
        container(
            column![
                text(&concept.name).size(16).style(p.text_primary),
                text(format!("{} notes", concept.notes.len())).size(12).style(p.text_muted),
                horizontal_rule(1).style(iced::theme::Rule::Custom(Box::new(RuleStyle(p)))),
                Column::with_children(notes).spacing(8).width(Length::Fill),
            ]
            .spacing(8)
            .padding(16)
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Card))))
        .into()
    }

    fn view_search(&self) -> Element<Message> {
        let p = self.theme.palette();
        
        let results: Vec<Element<Message>> = self.search_results.iter().map(|r| {
            let (c1, s1, co1) = (r.category.clone(), r.subcategory.clone(), r.concept.clone());
            let (c2, s2, co2, n2) = (c1.clone(), s1.clone(), co1.clone(), r.note.clone());
            let (c3, s3, co3, n3) = (c1.clone(), s1.clone(), co1.clone(), r.note.clone());
            
            container(
                column![
                    text(format!("{} / {} / {}", r.category, r.subcategory, r.concept))
                        .size(12)
                        .style(p.accent),
                    text(&r.note).size(14).style(p.text_secondary).width(Length::Fill),
                    vertical_space().height(8),
                    row![
                        button(text("Template").size(12))
                            .on_press(Message::UseAsTemplate { category: c1, subcategory: s1, concept: co1 })
                            .padding([4, 8])
                            .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
                        button(text("Edit").size(12))
                            .on_press(Message::EditNote { category: c2, subcategory: s2, concept: co2, note: n2 })
                            .padding([4, 8])
                            .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
                        button(text("Delete").size(12))
                            .on_press(Message::DeleteNote { category: c3, subcategory: s3, concept: co3, note: n3 })
                            .padding([4, 8])
                            .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Danger)))),
                    ].spacing(8),
                ]
                .spacing(4)
                .padding(16)
                .width(Length::Fill)
            )
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Card))))
            .into()
        }).collect();
        
        let results_text = format!("Found {} results", self.search_results.len());
        
        container(
            column![
                text("Search").size(24).style(p.text_primary),
                vertical_space().height(16),
                text_input("Search...", &self.search_query)
                    .on_input(Message::SearchQueryChanged)
                    .padding(12)
                    .size(14)
                    .style(iced::theme::TextInput::Custom(Box::new(InputStyle(p)))),
                vertical_space().height(16),
                text(results_text).size(12).style(p.text_muted),
                horizontal_rule(1).style(iced::theme::Rule::Custom(Box::new(RuleStyle(p)))),
                scrollable(Column::with_children(results).spacing(12).padding([8, 12, 8, 8]).width(Length::Fill))
                    .height(Length::Fill)
                    .style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p)))),
            ]
            .spacing(8)
            .padding(24)
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Panel))))
        .into()
    }

    fn view_create(&self) -> Element<Message> {
        let p = self.theme.palette();
        
        let field = |label: &str, placeholder: &str, val: &str, msg: fn(String) -> Message| -> Element<Message> {
            column![
                text(label).size(12).style(p.text_muted),
                text_input(placeholder, val)
                    .on_input(msg)
                    .padding(12)
                    .size(14)
                    .style(iced::theme::TextInput::Custom(Box::new(InputStyle(p)))),
            ]
            .spacing(6)
            .into()
        };
        
        let status_text = if self.status_message.is_empty() { 
            " ".to_string() 
        } else { 
            self.status_message.clone() 
        };
        
        container(
            scrollable(
                container(
                    column![
                        text("Create New Entry").size(24).style(p.text_primary),
                        text("Add a new note to your knowledge base").size(14).style(p.text_muted),
                        vertical_space().height(24),
                        field("Category", "e.g. Programming", &self.form_category, Message::FormCategoryChanged),
                        field("Subcategory", "e.g. Rust", &self.form_subcategory, Message::FormSubcategoryChanged),
                        field("Concept", "e.g. Ownership", &self.form_concept, Message::FormConceptChanged),
                        field("Note", "Your note...", &self.form_note, Message::FormNoteChanged),
                        vertical_space().height(24),
                        row![
                            button(text("Create Entry").size(14))
                                .on_press(Message::SubmitEntry)
                                .padding([12, 24])
                                .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Primary)))),
                            button(text("Clear").size(14))
                                .on_press(Message::ClearForm)
                                .padding([12, 24])
                                .style(iced::theme::Button::Custom(Box::new(ButtonStyle(p, ButtonKind::Secondary)))),
                        ].spacing(12),
                        vertical_space().height(16),
                        text(status_text).size(14).style(p.accent),
                    ]
                    .spacing(16)
                    .max_width(600)
                )
                .padding(32)
                .center_x()
            )
            .height(Length::Fill)
            .style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p))))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Panel))))
        .into()
    }

    fn view_stats(&self) -> Element<Message> {
        let p = self.theme.palette();
        let s = self.microfiche.stats();
        
        let stat_row = |label: &str, value: usize| -> Element<Message> {
            row![
                text(label).size(14).style(p.text_secondary).width(Length::FillPortion(1)),
                text(value.to_string()).size(16).style(p.accent).width(Length::FillPortion(1)),
            ]
            .padding(8)
            .into()
        };
        
        let mut cat_stats: Vec<_> = self.microfiche.categories.iter().map(|(name, cat)| {
            let notes: usize = cat.subcategories.iter()
                .flat_map(|s| &s.concepts)
                .map(|c| c.notes.len())
                .sum();
            (name.clone(), cat.subcategories.len(), notes)
        }).collect();
        cat_stats.sort_by(|a, b| b.2.cmp(&a.2));
        
        let cat_rows: Vec<Element<Message>> = cat_stats.iter().take(10).map(|(name, subs, notes)| {
            row![
                text(name).size(14).style(p.text_primary).width(Length::FillPortion(2)),
                text(format!("{} subs", subs)).size(12).style(p.text_muted).width(Length::FillPortion(1)),
                text(format!("{} notes", notes)).size(12).style(p.accent).width(Length::FillPortion(1)),
            ]
            .spacing(12)
            .padding(8)
            .into()
        }).collect();
        
        container(
            scrollable(
                container(
                    column![
                        text("Statistics").size(24).style(p.text_primary),
                        text("Overview of your knowledge base").size(14).style(p.text_muted),
                        vertical_space().height(24),
                        container(
                            column![
                                text("Overview").size(16).style(p.accent),
                                horizontal_rule(1).style(iced::theme::Rule::Custom(Box::new(RuleStyle(p)))),
                                stat_row("Categories", s.categories),
                                stat_row("Subcategories", s.subcategories),
                                stat_row("Concepts", s.concepts),
                                stat_row("Total Notes", s.notes),
                            ]
                            .spacing(4)
                            .padding(16)
                        )
                        .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Card)))),
                        vertical_space().height(24),
                        container(
                            column![
                                text("Top Categories").size(16).style(p.accent),
                                horizontal_rule(1).style(iced::theme::Rule::Custom(Box::new(RuleStyle(p)))),
                                Column::with_children(cat_rows).spacing(4),
                            ]
                            .spacing(8)
                            .padding(16)
                        )
                        .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Card)))),
                    ]
                    .spacing(8)
                    .max_width(800)
                )
                .padding(32)
                .center_x()
            )
            .height(Length::Fill)
            .style(iced::theme::Scrollable::Custom(Box::new(ScrollStyle(p))))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(ContainerStyle(p, ContainerKind::Panel))))
        .into()
    }

    fn empty_state(&self, msg: &str) -> Element<Message> {
        let p = self.theme.palette();
        container(text(msg).size(14).style(p.text_muted))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}