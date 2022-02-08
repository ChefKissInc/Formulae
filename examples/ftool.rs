/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![deny(warnings, unused_extern_crates, clippy::cargo, rust_2021_compatibility)]

use std::io::{BufWriter, Write};

use clap::{AppSettings, ErrorKind, IntoApp, Parser, Subcommand};
use formulae::{obj_types, Object};
use hashbrown::HashMap;

#[derive(Parser)]
#[clap(
    author = "Copyright 2021-2022 VisualDevelopment. All rights reserved.",
    version,
    about = "Formulae binary configuration format manipulation example CLI",
    long_about = None
)]
struct Cli {
    #[clap(short, long, required = true, parse(from_os_str))]
    filename: std::path::PathBuf,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New,
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Add {
        #[clap(short, long, required = false)]
        path: Option<String>,
        #[clap(short = 't', long = "type", required = true, parse(try_from_str))]
        obj_type: u8,
        #[clap(short, long, required = false)]
        name: Option<String>,
        #[clap(short, long, required = false)]
        value: Option<String>,
    },
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Set {
        #[clap(required = true)]
        path: String,
        #[clap(required = true)]
        value: String,
    },
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Rename {
        #[clap(required = true)]
        path: String,
        #[clap(required = true)]
        name: String,
    },
    Read {
        #[clap(required = false)]
        path: Option<String>,
    },
}

fn split_path(mut path: &str) -> Vec<String> {
    let mut buf = vec![];
    let mut escaped = false;

    'outer: loop {
        let mut s = String::new();
        for (n, c) in path.char_indices() {
            match c {
                _ if escaped => {
                    s.push(c);
                    escaped = false;
                }
                '\\' => escaped = true,
                '.' => {
                    buf.push(s);
                    path = &path[n + 1..];
                    continue 'outer;
                }
                _ => s.push(c),
            }
        }
        buf.push(s);
        break buf;
    }
}

fn traverse_path<'a>(path: &'a str, mut object: &'a mut Object) -> Result<&'a mut Object, String> {
    for path in split_path(path) {
        object = match object {
            Object::Root(data) | Object::Dictionary(data) => {
                data.get_mut(&path).map_or_else(
                    || Err(format!("Path to object '{}' missing", path)),
                    |v| Ok(v),
                )?
            }
            Object::Array(items) => {
                items
                    .get_mut(path.parse::<usize>().map_or_else(
                        |e| return Err(format!("Failed to parse index '{}': {:#X?}", path, e)),
                        |v| Ok(v),
                    )?)
                    .map_or_else(
                        || Err(format!("Index to object '{}' missing", path)),
                        |v| Ok(v),
                    )?
            }
            _ => return Err(format!("Object of type {:#X?} cannot be indexed", object)),
        };
    }

    Ok(object)
}

fn main() {
    let args = Cli::parse();
    let mut app = Cli::into_app();

    match &args.command {
        Commands::New {} => {
            let data = HashMap::new();
            BufWriter::new(std::fs::File::create(&args.filename).unwrap())
                .write(&Object::Root(data).into_bytes())
                .unwrap();
        }
        Commands::Add {
            path,
            obj_type,
            name,
            value,
        } => {
            let contents = std::fs::read(&args.filename).unwrap();
            let mut contents = Object::parse_root(&contents).unwrap();
            let object = if let Some(path) = path {
                match traverse_path(path, &mut contents) {
                    Ok(v) => v,
                    Err(e) => app.error(ErrorKind::ArgumentNotFound, e).exit(),
                }
            } else {
                &mut contents
            };

            let value = match *obj_type {
                obj_types::BOOL => {
                    if let Some(value) = value {
                        Object::Bool(value.parse().unwrap())
                    } else {
                        app.error(ErrorKind::MissingRequiredArgument, "Value argument missing")
                            .exit()
                    }
                }
                obj_types::UINT32 => {
                    if let Some(value) = value {
                        Object::UInt32(value.parse().unwrap())
                    } else {
                        app.error(ErrorKind::MissingRequiredArgument, "Value argument missing")
                            .exit()
                    }
                }
                obj_types::UINT64 => {
                    if let Some(value) = value {
                        Object::UInt64(value.parse().unwrap())
                    } else {
                        app.error(ErrorKind::MissingRequiredArgument, "Value argument missing")
                            .exit()
                    }
                }
                obj_types::STR => {
                    if let Some(value) = value {
                        Object::String(value.clone())
                    } else {
                        app.error(ErrorKind::MissingRequiredArgument, "Value argument missing")
                            .exit()
                    }
                }
                obj_types::DICT => {
                    if let None = value {
                        Object::Dictionary(HashMap::new())
                    } else {
                        app.error(
                            ErrorKind::ArgumentConflict,
                            "Inserting an object of Dict type in combination with the value \
                             argument is not allowed",
                        )
                        .exit()
                    }
                }
                obj_types::ARRAY => {
                    if let None = value {
                        Object::Array(Vec::new())
                    } else {
                        app.error(
                            ErrorKind::ArgumentConflict,
                            "Inserting an object of Array type in combination with the value \
                             argument is not allowed",
                        )
                        .exit()
                    }
                }
                _ => {
                    app.error(
                        ErrorKind::InvalidValue,
                        format!("Invalid type '{}'", obj_type),
                    )
                    .exit()
                }
            };

            match object {
                Object::Root(data) | Object::Dictionary(data) => {
                    if let Some(name) = name {
                        match data
                            .try_insert(name.clone(), value)
                            .map_err(|e| e.to_string())
                        {
                            Ok(v) => println!("Successfully inserted element: {:#X?}", v),
                            Err(e) => app.error(ErrorKind::InvalidValue, e).exit(),
                        }
                    } else {
                        app.error(ErrorKind::ArgumentNotFound, "Missing name flag")
                            .exit()
                    }
                }
                Object::Array(items) => {
                    if let None = name {
                        items.push(value);

                        println!(
                            "Successfully inserted element: {:#X?}",
                            items.last().unwrap()
                        );
                    } else {
                        app.error(
                            ErrorKind::ArgumentConflict,
                            "Cannot name object inserted to Array object",
                        )
                        .exit()
                    }
                }
                _ => {
                    app.error(
                        ErrorKind::InvalidValue,
                        "Can only add object to Root, Dict or Array object",
                    )
                    .exit()
                }
            }

            BufWriter::new(std::fs::File::create(&args.filename).unwrap())
                .write(&contents.into_bytes())
                .unwrap();
        }
        Commands::Set { path, value } => {
            let contents = std::fs::read(&args.filename).unwrap();
            let mut contents = Object::parse_root(&contents).unwrap();
            let object = match traverse_path(path, &mut contents) {
                Ok(v) => v,
                Err(e) => app.error(ErrorKind::ArgumentNotFound, e).exit(),
            };

            match object {
                Object::Bool(val) => *val = value.parse().unwrap(),
                Object::UInt32(val) => *val = value.parse().unwrap(),
                Object::UInt64(val) => *val = value.parse().unwrap(),
                Object::String(val) => *val = value.clone(),
                _ => {
                    app.error(
                        ErrorKind::InvalidValue,
                        "Cannot change value of Root, Dict or Array object",
                    )
                    .exit()
                }
            }

            println!("Successfully set value to {:#X?}", object);

            BufWriter::new(std::fs::File::create(&args.filename).unwrap())
                .write(&contents.into_bytes())
                .unwrap();
        }
        Commands::Rename { path, name } => {
            let contents = std::fs::read(&args.filename).unwrap();
            let mut contents = Object::parse_root(&contents).unwrap();
            let mut parts = split_path(path);
            let old_name = parts.pop().unwrap();
            let path = parts.join(".");
            let parent = if path.is_empty() {
                &mut contents
            } else {
                match traverse_path(&path, &mut contents) {
                    Ok(v) => v,
                    Err(e) => app.error(ErrorKind::ArgumentNotFound, e).exit(),
                }
            };

            match parent {
                Object::Root(data) | Object::Dictionary(data) => {
                    let old = data.remove(&old_name.to_string()).unwrap();
                    data.insert(name.clone(), old);
                }
                _ => {
                    app.error(
                        ErrorKind::InvalidValue,
                        "Tried to rename object, parent of which is not Root or Dict object",
                    )
                    .exit()
                }
            }

            BufWriter::new(std::fs::File::create(&args.filename).unwrap())
                .write(&contents.into_bytes())
                .unwrap();

            println!("Successfully renamed object from {} to {}", old_name, name);
        }
        Commands::Read { path } => {
            let contents = std::fs::read(&args.filename).unwrap();
            let mut contents = Object::parse_root(&contents).unwrap();
            if let Some(path) = path {
                println!(
                    "{:#X?}",
                    match traverse_path(path, &mut contents) {
                        Ok(v) => v,
                        Err(e) => app.error(ErrorKind::ArgumentNotFound, e).exit(),
                    }
                );
            } else {
                println!("{:#X?}", contents);
            }
        }
    }
}

#[test]
fn verify_app() {
    Cli::into_app().debug_assert()
}
