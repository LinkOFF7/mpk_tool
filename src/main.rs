use std::{env, process};

mod mpk;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Emio – The Smiling Man: Famicom Detective Club MPK Extractor\n");
        println!("Usage: {} INPUT_MPK", &args[0]);
        process::exit(1);
    } else {
        let _ = mpk::extract(&args[1]);
    }
}