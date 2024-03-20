use clap::Parser;
use std::{fs, collections::HashMap};
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

    /// DatasetDirectory: If the directory is a pytorch dataset directory
    #[arg(short, long, default_value = "false")]
    dataset_directory: bool,
}


fn main() {
    let cli = Cli::parse();
    let in_place: bool;
    let mut seen: HashMap<&String, u32> = HashMap::new();
    let mut files: Vec<String> = Vec::new();
    
    if cli.dataset_directory {
        for dir_entry in fs::read_dir(&cli.input_dir).unwrap() {
            let dir_path = dir_entry.unwrap().path();
            if dir_path.is_file() {
                eprintln!("Error: Root directory should not contain files in DATASET_DIRECTORY mode");
            }

            for entry in fs::read_dir(dir_path).unwrap() {
                let file_path = entry.unwrap().path();
                if file_path.is_file() {
                    files.push(file_path.to_str().unwrap().to_string());
                } else {
                    eprintln!("Error: Subdirectories should only contain files in DATASET_DIRECTORY mode");
                }
            }
        }
    } else {
        for entry in fs::read_dir(&cli.input_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                files.push(path.to_str().unwrap().to_string());
            } else {
                eprintln!("Error: Root should only contain files not directories");
            }
        }
    };

    // shuffle the files
    files.shuffle(&mut thread_rng());

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
    create_files(test_files, &test_dir, &mut seen, in_place, cli.dataset_directory);

    if val_dir.is_some() {
        let val_files = &files[test_size as usize..(test_size + val_size) as usize];
        create_files(val_files, &val_dir.unwrap(), &mut seen, in_place, cli.dataset_directory);
    }

    let train_files = &files[(test_size + val_size) as usize..];
    create_files(train_files, &train_dir, &mut seen, in_place, cli.dataset_directory);

    if cli.dataset_directory {
        for entry in fs::read_dir(&root).unwrap() {
            let dir_path = entry.unwrap().path();
            if dir_path.file_name().unwrap() != "train" && dir_path.file_name().unwrap() != "test" && dir_path.file_name().unwrap() != "val" {
                fs::remove_dir_all(dir_path).unwrap();
            }
        }
    }
}

fn create_files<'a>(files: &'a [String], out_dir: &str, seen: &mut HashMap<&'a String, u32>, rename: bool, dataset_dir: bool) {
    for file in files {
        let file_name = file.split("/").last().unwrap();
        let mut dest: String;

        if dataset_dir {
            let class = file.split("/").nth(1).unwrap();
            let class_dir = format!("{}/{}", out_dir, class);
            fs::create_dir_all(&class_dir).unwrap();
            dest = format!("{}/{}", class_dir, file_name);
        } else {
            dest = format!("{}/{}", out_dir, file_name);
        }

        if seen.contains_key(&file) {
            let count = seen.get(&file).unwrap();
            let has_extension = file_name.contains(".");
            if has_extension {
                let splits = file_name.split(".").collect::<Vec<&str>>();
                let extension = splits.last().unwrap();
                let rest = splits[0..splits.len() - 1].join(".");
                dest = format!("{}/{}-{}.{}", out_dir, rest, count, extension);
            } else {
                dest = format!("{}/{}-{}", out_dir, file_name, count);
            }
            seen.insert(&file, count + 1);
        } else {
            seen.insert(&file, 1);
        }

        if rename {
            fs::rename(file, dest).unwrap();
        } else {
            fs::copy(file, dest).unwrap();
        }
    }
}

