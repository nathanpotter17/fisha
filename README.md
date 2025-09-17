# Fisha

A hierarchical knowledge base system for organizing and querying structured information via CSV.

## Structure

Data is organized in a 5-level hierarchy:

```
Category → Subcategory → Concept → KeyDetail → Note
```

## CSV Format

Input file: `db/in.csv`

| Category    | Subcategory | Concept           | KeyDetail  | Note                    |
| ----------- | ----------- | ----------------- | ---------- | ----------------------- |
| Mathematics | Algebra     | Quadratic Formula | Definition | x = (-b ± √(b²-4ac))/2a |

## Usage

```bash
cargo run
```

## Features

- **Import/Export**: Load from and save to CSV
- **Validation**: Check for empty fields, duplicates, and formatting issues
- **Statistics**: View counts and distribution across hierarchy levels
- **Search**: Multiple query modes for precise information retrieval

## Search Commands

| Command            | Description                              |
| ------------------ | ---------------------------------------- |
| `mathematics`      | Show all entries in Mathematics category |
| `computer science` | Show all entries in matching subcategory |
| `halting problem`  | Find entries containing both terms       |
| `cat:math`         | Filter by category (partial match)       |
| `sub:computer`     | Filter by subcategory (partial match)    |
| `#3`               | View result #3 in detail                 |
| `list`             | Show complete structure                  |
| `help`             | Display search help                      |
| `quit`             | Exit program                             |

## Search Behavior

- Single word matching a category/subcategory returns all its contents
- Multiple words require ALL terms to match (AND logic)
- Searches are case-insensitive
- Results show first 10 entries with numbered selection

## API Methods

```rust
Microfiche::from_csv(path)     // Import CSV
Microfiche::to_csv(path)        // Export CSV
Microfiche::validate_csv(path)  // Check integrity
Microfiche::stats()             // Get statistics
Microfiche::search(query)       // Basic search
Microfiche::advanced_search()   // Filtered search
Microfiche::preview(max_items) // Structure preview
```

## Dependencies

```toml
[dependencies]
csv = "1.3"
serde = { version = "1.0", features = ["derive"] }
```
