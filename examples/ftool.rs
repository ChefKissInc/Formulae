/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![deny(warnings, unused_extern_crates, clippy::cargo, rust_2021_compatibility)]

use std::io::{BufWriter, Write};

use clap::{AppSettings, Parser, Subcommand};
use hashbrown::HashMap;

#[derive(Parser)]
#[clap(name = "ftool")]
#[clap(about = "Formulae binary configuration format manipulation example CLI", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    New {
        #[clap(required = true, parse(from_os_str))]
        filename: std::path::PathBuf,
    },
    Set {
        #[clap(required = true, parse(from_os_str))]
        filename: std::path::PathBuf,
        #[clap(required = true)]
        path: String,
        #[clap(required = true)]
        value: String,
    },
    Read {
        #[clap(required = true, parse(from_os_str))]
        filename: std::path::PathBuf,
        #[clap(required = false)]
        path: Option<String>,
    },
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::New { filename } => {
            let mut data = HashMap::new();
            data.insert("Cool".to_string(), formulae::Node::Bool(true));
            data.insert("somenumber".to_string(), formulae::Node::Int64(0xABCDEF));
            data.insert(
                "A string".to_string(),
                formulae::Node::String("hello world".to_string()),
            );
            data.insert(
                "array".to_string(),
                formulae::Node::Array(vec![
                    formulae::Node::String("hello world".to_string()),
                    formulae::Node::Int64(0xABCDEF),
                ]),
            );
            let mut map = HashMap::new();
            map.insert("macos".to_string(), formulae::Node::Bool(true));
            map.insert("me".to_string(), formulae::Node::Bool(true));
            map.insert("microsoft".to_string(), formulae::Node::Bool(false));
            data.insert("is_cool".to_string(), formulae::Node::Dictionary(map));
            BufWriter::new(std::fs::File::create(filename).unwrap())
                .write(&formulae::Node::Root(data).into_bytes())
                .unwrap();
        }
        Commands::Set {
            filename: _,
            path: _,
            value: _,
        } => {
            unimplemented!()
        }
        Commands::Read { filename, path } => {
            let contents = std::fs::read(filename).unwrap();
            let contents = formulae::Node::parse_root(&contents).unwrap();
            if let Some(path) = path {
                let mut node = Some(&contents);
                for node_path in path.split(".") {
                    node = match node {
                        Some(formulae::Node::Root(map) | formulae::Node::Dictionary(map)) => {
                            map.get(node_path)
                        }
                        Some(formulae::Node::Array(nodes)) => {
                            nodes.get(node_path.parse::<usize>().unwrap())
                        }
                        None => panic!("Path to node not found"),
                        _ => {
                            panic!(
                                "Node of type {:#X?} cannot be indexed",
                                node.unwrap().to_node_type()
                            )
                        }
                    };
                }
                if let Some(node) = node {
                    println!("{:#X?}", node);
                } else {
                    panic!("Path to node not found")
                }
            } else {
                println!("{:#X?}", contents);
            }
        }
    }
}
