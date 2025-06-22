use std::env::current_exe;
use std::fs::create_dir_all;
use std::path::Component::ParentDir;
use std::path::PathBuf;
use clap::{Arg, ArgAction, ArgMatches, Command};
use clap::builder::PathBufValueParser;
use log::error;

pub fn command() -> Command {
    Command::new("split")
        .about("Splits a single video into multiple clips of a specified length")
        .arg(Arg::new("length")
            .long("length")
            .short('l')
            .help("The desired length of each new clip")
            .value_name("LENGTH")
            .value_parser(clap::value_parser!(u32))
            .required(true)
        )
        .arg(Arg::new("output_directory")
            .long("output")
            .short('o')
            .help("The directory to save the split clips")
            .value_name("OUTPUT")
            .value_parser(clap::value_parser!(PathBuf))
            .required(true)
        )
        .arg(Arg::new("reencode")
            .long("reencode")
            .short('r')
            .help("When true, ffmpeg will reencode each clip - this may help with periods of missing video")
            .action(ArgAction::SetTrue)
            .required(false)
        )
        .arg(Arg::new("input_file")
            .help("The input video to be split")
            .value_name("INPUT")
            .value_parser(clap::value_parser!(PathBuf))
            .required(true)
        )
}

pub fn run(matches: &ArgMatches) {
    let length = matches.get_one::<u32>("length").expect("required");
    let out_dir: &PathBuf = matches.get_one::<PathBuf>("output_directory").expect("required");
    let input_file = matches.get_one::<PathBuf>("input_file").expect("required");
    let reencode = matches.get_flag("reencode");

    // Create out dir if not exists
    if (!out_dir.exists()) {
        match create_dir_all(out_dir) {
            Err(_) => {
                println!("ERROR: Failed to create output directory");
                eprintln!("Failed to create output directory");
            }
            _ => {}
        }
    }

    // Verify input file
    if (!input_file.exists()) {
        error!("Input file does not exist");
    }

    // Probe file
    let mut duration: f64 = 0.0;
    match ffprobe::ffprobe(input_file) {
        Ok(info) => {
            if (info.streams.len() < 1) {
                error!("Zero streams found in file");
                std::process::exit(1);
            }
            duration = match &info.streams[0].duration {
                Some(x) => {
                    println!("Num: {x}");
                    x.parse::<f64>().unwrap()
                },
                None => 0.0
            };
        }
        Err(_) => {
            println!("ERROR: Failed to probe input file");
            eprintln!("Failed to probe input file");
        }
    }

    // Perform splitting
    let num_clips = (duration / (*length as f64)).ceil();
    let mut current_time: f64 = 0.0;
    let mut clip_number: u16 = 1;
    while (current_time < duration) {
        println!("Processing clip {} of {}", clip_number, num_clips);
        let clip_start = current_time;
        let mut clip_end = current_time + (*length as f64);
        if (clip_end > duration) {
            clip_end = duration;
        }

        let mut new_path = out_dir.clone();
        new_path.push(format!("{}_{current_time:.2}_{clip_end:.2}.mp4", input_file.file_stem().unwrap().to_str().unwrap()));
        if (!reencode) {
            let cmd = std::process::Command::new("ffmpeg")
                .args([
                    "-i",
                    input_file.to_str().unwrap(),
                    "-ss",
                    clip_start.to_string().as_str(),
                    "-to",
                    clip_end.to_string().as_str(),
                    "-c",
                    "copy",
                    new_path.to_str().unwrap()
                ])
                .spawn()
                .expect("Failed to launch ffmpeg")
                .wait()
                .expect("ffmpeg failed for an unknown reason");
        } else {
            let cmd = std::process::Command::new("ffmpeg")
                .args([
                    "-i",
                    input_file.to_str().unwrap(),
                    "-ss",
                    clip_start.to_string().as_str(),
                    "-to",
                    clip_end.to_string().as_str(),
                    new_path.to_str().unwrap()
                ])
                .spawn()
                .expect("Failed to launch ffmpeg")
                .wait()
                .expect("ffmpeg failed for an unknown reason");
        }
        println!("{}", new_path.to_str().unwrap());
        current_time += (*length as f64);
        clip_number += 1;
    }
}