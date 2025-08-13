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
use users::{Groups, Users, UsersCache};

#[derive(Debug, Display, Clone)]
enum Types {
    File,
    Dir,
}

#[derive(Debug, Clone, ValueEnum)]
enum SortField {
    Name,
    Size,
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
struct Size {
    #[tabled(rename = "Size")]
    size: String,
}

#[derive(Debug, Tabled, Clone)]
struct MAC {
    #[tabled(rename = "Date Modified")]
    modified: String,
    #[tabled(rename = "Date Accessed")]
    accessed: String,
    #[tabled(rename = "Date Created")]
    created: String,
}

#[derive(Debug, Tabled, Clone)]
struct Permission {
    #[tabled(rename = "Permission")]
    permission: String,
}

#[derive(Debug, Tabled, Clone)]
struct Binary {
    #[tabled(rename = "Binary")]
    size: String,
}

#[derive(Debug, Tabled, Clone)]
struct GroupOwner {
    #[tabled(rename = "Owner")]
    owner: String,
    #[tabled(rename = "Group")]
    group: String,
}

#[derive(Debug, Parser)]
#[command(
    version,
    about,
    long_about = "List directory sizes with various display options.\n\n\
    A modern replacement for 'ls' with colorful output and additional features."
)]
struct Cli {
    path: Option<PathBuf>,

    // Filtering options
    #[arg(short, long, help = "Show hidden files (starting with '.')", help_heading = Some("FILTERING OPTIONS"))]
    all: bool,
    #[arg(short, long, help = "Show directories only", help_heading = Some("FILTERING OPTIONS"))]
    dirs: bool,
    #[arg(short, long, help = "Reverse the sort order", help_heading = Some("FILTERING OPTIONS"))]
    reverse: bool,
    #[arg(
        short = 'S',
        long,
        value_enum,
        default_value = "name",
        help = "Sort by specific field",
        long_help = "Sort criteria:\n\
        - name: Alphabetical order\n\
        - size: File size\n\
        - extension: File extension\n\
        - modified: Last modification time\n\
        - changed: Last status change time\n\
        - accessed: Last access time\n\
        - created: Creation time\n\
        - inode: Inode number\n\
        - file-type: Directory first then files\n\
        - none: No sorting",
        help_heading = Some("FILTERING OPTIONS")
    )]
    sort: SortField,
    #[arg(short = 'i', long = "git-ignore", help = "ignore files mentioned in \'.gitignore\'", help_heading = Some("FILTERING OPTIONS"))]
    git_ignore: bool,

    // Display options
    #[arg(short, long, help = "Show file permissions in Unix format", help_heading = Some("DISPLAY OPTIONS"))]
    permission: bool,
    #[arg(short, long, help = "Show file sizes (size)", help_heading = Some("DISPLAY OPTIONS"))]
    size: bool,
    #[arg(short, long, help = "list file sizes with binary prefixes", help_heading = Some("DISPLAY OPTIONS"))]
    binary: bool,
    #[arg(short = 'g', long = "group_and_owner", help = "list each file's group and owner", help_heading = Some("DISPLAY OPTIONS"))]
    group_and_owner: bool,
    #[arg(short = 't', long = "mac", help = "Show last MAC (modification/accessed/created) timestamp time", help_heading = Some("DISPLAY OPTIONS"))]
    mac: bool,
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

            if cli.permission && cli.size && cli.mac && cli.binary && cli.group_and_owner {
                // Show all fields
                let combined: Vec<(Basic, Size, Binary, GroupOwner, MAC, Permission)> = files;
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(4), Color::FG_BLUE);
                table.modify(Columns::one(5), Color::FG_BLUE);
                table.modify(Columns::one(6), Color::FG_YELLOW);
                table.modify(Columns::one(7), Color::FG_YELLOW);
                table.modify(Columns::one(8), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.permission && cli.size && cli.mac && cli.binary {
                // Show all fields
                let combined: Vec<(Basic, Size, Binary, MAC, Permission)> = files
                    .into_iter()
                    .map(|(basic, size, binary, _, modified, permission)| {
                        (basic, size, binary, modified, permission)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(4), Color::FG_YELLOW);
                table.modify(Columns::one(5), Color::FG_YELLOW);
                table.modify(Columns::one(6), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.binary && cli.group_and_owner && cli.mac {
                // Show all fields
                let combined: Vec<(Basic, Size, Binary, GroupOwner, MAC)> = files
                    .into_iter()
                    .map(|(basic, size, binary, group_and_owner, modified, _)| {
                        (basic, size, binary, group_and_owner, modified)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(4), Color::FG_BLUE);
                table.modify(Columns::one(5), Color::FG_BLUE);
                table.modify(Columns::one(6), Color::FG_YELLOW);
                table.modify(Columns::one(7), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.binary && cli.group_and_owner && cli.permission {
                // Show all fields
                let combined: Vec<(Basic, Size, Binary, GroupOwner, Permission)> = files
                    .into_iter()
                    .map(|(basic, size, binary, group_and_owner, _, permission)| {
                        (basic, size, binary, group_and_owner, permission)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(4), Color::FG_BLUE);
                table.modify(Columns::one(5), Color::FG_BLUE);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.group_and_owner && cli.mac && cli.permission {
                // Show all fields
                let combined: Vec<(Basic, Size, GroupOwner, MAC, Permission)> = files
                    .into_iter()
                    .map(|(basic, size, _, group_and_owner, modified, permission)| {
                        (basic, size, group_and_owner, modified, permission)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BLUE);
                table.modify(Columns::one(4), Color::FG_BLUE);
                table.modify(Columns::one(5), Color::FG_YELLOW);
                table.modify(Columns::one(6), Color::FG_YELLOW);
                table.modify(Columns::one(7), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.binary && cli.group_and_owner && cli.mac && cli.permission {
                // Show all fields
                let combined: Vec<(Basic, Binary, GroupOwner, MAC, Permission)> = files
                    .into_iter()
                    .map(
                        |(basic, _, binary, group_and_owner, modified, permission)| {
                            (basic, binary, group_and_owner, modified, permission)
                        },
                    )
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BLUE);
                table.modify(Columns::one(4), Color::FG_BLUE);
                table.modify(Columns::one(5), Color::FG_YELLOW);
                table.modify(Columns::one(6), Color::FG_YELLOW);
                table.modify(Columns::one(7), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.binary && cli.mac {
                // Show size, binary and modifier
                let combined: Vec<(Basic, Size, Binary, MAC)> = files
                    .into_iter()
                    .map(|(basic, size, binary, _, modified, _)| (basic, size, binary, modified))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(4), Color::FG_YELLOW);
                table.modify(Columns::one(5), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.binary && cli.group_and_owner {
                // Show size, binary and modifier
                let combined: Vec<(Basic, Size, Binary, GroupOwner)> = files
                    .into_iter()
                    .map(|(basic, size, binary, group_and_owner, _, _)| {
                        (basic, size, binary, group_and_owner)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(4), Color::FG_BLUE);
                table.modify(Columns::last(), Color::FG_BLUE);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.group_and_owner && cli.mac {
                // Show size, binary and modifier
                let combined: Vec<(Basic, Size, GroupOwner, MAC)> = files
                    .into_iter()
                    .map(|(basic, size, _, group_and_owner, modified, _)| {
                        (basic, size, group_and_owner, modified)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BLUE);
                table.modify(Columns::one(4), Color::FG_BLUE);
                table.modify(Columns::one(5), Color::FG_YELLOW);
                table.modify(Columns::one(6), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.group_and_owner && cli.permission {
                // Show size, binary and modifier
                let combined: Vec<(Basic, Size, GroupOwner, Permission)> = files
                    .into_iter()
                    .map(|(basic, size, _, group_and_owner, _, permission)| {
                        (basic, size, group_and_owner, permission)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BLUE);
                table.modify(Columns::one(4), Color::FG_BLUE);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.binary && cli.group_and_owner && cli.mac {
                // Show size, binary and modifier
                let combined: Vec<(Basic, Binary, GroupOwner, MAC)> = files
                    .into_iter()
                    .map(|(basic, _, binary, group_and_owner, modified, _)| {
                        (basic, binary, group_and_owner, modified)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BLUE);
                table.modify(Columns::one(4), Color::FG_BLUE);
                table.modify(Columns::one(5), Color::FG_YELLOW);
                table.modify(Columns::one(6), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.binary && cli.group_and_owner && cli.permission {
                // Show size, binary and modifier
                let combined: Vec<(Basic, Binary, GroupOwner, Permission)> = files
                    .into_iter()
                    .map(|(basic, _, binary, group_and_owner, _, permission)| {
                        (basic, binary, group_and_owner, permission)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BLUE);
                table.modify(Columns::one(4), Color::FG_BLUE);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.binary && cli.permission {
                // Show size, binary and modifier
                let combined: Vec<(Basic, Size, Binary, Permission)> = files
                    .into_iter()
                    .map(|(basic, size, binary, _, _, permission)| {
                        (basic, size, binary, permission)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.mac && cli.permission {
                // Show size, binary and modifier
                let combined: Vec<(Basic, Size, MAC, Permission)> = files
                    .into_iter()
                    .map(|(basic, size, _, _, modified, permission)| {
                        (basic, size, modified, permission)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_YELLOW);
                table.modify(Columns::one(4), Color::FG_YELLOW);
                table.modify(Columns::one(5), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.binary && cli.mac && cli.permission {
                // Show size, binary and modifier
                let combined: Vec<(Basic, Binary, MAC, Permission)> = files
                    .into_iter()
                    .map(|(basic, _, binary, _, modified, permission)| {
                        (basic, binary, modified, permission)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_YELLOW);
                table.modify(Columns::one(4), Color::FG_YELLOW);
                table.modify(Columns::one(5), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.binary {
                // Show size and binary
                let combined: Vec<(Basic, Size, Binary)> = files
                    .into_iter()
                    .map(|(basic, size, binary, _, _, _)| (basic, size, binary))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.binary && cli.mac {
                // Show size and binary
                let combined: Vec<(Basic, Binary, MAC)> = files
                    .into_iter()
                    .map(|(basic, _, binary, _, modified, _)| (basic, binary, modified))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_YELLOW);
                table.modify(Columns::one(4), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.binary && cli.permission {
                // Show binary and permission
                let combined: Vec<(Basic, Binary, Permission)> = files
                    .into_iter()
                    .map(|(basic, _, binary, _, _, permission)| (basic, binary, permission))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.permission && cli.size {
                // Show permission and size
                let combined: Vec<(Basic, Size, Permission)> = files
                    .into_iter()
                    .map(|(basic, size, _, _, _, permission)| (basic, size, permission))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.permission && cli.mac {
                // Show permission and modified time
                let combined: Vec<(Basic, MAC, Permission)> = files
                    .into_iter()
                    .map(|(basic, _, _, _, modified, permission)| (basic, modified, permission))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_YELLOW);
                table.modify(Columns::one(3), Color::FG_YELLOW);
                table.modify(Columns::one(4), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.mac {
                // Show size and modified time
                let combined: Vec<(Basic, Size, MAC)> = files
                    .into_iter()
                    .map(|(basic, size, _, _, modified, _)| (basic, size, modified))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_YELLOW);
                table.modify(Columns::one(4), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size && cli.group_and_owner {
                // show size and grop/owner
                let combined: Vec<(Basic, Size, GroupOwner)> = files
                    .into_iter()
                    .map(|(basic, size, _, group_and_owner, _, _)| (basic, size, group_and_owner))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BLUE);
                table.modify(Columns::last(), Color::FG_BLUE);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.binary && cli.group_and_owner {
                // show size and grop/owner
                let combined: Vec<(Basic, Binary, GroupOwner)> = files
                    .into_iter()
                    .map(|(basic, _, binary, group_and_owner, _, _)| {
                        (basic, binary, group_and_owner)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
                table.modify(Columns::one(3), Color::FG_BLUE);
                table.modify(Columns::last(), Color::FG_BLUE);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.permission && cli.group_and_owner {
                // show size and grop/owner
                let combined: Vec<(Basic, GroupOwner, Permission)> = files
                    .into_iter()
                    .map(|(basic, _, _, group_and_owner, _, permission)| {
                        (basic, group_and_owner, permission)
                    })
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BLUE);
                table.modify(Columns::one(3), Color::FG_BLUE);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.permission {
                // Show only permission
                let combined: Vec<(Basic, Permission)> = files
                    .into_iter()
                    .map(|(basic, _, _, _, _, permission)| (basic, permission))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::last(), Color::FG_BRIGHT_GREEN);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.size {
                // Show only size
                let combined: Vec<(Basic, Size)> = files
                    .into_iter()
                    .map(|(basic, size, _, _, _, _)| (basic, size))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::last(), Color::FG_BRIGHT_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.mac {
                // Show only modified time
                let combined: Vec<(Basic, MAC)> = files
                    .into_iter()
                    .map(|(basic, _, _, _, modified, _)| (basic, modified))
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_YELLOW);
                table.modify(Columns::one(3), Color::FG_YELLOW);
                table.modify(Columns::last(), Color::FG_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.binary {
                let combined: Vec<(Basic, Binary)> = files
                    .into_iter()
                    .map(|(basic, _, binary, _, _, _)| (basic, binary)) // Changed to access binary field
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::last(), Color::FG_BRIGHT_YELLOW);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else if cli.group_and_owner {
                let combined: Vec<(Basic, GroupOwner)> = files
                    .into_iter()
                    .map(|(basic, _, _, group_and_owner, _, _)| (basic, group_and_owner)) // Changed to access binary field
                    .collect();
                let mut table = Table::new(combined);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
                table.modify(Columns::one(2), Color::FG_BLUE);
                table.modify(Columns::last(), Color::FG_BLUE);
                table.modify(Rows::first(), Color::FG_BRIGHT_BLACK);
                println!("{}", table);
            } else {
                // Show basic info only
                let basic_info: Vec<Basic> = files
                    .into_iter()
                    .map(|(basic, _, _, _, _, _)| basic)
                    .collect();
                let mut table = Table::new(basic_info);
                table.with(Style::empty());
                table.modify(Columns::one(1), Color::FG_MAGENTA);
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
) -> Vec<(Basic, Size, Binary, GroupOwner, MAC, Permission)> {
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

                if directories_only && meta.is_file() {
                    return false;
                }
                if show_hidden && file_name.starts_with('.') {
                    if git_ignore && file_name.eq(".gitignore") {
                        return false;
                    }
                    return true;
                } else if !show_hidden && file_name.starts_with('.') {
                    return false;
                }
                return true;
            })
            .collect()
        })
        .unwrap_or_default();

    // Sort entries based on the specified field
    match sort {
        SortField::Name => {
            entries.sort_by(|a, b| a.0.file_name().cmp(&b.0.file_name()));
        }
        SortField::Size => {
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
                size_mode(&meta),
                binary_mode(&meta),
                group_and_owner_mode(&meta),
                mac_mode(&meta),
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

fn size_mode(meta: &Metadata) -> Size {
    Size {
        size: human_readable_size(meta.len()),
    }
}

fn mac_mode(meta: &Metadata) -> MAC {
    MAC {
        modified: if let Ok(modi) = meta.modified() {
            let date: DateTime<Utc> = modi.into();
            format!("{}", date.format("%a %b %e %Y"))
        } else {
            String::default()
        },

        accessed: if let Ok(access) = meta.accessed() {
            let date: DateTime<Utc> = access.into();
            format!("{}", date.format("%a %b %e %Y"))
        } else {
            String::default()
        },
        created: if let Ok(created) = meta.created() {
            let date: DateTime<Utc> = created.into();
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

fn binary_mode(meta: &Metadata) -> Binary {
    Binary {
        size: meta.len().to_string(),
    }
}

fn group_and_owner_mode(meta: &Metadata) -> GroupOwner {
    let cache = UsersCache::new();
    let uid = meta.uid();
    let gid = meta.gid();

    GroupOwner {
        owner: cache
            .get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| uid.to_string()),

        group: cache
            .get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| gid.to_string()),
    }
}

fn human_readable_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "K", "M", "G", "T", "P"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    // Show 1 decimal place only if needed
    if size >= 10.0 || unit_index == 0 {
        format!("{:.0}{}", size, UNITS[unit_index])
    } else {
        format!("{:.1}{}", size, UNITS[unit_index])
    }
}
