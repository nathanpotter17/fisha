# Fisha

A lightweight desktop GUI application for organizing knowledge in a hierarchical structure with CSV import/export capabilities.

## Overview

Fisha provides a clean, intuitive interface for managing knowledge organized in a 4-level hierarchy:
```
Category → Subcategory → Concept → Note
```

## Features

- **Browse View**: Navigate your knowledge hierarchy with side panels for categories and subcategories
- **Search**: Full-text search across all fields with inline results
- **Create**: Add new entries through a guided form interface
- **Statistics**: Visual dashboard with category distribution bars and hierarchy counts
- **CSV Import/Export**: Load and save your knowledge base with File menu
- **Theme Support**: Three professionally designed dark themes (Monokai, Tomorrow Blue Hour, Dark+)
- **Entry Management**: Edit, delete, or use as template for quick entry creation
- **Auto-save**: Loads `microfiche.csv` from current directory on startup

## CSV Format

The application expects a CSV file with the following 4 columns:

| Category | Subcategory | Concept | Note |
|----------|-------------|---------|------|
| Mathematics | Algebra | Quadratic Formula | x = (-b ± √(b²-4ac))/2a |

**Important**: 
- Headers must be: `Category`, `Subcategory`, `Concept`, `Note`
- Multiple notes can exist for the same concept
- Avoid excessive commas in note content as they're used for CSV field separation

## Usage
```bash
# Run the application
cargo run

# Or build and run release version
cargo build --release
./target/release/fisha
```

On startup, the application automatically loads `microfiche.csv` from the current directory if it exists.

## Controls

### Browse Tab
- Click categories in left panel to view subcategories
- Click subcategories in middle panel to view concepts and notes
- **Template**: Load category/subcategory/concept to create a new note
- **Edit**: Load an entry into the Create form for modification
- **Delete**: Remove the note from the database

### Search Tab
- Enter search terms to find matches across all fields
- Results show full hierarchy path: Category > Subcategory > Concept
- Edit, Delete, and Template buttons available for each result

### Create Tab
- Fill in Category, Subcategory, Concept, and Note fields
- All fields are required
- Click "Create" to add the entry
- Form clears automatically after successful creation

### Stats Tab
- View total counts for categories, subcategories, concepts, and notes
- Term co-occurence and pair frequency shows note distribution across categories

### File Menu
- **Open**: Import a CSV file
- **Save**: Save to current file (or prompt if no file loaded)
- **Save As**: Export to a new CSV file

### Theme Selector
- Click "Theme" button in top bar
- Choose from Monokai, Tomorrow (Blue Hour), or Dark+
- Theme applies immediately

## Building
```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release
```

## System Requirements

- Rust 1.70+
- Dependencies: `eframe`, `egui`, `csv`, `serde`, `rfd`
- Cross-platform: Windows, macOS, and Linux

## Data Structure

The application loads CSV data into an in-memory hierarchical structure:
- Fast navigation and searching
- Changes are maintained in memory until saved
- Export back to CSV preserves all data

## Tips

- **Manual Saves**: Remember to save periodically via File → Save
- **Edit Workflow**: Click "Edit" to modify an entry (deletes original, loads into Create form)
- **Template Workflow**: Click "Template" to quickly create similar entries with same category/subcategory/concept
- **Search Performance**: Search is case-insensitive and searches across all text fields

## Example CSV File
```csv
Category,Subcategory,Concept,Note
19th Century Computing,Apollo Project,VCF Midwest 2025,The SAGE Air Defense System - https://www.youtube.com/watch?v=Q8iOfaMd5oY
19th Century Computing,Apollo Computer,1969 AGC,Light Years Ahead 1969 Apollo Guidance Computer - https://www.youtube.com/watch?v=B1J2RMorJXM
```