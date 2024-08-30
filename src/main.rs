use std::{env, process};

mod mpk;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        print_usage(&args[0]);
        process::exit(1);
    } else {
        if &args[1] == "extract" {
            let _ = match mpk::extract(&args[2]) {
                Err(e) => panic!("{}", e),
                Ok(r) => r
            };
        } else if &args[1] == "build" {
            let _ = match mpk::build(&args[2]) {
                Err(e) => panic!("{}", e),
                Ok(r) => r
            };
        } else {
            print_usage(&args[0]);
            process::exit(1);
        }
    }
}

fn print_usage(arg: &str) {
    println!("Emio – The Smiling Man: Famicom Detective Club MPK Extractor\n");
    println!("Usage: {} <mode> <mpk/directory>\n", &arg);
    println!("MODES:\n  extract\tExtract all files from archive\n  build\t\tCreate new archive from folder");
}