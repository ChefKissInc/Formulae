/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![deny(warnings, unused_extern_crates, clippy::cargo, rust_2021_compatibility)]

use std::io::{BufWriter, Write};

use clap::{AppSettings, ErrorKind, IntoApp, Parser, Subcommand};
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
        node_type: u8,
        #[clap(required = true)]
        name: String,
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

fn traverse_path<'a>(
    path: &'a str,
    mut node: &'a mut formulae::Node,
) -> Result<&'a mut formulae::Node, String> {
    for node_path in split_path(path) {
        node = match node {
            formulae::Node::Root(map) | formulae::Node::Dictionary(map) => {
                map.get_mut(&node_path).map_or_else(
                    || Err(format!("Path to node '{}' missing", node_path)),
                    |v| Ok(v),
                )?
            }
            _ => return Err(format!("Node of type {:#X?} cannot be indexed", node)),
        };
    }

    Ok(node)
}

fn main() {
    let args = Cli::parse();
    let mut app = Cli::into_app();

    match &args.command {
        Commands::New {} => {
            let data = HashMap::new();
            BufWriter::new(std::fs::File::create(&args.filename).unwrap())
                .write(&formulae::Node::Root(data).into_bytes())
                .unwrap();
        }
        Commands::Add {
            path,
            node_type,
            name,
            value,
        } => {
            let contents = std::fs::read(&args.filename).unwrap();
            let mut contents = formulae::Node::parse_root(&contents).unwrap();
            let node = if let Some(path) = path {
                match traverse_path(path, &mut contents) {
                    Ok(node) => node,
                    Err(e) => app.error(ErrorKind::ArgumentNotFound, e).exit(),
                }
            } else {
                &mut contents
            };

            match node {
                formulae::Node::Root(map) | formulae::Node::Dictionary(map) => {
                    let value = match *node_type {
                        formulae::node_types::BOOL => {
                            if let Some(value) = value {
                                formulae::Node::Bool(value.parse().unwrap())
                            } else {
                                app.error(
                                    ErrorKind::MissingRequiredArgument,
                                    "Value argument missing",
                                )
                                .exit()
                            }
                        }
                        formulae::node_types::INT32 => {
                            if let Some(value) = value {
                                formulae::Node::Int32(value.parse().unwrap())
                            } else {
                                app.error(
                                    ErrorKind::MissingRequiredArgument,
                                    "Value argument missing",
                                )
                                .exit()
                            }
                        }
                        formulae::node_types::INT64 => {
                            if let Some(value) = value {
                                formulae::Node::Int64(value.parse().unwrap())
                            } else {
                                app.error(
                                    ErrorKind::MissingRequiredArgument,
                                    "Value argument missing",
                                )
                                .exit()
                            }
                        }
                        formulae::node_types::STR => {
                            if let Some(value) = value {
                                formulae::Node::String(value.clone())
                            } else {
                                app.error(
                                    ErrorKind::MissingRequiredArgument,
                                    "Value argument missing",
                                )
                                .exit()
                            }
                        }
                        formulae::node_types::DICT => {
                            if let None = value {
                                formulae::Node::Dictionary(HashMap::new())
                            } else {
                                app.error(
                                    ErrorKind::ArgumentConflict,
                                    "Inserting an object of Dict type in combination with the \
                                     value argument is not allowed",
                                )
                                .exit()
                            }
                        }
                        _ => {
                            app.error(
                                ErrorKind::InvalidValue,
                                format!("Invalid type '{}'", node_type),
                            )
                            .exit()
                        }
                    };

                    match map
                        .try_insert(name.clone(), value)
                        .map_err(|e| e.to_string())
                    {
                        Ok(v) => println!("Successfully inserted element: {:#X?}", v),
                        Err(e) => app.error(ErrorKind::InvalidValue, e).exit(),
                    }
                }
                formulae::Node::Bool(_)
                | formulae::Node::Int32(_)
                | formulae::Node::Int64(_)
                | formulae::Node::String(_) => {
                    panic!("Can only add node to Root or Dict object")
                }
            }

            BufWriter::new(std::fs::File::create(&args.filename).unwrap())
                .write(&contents.into_bytes())
                .unwrap();
        }
        Commands::Set { path, value } => {
            let contents = std::fs::read(&args.filename).unwrap();
            let mut contents = formulae::Node::parse_root(&contents).unwrap();
            let node = match traverse_path(path, &mut contents) {
                Ok(node) => node,
                Err(e) => app.error(ErrorKind::ArgumentNotFound, e).exit(),
            };

            println!("Before: {:#X?}", node);
            match node {
                formulae::Node::Root(_) | formulae::Node::Dictionary(_) => {
                    panic!("Cannot change value of Root or Dict object")
                }
                formulae::Node::Bool(val) => *val = value.parse().unwrap(),
                formulae::Node::Int32(val) => *val = value.parse().unwrap(),
                formulae::Node::Int64(val) => *val = value.parse().unwrap(),
                formulae::Node::String(val) => *val = value.clone(),
            }
            println!("After: {:#X?}", node);

            BufWriter::new(std::fs::File::create(&args.filename).unwrap())
                .write(&contents.into_bytes())
                .unwrap();
        }
        Commands::Rename { path, name } => {
            let contents = std::fs::read(&args.filename).unwrap();
            let mut contents = formulae::Node::parse_root(&contents).unwrap();
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

            println!("Before: {:#X?}", parent);
            match parent {
                formulae::Node::Root(map) | formulae::Node::Dictionary(map) => {
                    let old = map.remove(&old_name.to_string()).unwrap();
                    map.insert(name.clone(), old);
                }
                _ => unreachable!(),
            }
            println!("After: {:#X?}", parent);

            BufWriter::new(std::fs::File::create(&args.filename).unwrap())
                .write(&contents.into_bytes())
                .unwrap();
        }
        Commands::Read { path } => {
            let contents = std::fs::read(&args.filename).unwrap();
            let mut contents = formulae::Node::parse_root(&contents).unwrap();
            if let Some(path) = path {
                println!(
                    "{:#X?}",
                    match traverse_path(path, &mut contents) {
                        Ok(node) => node,
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
