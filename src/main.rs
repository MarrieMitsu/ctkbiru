use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand};

extern crate dirs;

fn home_path() -> Result<PathBuf> {
    dirs::home_dir()
        .and_then(|p| Some(p.join(".ctkbiru")))
        .context("Something went wrong, user's home directory does not exist!")
}

fn is_file_valid(path: &PathBuf) -> bool {
    let is_txt = match path.extension().and_then(OsStr::to_str) {
        Some("txt") => true,
        _ => false,
    };

    path.is_file() && is_txt
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

fn char_to_bytes(c: char) -> Vec<u8> {
    let mut buffer = [0; 4];
    let encoded = c.encode_utf8(&mut buffer);
    encoded.as_bytes().to_vec()
}

/// BlueprintType
///
/// Represent type of each line inside the blueprint file. `/` character at the end
/// of the line indicates a directory
#[derive(Debug, Eq, PartialEq)]
enum BlueprintType {
    File,
    Dir,
}

/// Blueprint
///
/// Represent each line inside the blueprint file
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

        // check if line empty or only contain `/` character
        if bytes.len() == 0 || (bytes.len() == 1 && bytes[0] == 47) {
            return None;
        }

        // count depth level
        for c in pattern.chars() {
            let b = char_to_bytes(c);

            if b.len() == 1 && b[0] == (32 as u8) {
                depth_level += 1;
            } else {
                break;
            }
        }

        // determine blueprint type and construct blueprint name
        let count = pattern.chars().count();
        for (i, c) in pattern.chars().enumerate().skip(depth_level as usize) {
            let b = char_to_bytes(c);

            if i == count - 1 && (b.len() == 1 && b[0] == (47 as u8)) {
                pattern_type = BlueprintType::Dir;
            } else {
                filtered_bytes.extend(b);
            }
        }

        let name = String::from_utf8(filtered_bytes).ok()?;

        Some(Self {
            name,
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
    pub fn exec(&self) -> Result<()> {
        let home_path = home_path()?;

        println!("List blueprint:\n\nBlueprint\n----");

        if home_path.exists() {
            let entries = fs::read_dir(&home_path)?;

            for entry in entries {
                let p = entry?.path();
                let is_valid = is_file_valid(&p);

                // this check make sure file_stem never failed
                if is_valid {
                    let file_stem = p.file_stem().unwrap().to_str().unwrap();
                    println!("{file_stem}");
                }
            }
        }

        Ok(())
    }
}

#[derive(Args)]
struct Show {
    /// Blueprint name
    blueprint: String,
}

impl Show {
    pub fn exec(&self) -> Result<()> {
        let mut home_path = home_path()?;

        if !home_path.exists() {
            println!("No blueprints found!");
            return Ok(());
        }

        home_path.push(&self.blueprint);
        home_path.set_extension("txt");

        if !is_file_valid(&home_path) {
            println!("No blueprints found!");
            return Ok(());
        }

        let f = File::open(&home_path).context("Cannot open blueprint!")?;
        println!("Show blueprint:\n\n{}\n----", &self.blueprint);

        let buf_reader = BufReader::new(f);

        for line in buf_reader.lines() {
            let line = line.unwrap_or("".to_string());
            println!("{line}");
        }

        Ok(())
    }
}

#[derive(Args)]
struct Add {
    /// Blueprint file path
    file: PathBuf,

    /// Optional blueprint name
    #[clap(short, long)]
    name: Option<String>,
}

impl Add {
    pub fn exec(&self) -> Result<()> {
        if !is_file_valid(&self.file) {
            bail!("Invalid file format or file doesn't exist!");
        }

        let mut home_path = home_path()?;

        if !home_path.exists() {
            fs::create_dir(&home_path)
                .context("Something went wrong, cannot create home directory!")?;
        }

        let name = if let Some(v) = &self.name {
            format!("{v}.txt")
        } else {
            // filename cannot failed because already guarded by conditional check
            self.file.file_name().unwrap().to_str().unwrap().to_string()
        };

        home_path.push(name);

        match read_and_write_to_file(&self.file, &home_path) {
            Ok(_) => {
                println!("Blueprint added!");
            }
            Err(e) => {
                bail!("Failed to add blueprint: {:?}", e);
            }
        }

        Ok(())
    }
}

#[derive(Args)]
struct Rm {
    /// Blueprint name
    blueprint: String,
}

impl Rm {
    pub fn exec(&self) -> Result<()> {
        let mut home_path = home_path()?;

        if !home_path.exists() {
            println!("No blueprints found!");
            return Ok(());
        }

        home_path.push(&self.blueprint);
        home_path.set_extension("txt");

        if !is_file_valid(&home_path) {
            println!("No blueprints found!");
        } else {
            match fs::remove_file(&home_path) {
                Ok(_) => println!("Blueprint removed!"),
                Err(e) => bail!("Failed to remove blueprint: {:?}", e),
            }
        }

        Ok(())
    }
}

#[derive(Args)]
struct Gen {
    /// Blueprint name
    blueprint: String,

    /// Optional name for parent directory of generated
    /// blueprint. By default it uses current working directory
    /// or target path by checking if it's empty
    #[clap(short, long)]
    name: Option<String>,

    /// Specify target path, default current working
    /// directory
    #[clap(short, long)]
    path: Option<PathBuf>,
}

impl Gen {
    pub fn exec(&self) -> Result<()> {
        let target_path = &mut self
            .path
            .as_ref()
            .map_or_else(|| env::current_dir(), |v| Ok(v.to_path_buf()))
            .context("Something went wrong!")?;
        let mut home_path = home_path()?;

        if !home_path.exists() {
            fs::create_dir(&home_path)
                .context("Something went wrong, cannot create home directory!")?;
            println!("No blueprints found!");
            return Ok(());
        }

        home_path.push(&self.blueprint);
        home_path.set_extension("txt");

        if !is_file_valid(&home_path) {
            println!("No blueprints found!");
            return Ok(());
        }

        let f = File::open(&home_path).context("Cannot open blueprint!")?;

        let is_name_empty = if let Some(val) = &self.name {
            target_path.push(val);
            fs::create_dir(&target_path).context("Something went wrong!")?;
            false
        } else {
            true
        };

        let is_target_path_empty = target_path.read_dir()?.next().is_none();

        if is_name_empty && !is_target_path_empty {
            println!("Directory must be empty!");
            return Ok(());
        }

        let buf_reader = BufReader::new(f);

        match self.generate_blueprint(buf_reader, target_path.to_owned()) {
            Ok(_) => println!("Finish generate blueprint!"),
            Err(err) => {
                fs::remove_dir_all(&target_path)?;
                bail!("Failed to generate blueprint: {:?}", err);
            }
        }

        Ok(())
    }

    fn generate_blueprint(&self, buffer: BufReader<File>, mut path: PathBuf) -> Result<()> {
        let mut depth_level: usize = 0;
        let mut segments: Vec<String> = vec![];

        for line in buffer.lines() {
            let line = line?;
            let blueprint = Blueprint::new(&line).ok_or(anyhow!("Invalid blueprint bytes data"))?;

            if depth_level == blueprint.depth_level.into() {
                if segments.len() == 0 {
                    segments.push(blueprint.name.to_string());
                } else {
                    segments[depth_level] = blueprint.name.to_string();
                }
            } else if depth_level < blueprint.depth_level.into() {
                segments.push(blueprint.name.to_string());
                depth_level += 1;
            } else {
                while depth_level > blueprint.depth_level.into() {
                    segments.pop();
                    depth_level -= 1;
                }
                segments[depth_level] = blueprint.name.to_string();
            }

            path.push(segments.join("/"));

            if blueprint.pattern_type == BlueprintType::Dir {
                fs::create_dir(&path).context("Cannot create blueprint directory")?;
            } else {
                File::create(&path).context("Cannot create blueprint file")?;
            }

            for _ in 0..segments.len() {
                path.pop();
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List(val) => val.exec()?,
        Commands::Show(val) => val.exec()?,
        Commands::Add(val) => val.exec()?,
        Commands::Rm(val) => val.exec()?,
        Commands::Gen(val) => val.exec()?,
    }

    Ok(())
}
