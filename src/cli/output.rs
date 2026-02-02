use colored::Colorize;

pub fn success(message: &str) {
    println!("{} {}", "SUCCESS".green().bold(), message);
}

pub fn error(message: &str) {
    eprintln!("{} {}", "ERROR".red().bold(), message);
}

pub fn warning(message: &str) {
    println!("{} {}", "WARNING".yellow().bold(), message);
}

pub fn info(message: &str) {
    println!("{} {}", "INFO".blue().bold(), message);
}

pub fn version_line(version: &str, is_current: bool, path: &str) {
    if is_current {
        println!(
            "  {} {} {}",
            "*".green().bold(),
            version.green().bold(),
            format!("(current) [{}]", path).dimmed()
        );
    } else {
        println!("    {} {}", version, format!("[{}]", path).dimmed());
    }
}

pub fn step(number: u32, total: u32, message: &str) {
    println!(
        "{} {}",
        format!("[{}/{}]", number, total).cyan().bold(),
        message
    );
}

pub fn header(message: &str) {
    println!("\n{}", message.bold().underline());
}
