use chrono::DateTime;
use chrono::Utc;
use clap::Parser;
use owo_colors::OwoColorize;
use std::fs::DirEntry;
use std::fs::Metadata;
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::{Path, PathBuf},
};
use strum::Display;
use tabled::settings::object::Columns;
use tabled::{
    Table, Tabled,
    settings::{Color, Style, object::Rows},
};

#[derive(Debug, Display, Clone)]
enum Types {
    File,
    Dir,
}

#[derive(Debug, Tabled, Clone)]
struct Basic {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    types: Types,
}

#[derive(Debug, Tabled, Clone)]
struct Size {
    #[tabled(rename = "Size")]
    size: u64,
}

#[derive(Debug, Tabled, Clone)]
struct Modified {
    #[tabled(rename = "Modified")]
    modified: String,
}

#[derive(Debug, Tabled, Clone)]
struct Permission {
    #[tabled(rename = "Permission")]
    permission: String,
}

#[derive(Debug, Parser)]
#[command(
    version,
    about,
    long_about = "List information about the FILEs/DIRECTORYs."
)]
struct Cli {
    path: Option<PathBuf>,
    #[arg(short, long)]
    permission: bool,
    #[arg(short, long)]
    size: bool,
    #[arg(short, long)]
    modified_time: bool,
}

fn main() {
    let cli: Cli = Cli::parse();
    let path: PathBuf = cli.path.unwrap_or(PathBuf::from("."));

    println!("Path: {}", path.display());
    if let Ok(is_exist) = fs::exists(&path) {
        if is_exist {
            let files = get_files(&path);

            if cli.permission && cli.size && cli.modified_time {
                // Show all fields
                let combined: Vec<(Basic, Size, Modified, Permission)> = files;
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.permission && cli.size {
                // Show permission and size
                let combined: Vec<(Basic, Size, Permission)> = files
                    .into_iter()
                    .map(|(basic, size, _, permission)| (basic, size, permission))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.permission && cli.modified_time {
                // Show permission and modified time
                let combined: Vec<(Basic, Modified, Permission)> = files
                    .into_iter()
                    .map(|(basic, _, modified, permission)| (basic, modified, permission))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(2), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.permission {
                // Show only permission
                let combined: Vec<(Basic, Permission)> = files
                    .into_iter()
                    .map(|(basic, _, _, permission)| (basic, permission))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.modified_time {
                // Show size and modified time
                let combined: Vec<(Basic, Size, Modified)> = files
                    .into_iter()
                    .map(|(basic, size, modified, _)| (basic, size, modified))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::last(), Color::FG_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size {
                // Show only size
                let combined: Vec<(Basic, Size)> = files
                    .into_iter()
                    .map(|(basic, size, _, _)| (basic, size))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::last(), Color::FG_BRIGHT_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.modified_time {
                // Show only modified time
                let combined: Vec<(Basic, Modified)> = files
                    .into_iter()
                    .map(|(basic, _, modified, _)| (basic, modified))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::last(), Color::FG_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else {
                // Show basic info only
                let basic_info: Vec<Basic> =
                    files.into_iter().map(|(basic, _, _, _)| basic).collect();
                let mut table = Table::new(basic_info);
                table.with(Style::empty());
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            }
        } else {
            println!(
                "{}",
                "error:\nPath doesn't exist. (try other location)".red()
            );
        }
    } else {
        println!("{}", "error:\nCan't read directory.".red());
    }
}

fn get_files(path: &Path) -> Vec<(Basic, Size, Modified, Permission)> {
    let mut data = Vec::new();

    if let Ok(directory) = fs::read_dir(path) {
        for value in directory {
            if let Ok(file) = value {
                if let Ok(meta) = fs::metadata(&file.path()) {
                    data.push((
                        basic_mode(&file, &meta),
                        size_mode(&meta),
                        modified_mode(&meta),
                        permission_mode(&meta),
                    ));
                }
            }
        }
    }
    data
}

fn basic_mode(file: &DirEntry, meta: &Metadata) -> Basic {
    Basic {
        name: file
            .file_name()
            .into_string()
            .unwrap_or("UNKNOWN NAME".into()),
        types: if meta.is_dir() {
            Types::Dir
        } else {
            Types::File
        },
    }
}

fn size_mode(meta: &Metadata) -> Size {
    Size { size: meta.len() }
}

fn modified_mode(meta: &Metadata) -> Modified {
    Modified {
        modified: if let Ok(modi) = meta.modified() {
            let date: DateTime<Utc> = modi.into();
            format!("{}", date.format("%a %b %e %Y"))
        } else {
            String::default()
        },
    }
}

fn permission_mode(meta: &Metadata) -> Permission {
    let permissions = meta.permissions();
    let mode = permissions.mode();

    let mut perm_string = String::with_capacity(10);

    // File type
    perm_string.push(if meta.is_dir() { 'd' } else { '-' });

    // User permissions
    perm_string.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    perm_string.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perm_string.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // Group permissions
    perm_string.push(if mode & 0o40 != 0 { 'r' } else { '-' });
    perm_string.push(if mode & 0o20 != 0 { 'w' } else { '-' });
    perm_string.push(if mode & 0o10 != 0 { 'x' } else { '-' });

    // Other permissions
    perm_string.push(if mode & 0o4 != 0 { 'r' } else { '-' });
    perm_string.push(if mode & 0o2 != 0 { 'w' } else { '-' });
    perm_string.push(if mode & 0o1 != 0 { 'x' } else { '-' });

    Permission {
        permission: perm_string,
    }
}
