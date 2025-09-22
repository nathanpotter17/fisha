# Fisha

A lightweight knowledge base system that leverages Unix tools (`awk`, `sed`, `grep`) for efficient CSV data organization and querying.

## Structure

Data is organized in a 5-level hierarchy:

```
Category → Subcategory → Concept → KeyDetail → Note
```

## CSV Format

Change the IN_FILE path `db/ethos.csv` at the top of main.rs to the appropriate path,
and be sure the following column structure is preserved.

| Category    | Subcategory | Concept           | KeyDetail  | Note                    |
| ----------- | ----------- | ----------------- | ---------- | ----------------------- |
| Mathematics | Algebra     | Quadratic Formula | Definition | x = (-b ± √(b²-4ac))/2a |

**Important**: Fields containing commas must be wrapped in quotes:

```csv
Classic Design Talks,Design,Alan Kay,COFES,"Rethinking Design, Risk, and Software - url"
```

## Usage

```bash
cargo run
```

## Features

- **Unix-Powered**: Uses `awk`, `sed`, and `grep` for blazing-fast text processing
- **Validation**: Shows problematic rows with detailed error messages
- **Statistics**: View counts and distribution using awk aggregations
- **Search**: Multiple query modes with grep-based pattern matching

## Search Commands

| Command                | Description                            |
| ---------------------- | -------------------------------------- |
| `search <term>`        | Search all fields for term             |
| `cat:<category>`       | Filter by category (partial match)     |
| `sub:<subcategory>`    | Filter by subcategory (partial match)  |
| `and <t1> <t2> ...`    | Search multiple terms (ALL must match) |
| `random <n>`           | Show n random entries                  |
| `unique <field>`       | Show unique values for field           |
| `export <term> <file>` | Export filtered results to file        |
| `stats`                | Display statistics                     |
| `list`                 | Show structure overview                |
| `help`                 | Display available commands             |
| `quit`                 | Exit program                           |

## Search Examples

```bash
> search byzantine
> cat:math
> sub:distributed systems
> and consensus algorithm
> random 5
> unique category
> export "machine learning" ml_entries.csv
```

## Core Methods

```rust
Fisha::new(path)                // Create instance
Fisha::validate()               // Validate CSV with row details
Fisha::stats()                  // Statistics via awk
Fisha::search(query)            // Search via grep
Fisha::search_by_field()        // Field-specific awk search
Fisha::search_all_terms()       // Multi-term grep pipeline
Fisha::category_distribution()  // Top categories by count
Fisha::list_structure()         // Structure overview
Fisha::random_samples(n)        // Random entries via shuf
Fisha::unique_values(field)     // Unique field values
Fisha::export_filtered()        // Export search results
```

## Unix Tools Used

- **awk**: Field processing, statistics, validation, aggregations
- **sed**: Text transformations, CSV escaping
- **grep**: Pattern matching, multi-term searches
- **sort**: Ordering results
- **shuf**: Random sampling

## Performance Benefits

- Stream processing without loading entire file into memory
- C-optimized tools handle large datasets efficiently
- Pipe composition for complex queries
- Parallel processing capability with GNU parallel

## Validation Output

The validator will shows problematic rows:

```
Line 175: Expected 5 fields, got 6
  Row: Learning Resources,Security,DEFCON,Talk,Title, Author - url
Line 205: Empty field 3
  Row: Mathematics,Algebra,,Definition,Missing concept field
✗ Found 2 validation errors
```

## System Requirements

- Unix-like system (Linux, macOS, WSL on Windows)
- Basic Unix tools: `awk`, `sed`, `grep`, `sort`, `shuf`
- Optional: `parallel` for parallel processing
