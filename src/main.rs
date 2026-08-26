use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};


fn main() {
    // Definierar vilken mapp sökningen börjar i
    let start_dir = Path::new("/home/pakn/Disk021T/docker/docker-compose");

    // Kontroledrar att mappen finns
    if !start_dir.exists() {
        eprintln!("Fel: Sökvägen {:?} existerar inte!", start_dir);
        return;
    }

    println!("Söker efter Docker Compose projekt i: {:?}", start_dir);
    let mut compose_dirs: Vec<PathBuf> = Vec::new();
    find_compose_files(start_dir, &mut compose_dirs);

    if compose_dirs.is_empty() {
        println!("Hittade inga Docker Compose-filer.");
        return;
    }

    println!("\nHittade följande projekt:");
    for dir in &compose_dirs {
        println!("  - {:?}", dir);
    }

    // 1. Stoppa alla projekt
    //println!("\n=== 1. STOPPAR ALLA PROJEKT ===");
    //for dir in &compose_dirs {
    //    println!("\nStoppar i: {:?}", dir);
    //    run_command_live(dir, "docker", &["compose", "stop"]);
    //}

    // 2. Kör docker system prune
    //println!("\n=== 2. KÖR DOCKER SYSTEM PRUNE ===");
    //run_command_live(start_dir, "docker", &["system", "prune", "-a", "--force"]);

    // 3. Startar alla projekt igen
    println!("\n=== 3. STARTAR ALLA PROJEKT IGEN ===");
    for dir in &compose_dirs {
        println!("\nStartar i: {:?}", dir);
        run_command_live(dir, "docker", &["compose", "up", "-d"]);
    }

    println!("\n=== KLART, Alla processer har körts. ===");

}

// Rekursiv funktion för att hitta mappar med docker-compose.yml eller docker-compose.yaml
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

        // Om mappen innehåller en compose-fil, spara den
        if has_compose {
            compose_dirs.push(dir.to_path_buf());
        }

        // Sök vidare i underkataloger
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

// Funktion som kör kommandot och strömmar output (stdout och stderr) till konsolen i realtid
fn run_command_live(dir: &Path, program: &str, args: &[&str]) {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Misslyckades med att starta kommandot");

    let stdout = child.stdout.take().expect("Kunde inte öppna stdout");
    let stderr = child.stderr.take().expect("Kunde inte öppna stderr");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Trådar för att läsa stdout och stderr samtidigt så att de inte blockerar varandra
    let stdout_handle = std::thread::spawn(move || {
        for line in stdout_reader.lines().flatten() {
            println!("{}", line);
        }
    });

    let stderr_handle = std::thread::spawn(move || {
        for line in stderr_reader.lines().flatten() {
            eprintln!("{}", line);
        }
    });

    // Vänta på att trådarna och kommandot ska bli klara
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();
    let _ = child.wait().expect("Kommandot misslyckades under körning");
}

