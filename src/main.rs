use clap::Parser;
use std::fs;
use rand::prelude::*;

/// Rust program that splits a directory full of files into train test and validation subdirs
/// randomly.

#[derive(Parser, Debug)]

#[command(version, about, long_about = None)]
struct Cli {
    /// Input directory
    input_dir: String,

    /// Output directory, if not provided operation will happen in place
    output_dir: Option<String>,

    /// Test dir size, if not provided test dir will be 20% of input dir
    #[arg(short, long)]
    test: Option<u32>,

    /// Validation dir size, if not provided val dir will not be created
    #[arg(short, long)]
    val: Option<u32>,
}

fn main() {
    let cli = Cli::parse();
    let in_place: bool;
    let entries = fs::read_dir(&cli.input_dir).unwrap();

    let root: String = match cli.output_dir {
        Some(output_dir) => {
            in_place = false;

            fs::create_dir_all(&output_dir).unwrap();
            output_dir
        },
        None => {
            in_place = true;
            cli.input_dir
        },
    };
    
    let mut files: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            files.push(path.to_str().unwrap().to_string());
        }
    }

    let train_dir:String = format!("{}/train", root);
    let test_dir: String = format!("{}/test", root);
    fs::create_dir_all(&train_dir).unwrap();
    fs::create_dir_all(&test_dir).unwrap();

    let val_dir: Option<String> = if cli.val.is_some() {
        fs::create_dir_all(&format!("{}/val", root)).unwrap();
        Some(format!("{}/val", root))
    } else {
        None
    };

    let test_size: u32;
    match cli.test {
        Some(test) => test_size = test,
        None => test_size = (files.len() as f32 * 0.2) as u32,
    }

    let val_size: u32;
    match cli.val {
        Some(val) => val_size = val,
        None => val_size = 0,
    }

    let mut rng = rand::thread_rng();
    files.shuffle(&mut rng);

    let test_files = &files[0..test_size as usize];

    for file in test_files {
        let file_name = file.split("/").last().unwrap();
        let dest = format!("{}/{}", test_dir, file_name);
        if in_place {
            fs::rename(file, dest).unwrap();
        } else {
            fs::copy(file, dest).unwrap();
        }
    }

    if val_dir.is_some() {
        let val_files = &files[test_size as usize..(test_size + val_size) as usize];
        for file in val_files {
            let file_name = file.split("/").last().unwrap();
            let dest = format!("{}/{}", val_dir.as_ref().unwrap(), file_name);
            if in_place {
                fs::rename(file, dest).unwrap();
            } else {
                fs::copy(file, dest).unwrap();
            }
        }
    }

    let train_files = &files[(test_size + val_size) as usize..];
    for file in train_files {
        let file_name = file.split("/").last().unwrap();
        let dest = format!("{}/{}", train_dir, file_name);
        if in_place {
            fs::rename(file, dest).unwrap();
        } else {
            fs::copy(file, dest).unwrap();
        }
    }
}
