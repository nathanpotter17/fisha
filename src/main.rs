use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Write;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

const IN_FILE: &str = "db/in.csv";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Notes {
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyDetails {
    name: String,
    notes: Vec<Notes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Concept {
    name: String,
    details: Vec<KeyDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Subcategory {
    name: String,
    concepts: Vec<Concept>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    
    fn from_csv(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut fiche = Microfiche::new();
        let mut rdr = Reader::from_path(path)?;
        
        for result in rdr.deserialize() {
            let row: FicheRow = result?;
            fiche.add_row(row);
        }
        
        Ok(fiche)
    }
    
    fn add_row(&mut self, row: FicheRow) {
        let category = self.categories.entry(row.category.clone())
            .or_insert_with(|| Category {
                name: row.category,
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
    
    fn validate_csv(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        let mut warnings = Vec::new();
        let mut seen_paths = HashSet::new();
        let mut line_num = 1;
        
        for result in rdr.deserialize() {
            line_num += 1;
            let row: FicheRow = result?;
            
            if row.category.is_empty() || row.subcategory.is_empty() || 
               row.concept.is_empty() || row.key_detail.is_empty() {
                warnings.push(format!("Line {}: Contains empty fields", line_num));
            }
            
            let path = format!("{}.{}.{}.{}: {}", 
                row.category, row.subcategory, row.concept, row.key_detail, row.note);
            if seen_paths.contains(&path) {
                warnings.push(format!("Line {}: Duplicate entry found", line_num));
            }
            seen_paths.insert(path);
            
            if row.note.contains('\n') || row.note.contains('\r') {
                warnings.push(format!("Line {}: Note contains newline characters", line_num));
            }
        }
        
        Ok(warnings)
    }
    
    fn stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        let mut total_subcats = 0;
        let mut total_concepts = 0;
        let mut total_details = 0;
        let mut total_notes = 0;
        let mut max_depth = 0;
        
        stats.insert("categories".to_string(), self.categories.len());
        
        for (_, category) in &self.categories {
            total_subcats += category.subcategories.len();
            for subcat in &category.subcategories {
                total_concepts += subcat.concepts.len();
                for concept in &subcat.concepts {
                    total_details += concept.details.len();
                    for detail in &concept.details {
                        total_notes += detail.notes.len();
                        max_depth = max_depth.max(detail.notes.len());
                    }
                }
            }
        }
        
        stats.insert("subcategories".to_string(), total_subcats);
        stats.insert("concepts".to_string(), total_concepts);
        stats.insert("key_details".to_string(), total_details);
        stats.insert("total_notes".to_string(), total_notes);
        stats.insert("max_notes_per_detail".to_string(), max_depth);
        
        stats
    }
    
    fn preview(&self, max_items: usize) -> String {
        let mut output = String::new();
        let mut cat_count = 0;
        
        for (cat_name, category) in &self.categories {
            if cat_count >= max_items { 
                output.push_str(&format!("\n... and {} more categories", 
                    self.categories.len() - cat_count));
                break; 
            }
            
            output.push_str(&format!("\n[CAT] {}\n", cat_name));
            
            let mut subcat_count = 0;
            for subcat in &category.subcategories {
                if subcat_count >= max_items.min(3) { 
                    if category.subcategories.len() > subcat_count {
                        output.push_str(&format!("    ... and {} more subcategories\n", 
                            category.subcategories.len() - subcat_count));
                    }
                    break; 
                }
                
                output.push_str(&format!("    [SUB] {}\n", subcat.name));
                
                let mut concept_count = 0;
                for concept in &subcat.concepts {
                    if concept_count >= max_items.min(2) { 
                        if subcat.concepts.len() > concept_count {
                            output.push_str(&format!("        ... and {} more concepts\n", 
                                subcat.concepts.len() - concept_count));
                        }
                        break; 
                    }
                    
                    let total_notes: usize = concept.details.iter()
                        .map(|d| d.notes.len())
                        .sum();
                    
                    output.push_str(&format!("        [CON] {} ({} details, {} notes)\n", 
                        concept.name, concept.details.len(), total_notes));
                    
                    concept_count += 1;
                }
                subcat_count += 1;
            }
            cat_count += 1;
        }
        
        output
    }
    
    fn category_distribution(&self) -> Vec<(String, usize)> {
        let mut distribution: Vec<(String, usize)> = self.categories
            .iter()
            .map(|(name, cat)| {
                let note_count: usize = cat.subcategories.iter()
                    .flat_map(|s| &s.concepts)
                    .flat_map(|c| &c.details)
                    .map(|d| d.notes.len())
                    .sum();
                (name.clone(), note_count)
            })
            .collect();
        
        distribution.sort_by(|a, b| b.1.cmp(&a.1));
        distribution
    }
    
    fn sample_entries(&self, count: usize) -> Vec<String> {
        let mut entries = Vec::new();
        let mut collected = 0;
        
        for (cat_name, category) in &self.categories {
            if collected >= count { break; }
            for subcat in &category.subcategories {
                if collected >= count { break; }
                for concept in &subcat.concepts {
                    if collected >= count { break; }
                    for detail in &concept.details {
                        if collected >= count { break; }
                        for note in &detail.notes {
                            if collected >= count { break; }
                            entries.push(format!("{}.{}.{}.{}: {}", 
                                cat_name, 
                                subcat.name, 
                                concept.name, 
                                detail.name,
                                if note.content.len() > 80 {
                                    format!("{}...", &note.content[..80])
                                } else {
                                    note.content.clone()
                                }
                            ));
                            collected += 1;
                        }
                    }
                }
            }
        }
        
        entries
    }

    fn search(&self, query: &str) -> Vec<String> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        let parts: Vec<&str> = query_lower.split_whitespace().collect();
        
        if parts.is_empty() {
            return results;
        }
        
        // Check for exact category name match (case-insensitive)
        for (cat_name, category) in &self.categories {
            let cat_lower = cat_name.to_lowercase();
            
            // Exact category match - return everything in that category
            if parts.len() == 1 && cat_lower == parts[0] {
                for subcat in &category.subcategories {
                    for concept in &subcat.concepts {
                        for detail in &concept.details {
                            for note in &detail.notes {
                                results.push(format!("{}.{}.{}.{}: {}", 
                                    cat_name, subcat.name, concept.name, detail.name,
                                    note.content));
                            }
                        }
                    }
                }
                return results;
            }
        }
        
        // Check for exact subcategory name match (case-insensitive)
        for (cat_name, category) in &self.categories {
            for subcat in &category.subcategories {
                let subcat_lower = subcat.name.to_lowercase();
                
                // Handle multi-word subcategory names
                let query_str = parts.join(" ");
                if subcat_lower == query_str {
                    for concept in &subcat.concepts {
                        for detail in &concept.details {
                            for note in &detail.notes {
                                results.push(format!("{}.{}.{}.{}: {}", 
                                    cat_name, subcat.name, concept.name, detail.name,
                                    note.content));
                            }
                        }
                    }
                    return results;
                }
            }
        }
        
        // If no exact matches, do content search with ALL terms
        for (cat_name, category) in &self.categories {
            for subcat in &category.subcategories {
                for concept in &subcat.concepts {
                    for detail in &concept.details {
                        for note in &detail.notes {
                            let full_path = format!("{} {} {} {} {}", 
                                cat_name.to_lowercase(), 
                                subcat.name.to_lowercase(), 
                                concept.name.to_lowercase(), 
                                detail.name.to_lowercase(),
                                note.content.to_lowercase());
                            
                            let mut all_match = true;
                            for part in &parts {
                                if !full_path.contains(part) {
                                    all_match = false;
                                    break;
                                }
                            }
                            
                            if all_match {
                                results.push(format!("{}.{}.{}.{}: {}", 
                                    cat_name, subcat.name, concept.name, detail.name,
                                    note.content));
                            }
                        }
                    }
                }
            }
        }
        
        results
    }
    
    fn advanced_search(&self, category_filter: Option<&str>, 
                       subcategory_filter: Option<&str>,
                       content_query: Option<&str>) -> Vec<String> {
        let mut results = Vec::new();
        
        for (cat_name, category) in &self.categories {
            // Category filter - partial match
            if let Some(cat_filter) = category_filter {
                if !cat_name.to_lowercase().contains(&cat_filter.to_lowercase()) {
                    continue;
                }
            }
            
            for subcat in &category.subcategories {
                // Subcategory filter - partial match  
                if let Some(sub_filter) = subcategory_filter {
                    if !subcat.name.to_lowercase().contains(&sub_filter.to_lowercase()) {
                        continue;
                    }
                }
                
                for concept in &subcat.concepts {
                    for detail in &concept.details {
                        for note in &detail.notes {
                            // Content filter
                            if let Some(query) = content_query {
                                let note_lower = note.content.to_lowercase();
                                let query_lower = query.to_lowercase();
                                let query_parts: Vec<&str> = query_lower.split_whitespace().collect();
                                
                                let mut all_match = true;
                                for part in query_parts {
                                    if !note_lower.contains(part) {
                                        all_match = false;
                                        break;
                                    }
                                }
                                if !all_match {
                                    continue;
                                }
                            }
                            
                            results.push(format!("{}.{}.{}.{}: {}", 
                                cat_name, subcat.name, concept.name, detail.name,
                                note.content));
                        }
                    }
                }
            }
        }
        
        results
    }
    
    fn list_structure(&self) -> String {
        let mut output = String::new();
        output.push_str("Knowledge Base Structure:\n");
        
        for (cat_name, category) in &self.categories {
            output.push_str(&format!("\n[{}]\n", cat_name));
            for subcat in &category.subcategories {
                let concept_count = subcat.concepts.len();
                let note_count: usize = subcat.concepts.iter()
                    .flat_map(|c| &c.details)
                    .map(|d| d.notes.len())
                    .sum();
                output.push_str(&format!("  - {} ({} concepts, {} notes)\n", 
                    subcat.name, concept_count, note_count));
            }
        }
        output
    }

    fn search_with_index(&self, query: &str) -> Vec<(usize, String)> {
        let results = self.search(query);
        results.into_iter().enumerate().map(|(i, r)| (i + 1, r)).collect()
    }
    
    fn display_entry(&self, path: &str) -> Option<String> {
        // Parse the path format: "Category.Subcategory.Concept.KeyDetail: Note"
        let parts: Vec<&str> = path.split(": ").collect();
        if parts.len() != 2 {
            return None;
        }
        
        let path_parts: Vec<&str> = parts[0].split('.').collect();
        if path_parts.len() != 4 {
            return None;
        }
        
        let mut output = String::new();
        output.push_str(&"=".repeat(80));
        output.push_str("\n");
        output.push_str(&format!("Category:    {}\n", path_parts[0]));
        output.push_str(&format!("Subcategory: {}\n", path_parts[1]));
        output.push_str(&format!("Concept:     {}\n", path_parts[2]));
        output.push_str(&format!("Key Detail:  {}\n", path_parts[3]));
        output.push_str(&"-".repeat(80));
        output.push_str("\n\n");
        
        let note = parts[1];
        let wrapped = self.word_wrap(note, 76);
        output.push_str(&wrapped);
        output.push_str("\n\n");
        output.push_str(&"=".repeat(80));
        
        Some(output)
    }
    
    fn word_wrap(&self, text: &str, max_width: usize) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut current_line = String::new();
        
        for word in words {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + word.len() + 1 <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        lines.join("\n")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fiche = Microfiche::from_csv(IN_FILE)?;
    let warnings = Microfiche::validate_csv(IN_FILE)?;
    if !warnings.is_empty() {
        println!("Validation warnings:");
        for warning in warnings {
            println!("  {}", warning);
        }
        println!();
    }
        
    println!("Statistics:");
    println!("{}", "=".repeat(50));
    let stats = fiche.stats();
    println!("  Categories:           {}", stats.get("categories").unwrap_or(&0));
    println!("  Subcategories:        {}", stats.get("subcategories").unwrap_or(&0));
    println!("  Concepts:             {}", stats.get("concepts").unwrap_or(&0));
    println!("  Key Details:          {}", stats.get("key_details").unwrap_or(&0));
    println!("  Total Notes:          {}", stats.get("total_notes").unwrap_or(&0));
    println!("  Max Notes per Detail: {}", stats.get("max_notes_per_detail").unwrap_or(&0));
    
    println!("\nTop Categories by Note Count:");
    println!("{}", "=".repeat(50));
    let distribution = fiche.category_distribution();
    for (i, (name, count)) in distribution.iter().take(5).enumerate() {
        let bar_length = (*count as f32 / distribution[0].1.max(1) as f32 * 30.0) as usize;
        let bar = "#".repeat(bar_length);
        println!("  {}. {:<20} {} {}", i+1, name, bar, count);
    }
    
    println!("\nStructure Preview:");
    println!("{}", "=".repeat(50));
    println!("{}", fiche.preview(3));
    
    println!("Sample Entries:");
    println!("{}", "=".repeat(50));
    let samples = fiche.sample_entries(5);
    for (i, sample) in samples.iter().enumerate() {
        println!("  {}. {}", i+1, sample);
    }

    println!("\nBegin Querying Knowledgebase:");
    println!("Commands: <query>, #<number>, cat:<category>, sub:<subcategory>, list, help, quit");

    let mut last_results: Vec<String> = Vec::new();

    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let query = input.trim();
        if query == "quit" { break; }
        
        if query == "help" {
            println!("Search commands:");
            println!("  mathematics          - Show all entries in Mathematics category");
            println!("  computer science     - Show all entries in Computer Science subcategory");
            println!("  halting problem      - Find entries with both 'halting' AND 'problem'");
            println!("  cat:math             - Filter to categories containing 'math'");
            println!("  sub:computer         - Filter to subcategories containing 'computer'");
            println!("  #(n)                  - View result #(n) from last search in detail");
            println!("  list                 - Show all categories and subcategories");
            continue;
        }
        
        if query == "list" {
            println!("{}", fiche.list_structure());
            continue;
        }
        
        // Handle numbered selection
        if query.starts_with('#') {
            if let Ok(num) = query[1..].parse::<usize>() {
                if num > 0 && num <= last_results.len() {
                    let selected = &last_results[num - 1];
                    if let Some(display) = fiche.display_entry(selected) {
                        println!("\n{}", display);
                    } else {
                        println!("Error parsing entry");
                    }
                } else {
                    println!("Invalid selection. Please choose a number between 1 and {}", last_results.len());
                }
            } else {
                println!("Invalid number format");
            }
            continue;
        }
        
        // Parse and execute search
        let results = if query.starts_with("cat:") {
            let cat_query = &query[4..];
            fiche.advanced_search(Some(cat_query), None, None)
        } else if query.starts_with("sub:") {
            let sub_query = &query[4..];
            fiche.advanced_search(None, Some(sub_query), None)
        } else if query.contains(" ") && query.contains(":") {
            // Handle combined queries
            let parts: Vec<&str> = query.split_whitespace().collect();
            let mut cat_filter = None;
            let mut sub_filter = None;
            let mut content_parts = Vec::new();
            
            for part in parts {
                if part.starts_with("cat:") {
                    cat_filter = Some(&part[4..]);
                } else if part.starts_with("sub:") {
                    sub_filter = Some(&part[4..]);
                } else {
                    content_parts.push(part);
                }
            }
            
            let content_query = if content_parts.is_empty() {
                None
            } else {
                Some(content_parts.join(" "))
            };
            
            fiche.advanced_search(cat_filter, sub_filter, content_query.as_deref())
        } else {
            fiche.search(query)
        };
        
        // Store results for numbered access
        last_results = results.clone();
        
        if results.is_empty() {
            println!("  No results found");
        } else {
            println!("  Found {} results (showing first 10):", results.len());
            for (i, result) in results.iter().enumerate().take(10) {
                // Show with numbers for selection
                if result.len() > 140 {
                    println!("  [{}] {}...", i + 1, &result[..140]);
                } else {
                    println!("  [{}] {}", i + 1, result);
                }
            }
            if results.len() > 10 {
                println!("  ... and {} more results", results.len() - 10);
            }
            println!("\n  Type '#<number>' to view a specific result in detail");
        }
    }
    
    Ok(())
}