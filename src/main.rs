use std::process::Command;
use std::io::Write;
use std::fs::File;
use std::error::Error;

const IN_FILE: &str = "db/ethos.csv";

struct Fisha {
    file_path: String,
}

impl Fisha {
    fn new(path: &str) -> Self {
        Fisha { 
            file_path: path.to_string() 
        }
    }
    
    // Use awk for statistics
    fn stats(&self) -> Result<String, Box<dyn Error>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(r#"
                awk -F',' '
                NR>1 {{
                    cats[$1]++
                    subcats[$1","$2]++
                    concepts[$1","$2","$3]++
                    details[$1","$2","$3","$4]++
                    total++
                }}
                END {{
                    print "Categories:", length(cats)
                    print "Subcategories:", length(subcats)
                    print "Concepts:", length(concepts)
                    print "Key Details:", length(details)
                    print "Total Notes:", total
                }}' {}"#, self.file_path))
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    // Use grep for searching
    fn search(&self, query: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let output = Command::new("grep")
            .arg("-i")  // case insensitive
            .arg(query)
            .arg(&self.file_path)
            .output()?;
        
        let results: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();
        
        Ok(results)
    }
    
    // Advanced search using awk for field-specific filtering
    fn search_by_field(&self, field: &str, value: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let field_num = match field {
            "category" => 1,
            "subcategory" => 2,
            "concept" => 3,
            "keydetail" => 4,
            "note" => 5,
            _ => 0,
        };
        
        let cmd = format!(
            r#"awk -F',' 'NR>1 && tolower(${}) ~ /{}/ {{ print }}' {}"#,
            field_num,
            value.to_lowercase(),
            self.file_path
        );
        
        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect())
    }
    
    // Use sed for transformations
    fn format_entry(&self, line: &str) -> Result<String, Box<dyn Error>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                r#"echo '{}' | sed 's/,/./g; s/"//g'"#,
                line
            ))
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    // Category distribution using awk
    fn category_distribution(&self) -> Result<String, Box<dyn Error>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(r#"
                awk -F',' 'NR>1 {{ count[$1]++ }}
                END {{ 
                    for (cat in count) 
                        print count[cat], cat 
                }}' {} | sort -rn | head -10"#, self.file_path))
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    // List structure with awk
    fn list_structure(&self) -> Result<String, Box<dyn Error>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(r#"
                awk -F',' 'NR>1 {{
                    key = $1 "," $2
                    if (!(key in seen)) {{
                        seen[key] = 1
                        subcats[$1] = subcats[$1] "  - " $2 "\n"
                    }}
                }}
                END {{
                    for (cat in subcats) {{
                        print "[" cat "]"
                        print subcats[cat]
                    }}
                }}' {} | sort"#, self.file_path))
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    // Multi-term AND search using grep pipeline
    fn search_all_terms(&self, terms: &[&str]) -> Result<Vec<String>, Box<dyn Error>> {
        let mut cmd = format!("cat {}", self.file_path);
        
        for term in terms {
            cmd.push_str(&format!(" | grep -i '{}'", term));
        }
        
        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect())
    }
    
    fn validate(&self) -> Result<String, Box<dyn Error>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(r#"
                awk -F',' '
                BEGIN {{ errors = 0 }}
                NR > 1 {{
                    if (NF != 5) {{
                        errors++
                        print "Line " NR ": Expected 5 fields, got " NF
                        print "  Row: " $0
                    }}
                    for (i=1; i<=NF && i<=5; i++) {{
                        if ($i == "") {{
                            errors++
                            print "Line " NR ": Empty field " i
                            print "  Row: " $0
                        }}
                    }}
                }}
                END {{
                    if (errors == 0) 
                        print "✓ CSV validation passed"
                    else
                        print "✗ Found " errors " validation errors"
                }}' {}"#, self.file_path))
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    // Get random samples using shuf
    fn random_samples(&self, count: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "tail -n +2 {} | shuf -n {}",
                self.file_path, count
            ))
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect())
    }
    
    // Extract unique values from a column
    fn unique_values(&self, field: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let field_num = match field {
            "category" => 1,
            "subcategory" => 2,
            "concept" => 3,
            "keydetail" => 4,
            _ => return Err("Invalid field".into()),
        };
        
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                r#"awk -F',' 'NR>1 {{ print ${} }}' {} | sort -u"#,
                field_num, self.file_path
            ))
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }
    
    // Export filtered results
    fn export_filtered(&self, filter: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
        let cmd = format!(
            "head -n 1 {} > {} && grep -i '{}' {} >> {}",
            self.file_path, output_file, filter, self.file_path, output_file
        );
        
        Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .status()?;
        
        println!("Exported filtered results to {}", output_file);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let fisha = Fisha::new(IN_FILE);
    
    println!("{}", fisha.validate()?);
    
    println!("\nStatistics:");
    println!("{}", fisha.stats()?);
    
    println!("\nTop Categories:");
    println!("{}", fisha.category_distribution()?);
    
    println!("\nRandom Sample:");
    for sample in fisha.random_samples(3)? {
        println!("  - {}", fisha.format_entry(&sample)?);
    }
    
    println!("\nSearch Interface (type 'help' for commands):");
    
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        match input {
            "quit" | "exit" => break,
            "help" => {
                println!("Commands:");
                println!("  search <term>        - Search all fields");
                println!("  cat:<category>       - Filter by category");
                println!("  sub:<subcategory>    - Filter by subcategory");
                println!("  and <t1> <t2> ...    - Search multiple terms (AND)");
                println!("  stats                - Show statistics");
                println!("  list                 - List structure");
                println!("  random <n>           - Show n random entries");
                println!("  unique <field>       - Show unique values");
                println!("  export <term> <file> - Export filtered results");
                println!("  quit                 - Exit");
            }
            cmd if cmd.starts_with("search ") => {
                let query = &cmd[7..];
                for result in fisha.search(query)? {
                    println!("  {}", fisha.format_entry(&result)?);
                }
            }
            cmd if cmd.starts_with("cat:") => {
                let cat = &cmd[4..];
                for result in fisha.search_by_field("category", cat)? {
                    println!("  {}", fisha.format_entry(&result)?);
                }
            }
            cmd if cmd.starts_with("sub:") => {
                let sub = &cmd[4..];
                for result in fisha.search_by_field("subcategory", sub)? {
                    println!("  {}", fisha.format_entry(&result)?);
                }
            }
            cmd if cmd.starts_with("and ") => {
                let terms: Vec<&str> = cmd[4..].split_whitespace().collect();
                for result in fisha.search_all_terms(&terms)? {
                    println!("  {}", fisha.format_entry(&result)?);
                }
            }
            cmd if cmd.starts_with("random ") => {
                if let Ok(n) = cmd[7..].parse::<usize>() {
                    for result in fisha.random_samples(n)? {
                        println!("  {}", fisha.format_entry(&result)?);
                    }
                }
            }
            cmd if cmd.starts_with("unique ") => {
                let field = &cmd[7..];
                println!("Unique values for {}:", field);
                for value in fisha.unique_values(field)? {
                    println!("  • {}", value);
                }
            }
            cmd if cmd.starts_with("export ") => {
                let parts: Vec<&str> = cmd[7..].split_whitespace().collect();
                if parts.len() == 2 {
                    fisha.export_filtered(parts[0], parts[1])?;
                }
            }
            "stats" => println!("{}", fisha.stats()?),
            "list" => println!("{}", fisha.list_structure()?),
            _ => println!("Unknown command. Type 'help' for commands."),
        }
    }
    
    Ok(())
}