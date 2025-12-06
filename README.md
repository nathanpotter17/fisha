# Fisha

A lightweight desktop GUI application for organizing knowledge in a hierarchical structure with CSV import/export capabilities.

## Overview

Fisha provides a clean, intuitive interface for managing knowledge organized in a 4-level hierarchy:
```
Category → Subcategory → Concept → Note
```

## Usage
```bash
cargo run --release
```

or grab the latest release from [Releases]()

Loads `microfiche.csv` from current directory on startup.

## CSV Format
```csv
Category,Subcategory,Concept,Note
Mathematics,Algebra,Quadratic Formula,x = (-b ± √(b²-4ac))/2a
```

Headers required. Multiple notes per concept allowed.

## Views

- **Browse**: Navigate hierarchy via side panels
- **Search**: Multi-term AND search (all terms must match)
- **Create**: Add entries (all fields required)
- **Stats**: Counts and top categories

## Actions

| Button | Effect |
|--------|--------|
| Template | Pre-fill form with category/subcategory/concept |
| Edit | Delete entry, load into form |
| Delete | Remove note |

## Themes

Monokai, Tomorrow Blue, Dark+ — selectable via dropdown.

## Building
```bash
cargo build --release
```

Requires Rust 1.70+. Dependencies: `iced`, `csv`, `serde`, `rfd`, `tokio`.