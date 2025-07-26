use chrono::DateTime;
use chrono::Utc;
use clap::Parser;
use clap::ValueEnum;
use owo_colors::OwoColorize;
use std::fs::DirEntry;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
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

#[derive(Debug, Clone, ValueEnum)]
enum SortField {
    Name,
    Content,
    Extension,
    Modified,
    Changed,
    Accessed,
    Created,
    Inode,
    FileType,
    None,
}

#[derive(Debug, Tabled, Clone)]
struct Basic {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    types: Types,
}

#[derive(Debug, Tabled, Clone)]
struct Content {
    #[tabled(rename = "Content")]
    content: String,
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
    long_about = "List directory contents with various display options.\n\n\
    A modern replacement for 'ls' with colorful output and additional features."
)]
struct Cli {
    path: Option<PathBuf>,
    #[arg(short, long, help = "Show hidden files (starting with '.')")]
    all: bool,
    #[arg(short, long, help = "Show file permissions in Unix format")]
    permission: bool,
    #[arg(short, long, help = "Show file sizes (content)")]
    content: bool,
    #[arg(short, long, help = "Show last modification time")]
    modified_time: bool,
    #[arg(short, long, help = "Reverse the sort order")]
    reverse: bool,
    #[arg(short, long, help = "Show directories only")]
    dirs: bool,
    #[arg(
        short,
        long,
        value_enum,
        default_value = "name",
        help = "Sort by specific field",
        long_help = "Sort criteria:\n\
        - name: Alphabetical order\n\
        - content: File size\n\
        - extension: File extension\n\
        - modified: Last modification time\n\
        - changed: Last status change time\n\
        - accessed: Last access time\n\
        - created: Creation time\n\
        - inode: Inode number\n\
        - file-type: Directory first then files\n\
        - none: No sorting"
    )]
    sort: SortField,
    #[arg(short = None, long = "git-ignore", help = "Respect .gitignore files")]
    git_ignore: bool,
}

fn main() {
    let cli: Cli = Cli::parse();
    let path: PathBuf = cli.path.unwrap_or(PathBuf::from("."));

    println!("Path: {}", path.display());
    if let Ok(is_exist) = fs::exists(&path) {
        if is_exist {
            let files = get_files(
                &path,
                cli.all,
                cli.reverse,
                cli.dirs,
                cli.sort,
                cli.git_ignore,
            );

            if cli.permission && cli.content && cli.modified_time {
                // Show all fields
                let combined: Vec<(Basic, Content, Modified, Permission)> = files;
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.permission && cli.content {
                // Show permission and content
                let combined: Vec<(Basic, Content, Permission)> = files
                    .into_iter()
                    .map(|(basic, content, _, permission)| (basic, content, permission))
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
            } else if cli.content && cli.modified_time {
                // Show content and modified time
                let combined: Vec<(Basic, Content, Modified)> = files
                    .into_iter()
                    .map(|(basic, content, modified, _)| (basic, content, modified))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::last(), Color::FG_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.content {
                // Show only content
                let combined: Vec<(Basic, Content)> = files
                    .into_iter()
                    .map(|(basic, content, _, _)| (basic, content))
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

fn get_files(
    path: &Path,
    show_hidden: bool,
    reverse: bool,
    directories_only: bool,
    sort: SortField,
    git_ignore: bool,
) -> Vec<(Basic, Content, Modified, Permission)> {
    let mut entries: Vec<_> = fs::read_dir(path)
        .ok()
        .map(|dir| {
            dir.filter_map(|entry| {
                let entry = entry.ok()?;
                let meta = entry.metadata().ok()?;
                Some((entry, meta))
            })
            .filter(|(entry, meta)| {
                let file_name = entry.file_name().into_string().unwrap_or_default();
                (show_hidden || !file_name.starts_with('.'))
                    && (!directories_only || meta.is_dir())
                    && (git_ignore && !file_name.eq(".gitignore"))
            })
            .collect()
        })
        .unwrap_or_default();

    // Sort entries based on the specified field
    match sort {
        SortField::Name => {
            entries.sort_by(|a, b| a.0.file_name().cmp(&b.0.file_name()));
        }
        SortField::Content => {
            entries.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
        }
        SortField::Extension => {
            entries.sort_by(|a, b| {
                let name_a = a.0.file_name();
                let name_b = b.0.file_name();

                let ext_a = Path::new(&name_a)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let ext_b = Path::new(&name_b)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                ext_a.cmp(ext_b)
            });
        }
        SortField::Modified => {
            entries.sort_by(|a, b| {
                a.1.modified()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                    .cmp(&b.1.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH))
            });
        }
        SortField::Changed => {
            entries.sort_by(|a, b| {
                a.1.accessed()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                    .cmp(&b.1.accessed().unwrap_or(std::time::SystemTime::UNIX_EPOCH))
            });
        }
        SortField::Accessed => {
            entries.sort_by(|a, b| {
                a.1.created()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                    .cmp(&b.1.created().unwrap_or(std::time::SystemTime::UNIX_EPOCH))
            });
        }
        SortField::Created => {
            entries.sort_by(|a, b| {
                a.1.created()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                    .cmp(&b.1.created().unwrap_or(std::time::SystemTime::UNIX_EPOCH))
            });
        }
        SortField::Inode => {
            entries.sort_by(|a, b| a.1.ino().cmp(&b.1.ino()));
        }
        SortField::FileType => {
            entries.sort_by(|a, b| {
                let a_type = a.1.file_type();
                let b_type = b.1.file_type();

                match (a_type.is_dir(), b_type.is_dir()) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.0.file_name().cmp(&b.0.file_name()),
                }
            });
        }
        SortField::None => {}
    }

    if reverse {
        entries.reverse();
    }

    entries
        .into_iter()
        .map(|(file, meta)| {
            (
                basic_mode(&file, &meta),
                content_mode(&meta),
                modified_mode(&meta),
                permission_mode(&meta),
            )
        })
        .collect()
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

fn content_mode(meta: &Metadata) -> Content {
    Content {
        content: human_readable_content(meta.len()),
    }
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

fn human_readable_content(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "K", "M", "G", "T", "P"];
    let mut content = bytes as f64;
    let mut unit_index = 0;

    while content >= 1024.0 && unit_index < UNITS.len() - 1 {
        content /= 1024.0;
        unit_index += 1;
    }

    // Show 1 decimal place only if needed
    if content >= 10.0 || unit_index == 0 {
        format!("{:.0}{}", content, UNITS[unit_index])
    } else {
        format!("{:.1}{}", content, UNITS[unit_index])
    }
}
