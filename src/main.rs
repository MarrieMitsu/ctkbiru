use std::env;
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{BufReader, BufRead, BufWriter, Read, Write};
use std::ffi::{OsStr};

use anyhow::{Result, Context};
use clap::{Args, Parser, Subcommand};

extern crate dirs;

fn home_path() -> Result<PathBuf> {
    dirs::home_dir()
        .and_then(|p| Some(p.join(".ctkbiru")))
        .context("Something wrong!")
}

fn is_file_valid(path: &PathBuf) -> bool {
    let is_txt = match path.extension().and_then(OsStr::to_str) {
        Some("txt") => true,
        _ => false,
    };

    if path.is_file() && is_txt {
        true
    } else {
        false
    }

}

fn read_and_write_to_file(source_path: &PathBuf, target_path: &PathBuf) -> Result<()> {
    let source = File::open(&source_path)?; 
    let target = File::create(&target_path)?;

    let mut contents = String::new();
    let mut buf_reader = BufReader::new(source);
    let mut buf_writter = BufWriter::new(target);

    buf_reader.read_to_string(&mut contents)?;
    buf_writter.write_all(contents.as_bytes())?;

    Ok(())
}

#[derive(Debug, PartialEq)]
enum BlueprintType { File, Dir }

#[derive(Debug)]
struct Blueprint {
    name: String,
    depth_level: u8,
    pattern_type: BlueprintType,
}

impl Blueprint {
    pub fn new(pattern: &str) -> Option<Self> {
        let bytes = pattern.as_bytes();

        let mut filtered_bytes: Vec<u8> = vec![];
        let mut depth_level: u8 = 0;
        let mut pattern_type = BlueprintType::File;

        if bytes.len() == 0 || (bytes.len() == 1 && bytes[0] == 47) {
            return None;
        }

        for i in 0..bytes.len() {
            if bytes[i] == (32 as u8) {
                depth_level += 1;
            } else {
                break;
            }
        }
        
        for i in depth_level.into()..bytes.len() {
            if i == (bytes.len() - 1) && bytes[i] == (47 as u8) {
                pattern_type = BlueprintType::Dir;
            } else {
                filtered_bytes.push(bytes[i]);
            }
            
        }

        let name = std::str::from_utf8(&filtered_bytes).ok()?;

        Some(Blueprint {
            name: name.to_string(),
            depth_level,
            pattern_type,
        })
    }
}

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print all availables blueprint
    List(List),

    /// Print blueprint tree structure
    Show(Show),

    /// Add new blueprint
    Add(Add),

    /// Remove blueprint from a list
    Rm(Rm),

    /// Generate directory tree from a blueprint
    Gen(Gen),
}

#[derive(Args)]
struct List;

impl List {
    pub fn result(&self) -> Result<()> {
        let home_path = home_path()?;

        if !home_path.exists() {
            println!("List blueprint:\n\nBlueprint\n----");
        } else {
            println!("List blueprint:\n\nBlueprint\n----");

            if let Ok(entries) = fs::read_dir(&home_path) {
                for entry in entries {
                    let is_exist = if let Some(val) = entry
                        .as_ref()
                        .ok()
                        .and_then(|val| Some(is_file_valid(&val.path()))) 
                    { val } else { false };

                    if is_exist {
                        if let Some(file_stem) = entry
                            .as_ref()
                            .ok()
                            .and_then(|val| {
                                val.path().file_stem()
                                    .and_then(OsStr::to_str)
                                    .map(String::from)
                            })
                        {
                            println!("{file_stem}"); 
                        }
                    }
                }
            }
        }

        Ok(())

    }
}

#[derive(Args)]
struct Show {
    #[clap(value_parser)]
    blueprint: Option<String>,
}

impl Show {
    pub fn result(&self) -> Result<()> {
        if let Some(blueprint) = &self.blueprint { 
            let mut home_path = home_path()?;

            if !home_path.exists() {
                println!("No blueprints found!");
            } else {
                home_path.push(blueprint);
                home_path.set_extension("txt");

                if !is_file_valid(&home_path) {
                    println!("No blueprints found!");
                } else {
                    if let Ok(f) = File::open(&home_path) {
                        println!("Show blueprint:\n\n{}\n----", blueprint);

                        let buf_reader = BufReader::new(f);

                        for line in buf_reader.lines() {
                            let line = line.unwrap_or("".to_string());
                            println!("{line}");
                        }
                    }

                }
            }
        } else {
            println!("Missing <BLUEPRINT> argument!");
        }

        Ok(())
            
    }
}

#[derive(Args)]
struct Add {
    #[clap(value_parser)]
    file: Option<PathBuf>,

    /// Optional blueprint name
    #[clap(short, long, value_parser)]
    name: Option<String>,
}

impl Add {
    pub fn result(&self) -> Result<()> {
        if let Some(input_path) = &self.file {
            if !is_file_valid(input_path) {
                println!("Invalid file format or file doesn't exist!");
            } else {
                let blueprint = input_path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .map(String::from)
                    .context("Something wrong!")?;

                let mut home_path = home_path()?;

                if !home_path.exists() {
                    fs::create_dir(&home_path).context("Something wrong!")?;
                }

                let name = if let Some(val) = &self.name {
                    format!("{val}.txt")
                } else {
                    blueprint
                };

                home_path.push(name);

                if let Ok(_) = read_and_write_to_file(input_path, &home_path) {
                    println!("Blueprint added!");
                } else {
                    println!("Failed to add blueprint!");
                }
            }

        } else {
            println!("Missing <FILE> argument!");
        }

        Ok(())
    }
}

#[derive(Args)]
struct Rm {
    #[clap(value_parser)]
    blueprint: Option<String>,
}

impl Rm {
    pub fn result(&self) -> Result<()> {
        if let Some(blueprint) = &self.blueprint { 
            let mut home_path = home_path()?;

            if !home_path.exists() {
                println!("No blueprints found!");
            } else {
                home_path.push(blueprint);
                home_path.set_extension("txt");

                if !is_file_valid(&home_path) {
                    println!("No blueprints found!");
                } else {
                    if let Ok(_) = fs::remove_file(&home_path) {
                        println!("Blueprint removed!");
                    } else {
                        println!("Failed to remove blueprint!");
                    }
                }
            }
        } else {
            println!("Missing <BLUEPRINT> argument!");
        }

        Ok(())
    }
}

#[derive(Args)]
struct Gen {
    #[clap(value_parser)]
    blueprint: Option<String>,

    /// Optional name for generated 
    /// blueprint
    #[clap(short, long, value_parser)]
    name: Option<String>,

    /// Specify target path, default
    /// current working directory
    #[clap(short, long, value_parser)]
    path: Option<PathBuf>,
}

impl Gen {
    pub fn result(&self) -> Result<()> {
        if let Some(blueprint) = &self.blueprint { 
            let target_path = &mut self.path.as_ref()
                .map_or_else(|| env::current_dir(), |val| Ok(val.to_path_buf()) )
                .context("Something wrong!")?;
            let mut home_path = home_path()?;

            if !home_path.exists() {
                fs::create_dir(&home_path).context("Something wrong!")?;
                println!("No blueprints found!");
            } else {
                home_path.push(blueprint);
                home_path.set_extension("txt");

                if !is_file_valid(&home_path) {
                    println!("No blueprints found!");
                } else {
                    if let Ok(f) = File::open(&home_path) {
                        let name = if let Some(val) = &self.name {
                            val
                        } else {
                            blueprint
                        };

                        target_path.push(name);

                        fs::create_dir(&target_path).context("Something wrong!")?;
                        let buf_reader = BufReader::new(f);

                        match self.generate_blueprint(buf_reader, target_path.to_owned()) {
                            Ok(_) => println!("Finish generate blueprint!"),
                            Err(err) => {
                                fs::remove_dir_all(&target_path)?;
                                return Err(err);
                            },
                        }
                    }
                }
            }
        } else {
            println!("Missing <BLUEPRINT> argument!");
        }

        Ok(())
    }

    fn generate_blueprint(&self, buffer: BufReader<File>, mut path: PathBuf) -> Result<()> {
        let mut depth_level: usize = 0;
        let mut segments: Vec<String> = vec![];

        for line in buffer.lines() {
            let line = line?;

            if let Some(val) = Blueprint::new(&line) {

                if depth_level == val.depth_level.into() {
                    if segments.len() == 0 {
                        segments.push(val.name.to_string());
                    } else {
                        segments[depth_level] = val.name.to_string();
                    }
                } else if depth_level < val.depth_level.into() {
                    segments.push(val.name.to_string());
                    depth_level += 1;
                } else {
                    while depth_level > val.depth_level.into() {
                        segments.pop();
                        depth_level -= 1;
                    }
                    segments[depth_level] = val.name.to_string();
                }

                path.push(segments.join("/"));

                if val.pattern_type == BlueprintType::Dir {
                    fs::create_dir(&path)?;
                } else {
                    File::create(&path)?;
                }

                for _ in 0..segments.len() {
                    path.pop();
                }

            }

        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List(val) => val.result()?,
        Commands::Show(val) => val.result()?,
        Commands::Add(val) => val.result()?,
        Commands::Rm(val) => val.result()?,
        Commands::Gen(val) => val.result()?,
    }

    Ok(())
}
