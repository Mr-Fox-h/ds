use clap::Parser;
use owo_colors::OwoColorize;
use std::fs::DirEntry;
use std::fs::Metadata;
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

#[derive(Debug, Display)]
enum Types {
    File,
    Dir,
}

#[derive(Debug, Tabled)]
struct Basic {
    #[tabled{rename="Name"}]
    name: String,
    #[tabled{rename="Type"}]
    types: Types,
}

// #[derive(Debug, Tabled)]
// struct Modified {
//     #[tabled{rename="Modified"}]
//     modified: String,
// }
//
// #[derive(Debug, Tabled)]
// struct Size {
//     #[tabled{rename="Size"}]
//     size: u64,
// }

#[derive(Debug, Parser)]
#[command(
    version,
    about,
    long_about = "List information about the FILEs/DIRECTORYs."
)]
struct Cli {
    path: Option<PathBuf>,
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
            if cli.size {
                todo!()
            } else {
                let get_files: Vec<Basic> = get_files(&path);
                let mut table = Table::new(get_files);
                table.with(Style::empty());
                table.modify(Columns::last(), Color::FG_YELLOW);
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

fn get_files(path: &Path) -> Vec<Basic> {
    let mut data = Vec::default();

    if let Ok(directory) = fs::read_dir(path) {
        for value in directory {
            if let Ok(file) = value {
                if let Ok(meta) = fs::metadata(&file.path()) {
                    data.push(basic_mode(&file, &meta));
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
            .unwrap_or("UNKNOW NAME".into()),
        types: if meta.is_dir() {
            Types::Dir
        } else {
            Types::File
        },
    }
}

// fn basic_mode(file: &DirEntry, meta: &Metadata) -> Basic {
//     Basic {
//         name: file
//             .file_name()
//             .into_string()
//             .unwrap_or("UNKNOW NAME".into()),
//         types: if meta.is_dir() {
//             Types::Dir
//         } else {
//             Types::File
//         },
//         size: meta.len(),
//         modified: if let Ok(modi) = meta.modified() {
//             // let date: DataTime<Utc> = modi.into();
//             let date: DateTime<Utc> = modi.into();
//             format!("{}", date.format("%a %b %e %Y"))
//         } else {
//             String::default()
//         },
//     }
// }
