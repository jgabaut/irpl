use easy_repl::{Repl, CommandStatus, command};
use std::path::PathBuf;
use std::fs;
use std::env;
use regex::Regex;

const IRPL_VERS: &'static str = "0.1.1";

fn find_file_size(file: &str) -> u64 {
    fs::metadata(file).unwrap().len()
}

fn check_is_file_or_dir(arg: &str) -> &str {
    let file = "File";
    let dir = "Directory";
    let re = Regex::new(r"/").unwrap();
    if re.is_match(arg) {
        return dir;
    }
    return file;
}

fn collect_user_arguments() -> Vec<String> {
    env::args().collect()
}

fn check_args_count(args: &Vec<String>) -> bool {
    if args.len() == 2 {
        return true
    }
    help();
    return false
}

fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}

fn help() {
    println!("irpl v{}",IRPL_VERS);
    println!("Usage: irpl <arg>\n");
}

fn main() {

    let working_path = get_current_working_dir();
    println!("Work path is: [{}]", working_path.expect("I guess a program can have no working path?").display());

	let mut repl = Repl::builder()
	    .add("echo", command! {
		"Echoes back",
		(name: String) => |name| {
		    println!("{}", name);
		    Ok(CommandStatus::Done)
		}
	    })
	    .add("add", command! {
		"Add X to Y",
		(X:i32, Y:i32) => |x, y| {
		    println!("{} + {} = {}", x, y, x + y);
		    Ok(CommandStatus::Done)
		}
	    })
	    .add("sub", command! {
		"Sub X from Y",
		(X:i32, Y:i32) => |x, y| {
		    println!("{} - {} = {}", x, y, x - y);
		    Ok(CommandStatus::Done)
		}
	    })
            .add("version", command! {
                "Display current irpl version",
                () => | | {
                    println!("irpl v{}",IRPL_VERS);
		    Ok(CommandStatus::Done)
		}
            })
	    .build().expect("Failed to create repl");

    	let args: Vec<String> = collect_user_arguments();

    	if check_args_count(&args) {
        	let arg1 = &args[1];
        	//let arg2 = &args[2];
        	println!("Arg1 is a: {:#?}", check_is_file_or_dir(&arg1));
		println!("Arg1: [{}]",&arg1);
		repl.run().expect("Critical REPL error");
		return;
    	} else {
        	println!("Quitting.");
		return;
	}

}
