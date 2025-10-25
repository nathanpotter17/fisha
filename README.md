# Fisha

A lightweight desktop GUI application for organizing knowledge in a hierarchical structure with CSV import/export capabilities.

## Overview

Fisha provides a clean, intuitive interface for managing knowledge organized in a 5-level hierarchy:

```
Category → Subcategory → Concept → KeyDetail → Note
```

## Features

- **Browse View**: Navigate your knowledge hierarchy with expandable sections
- **Search**: Full-text search across all fields with result highlighting
- **Create**: Add new entries through a guided form interface
- **Statistics**: Visual dashboard with category distribution and counts
- **CSV Import/Export**: Load and save your knowledge base
- **Theme Support**: Light and dark themes
- **Entry Management**: Delete individual notes or entire branches

## CSV Format

| Category | Subcategory | Concept | KeyDetail | Note |
|----------|-------------|---------|-----------|------|
| Mathematics | Algebra | Quadratic Formula | Definition | x = (-b ± √(b²-4ac))/2a |

**Note**: The application expects headers: `Category`, `Subcategory`, `Concept`, `KeyDetail`, `Note`

## Usage

```bash
cargo run
```

On startup, the application looks for `db/database.csv` and loads it automatically. You can load your
own file by using File -> Open.

## Controls

- **Browse Tab**: Click categories/subcategories to expand and view entries
- **Search Tab**: Enter search terms to filter across all fields
- **Create Tab**: Fill in all fields and click "Create" to add new entries
- **Stats Tab**: View hierarchy statistics and category distribution chart
- **File Menu**: Import/Export CSV files
- **Theme Toggle**: Switch between light and dark modes

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
- Works on Windows, macOS, and Linux

## Data Structure

The application loads CSV data into an in-memory hierarchical structure for fast navigation and search. Changes persist when exported back to CSV format.

## Usage Notes

Note: When creating many entries, or using the app for a long time, be sure to manually save periodically via File -> Save.

Note: When creating an entry, be sure to limit the usage of commas, as they are used for field separation.

## Example DB File

```
Category,Subcategory,Concept,KeyDetail,Note
19th Century Computing,Apollo Project,VCF Midwest 2025,SAGE Air Defense,The SAGE Air Defense System - https://www.youtube.com/watch?v=Q8iOfaMd5oY
19th Century Computing,Apollo Computer,1969 AGC,Light Years Ahead,Light Years Ahead 1969 Apollo Guidance Computer - https://www.youtube.com/watch?v=B1J2RMorJXM

```