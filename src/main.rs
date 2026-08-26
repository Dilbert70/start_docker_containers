use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() {
    // 1. Läs konfigurationsfilen config.json
    let config_path = Path::new("config.json");
    if !config_path.exists() {
        let default_config = "{\n  \"docker_dir\": \"/home/user/docker\"\n}\n";
        if fs::write(config_path, default_config).is_ok() {
            println!("Skapade config.json. Ändra sökvägen och kör igen.");
        } else {
            eprintln!("Fel: Kunde inte skapa config.json");
        }
        return;
    }

    let config_content = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Fel vid läsning av config.json: {}", e);
            return;
        }
    };

    let start_dir_str = match parse_json_value(&config_content, "docker_dir") {
        Some(path) => path,
        None => {
            eprintln!("Fel: Hittade inte \"docker_dir\" i config.json");
            return;
        }
    };

    let start_dir = Path::new(&start_dir_str);
    if !start_dir.exists() {
        eprintln!("Fel: Sökvägen {:?} existerar inte!", start_dir);
        return;
    }

    println!("Söker efter inaktiva Docker Compose-projekt i: {:?}", start_dir);
    let mut compose_dirs: Vec<PathBuf> = Vec::new();
    find_compose_files(start_dir, &mut compose_dirs);

    if compose_dirs.is_empty() {
        println!("Hittade inga Docker Compose-filer.");
        return;
    }

    println!("\n=== ANALYSERAR PROJEKT ===");
    let mut projects_to_start = Vec::new();

    for dir in compose_dirs {
        // Kontroll 1: Finns det en blockfil i mappen?
        if has_block_file(&dir) {
            println!("  [BLOCKERAD] {:?}", dir);
            continue;
        }

        // Kontroll 2: Körs projektet redan?
        if is_compose_project_running(&dir) {
            println!("  [IGÅNG]     {:?}", dir);
            continue;
        }

        // Om båda kontrollerna passerar ska projektet startas
        println!("  [SKA STARTAS] {:?}", dir);
        projects_to_start.push(dir);
    }

    if projects_to_start.is_empty() {
        println!("\nInga projekt behövde startas (alla var igång eller blockerade).");
        return;
    }

    // Starta de utvalda projekten
    println!("\n=== STARTAR PROJEKT ===");
    for dir in &projects_to_start {
        println!("\nStartar i: {:?}", dir);
        run_command_live(dir, "docker", &["compose", "up", "-d"]);
    }

    println!("\n=== KLART! Alla tillgängliga processer har startats. ===");
}

// Kontrollerar om en blockfil (t.ex. "no_autostart") finns i mappen
fn has_block_file(dir: &Path) -> bool {
    let block_file_name = "no_autostart";
    dir.join(block_file_name).exists()
}

// Kontrollerar om ett docker compose-projekt har några aktiva containrar igång
fn is_compose_project_running(dir: &Path) -> bool {
    let output = Command::new("docker")
        .args(&["compose", "ps", "--format", "json"])
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    if let Ok(out) = output {
        let stdout_str = String::from_utf8_lossy(&out.stdout);
        let trimmed = stdout_str.trim();
        !trimmed.is_empty() && trimmed != "[]"
    } else {
        false
    }
}

// Rekursiv sökning efter mappar med docker-compose
fn find_compose_files(dir: &Path, compose_dirs: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut has_compose = false;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name == "docker-compose.yml" || file_name == "docker-compose.yaml" {
                        has_compose = true;
                    }
                }
            }
        }

        if has_compose {
            compose_dirs.push(dir.to_path_buf());
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    find_compose_files(&path, compose_dirs);
                }
            }
        }
    }
}

// Enkel JSON-parser utan externa bibliotek
fn parse_json_value(json: &str, key: &str) -> Option<String> {
    let search_pattern = format!("\"{}\"", key);
    if let Some(pos) = json.find(&search_pattern) {
        let remainder = &json[pos + search_pattern.len()..];
        if let Some(colon_pos) = remainder.find(':') {
            let value_part = &remainder[colon_pos + 1..];
            if let Some(start_quote) = value_part.find('"') {
                let text_after_quote = &value_part[start_quote + 1..];
                if let Some(end_quote) = text_after_quote.find('"') {
                    return Some(text_after_quote[..end_quote].to_string());
                }
            }
        }
    }
    None
}

// Kör kommandon med realtidsutmatning
fn run_command_live(dir: &Path, program: &str, args: &[&str]) {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::inherit()) // Låter Docker skriva direkt till din terminal
        .stderr(Stdio::inherit()) // Docker Compose skickar grafik/loggar hit
        .stdin(Stdio::null())
        .spawn()
        .expect("Misslyckades med att starta kommandot");

    let _ = child.wait().expect("Kommandot misslyckades under körning");
}

