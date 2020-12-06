use std::{env, process};

fn main() {
    let status = match distinst_cli::cli(&mut env::args_os()) {
        Ok(()) => {
            println!("install was successful");
            0
        }
        Err(err) => {
            println!("install failed: {}", err);
            1
        }
    };
    process::exit(status);
}
