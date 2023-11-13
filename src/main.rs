use easy_repl::{Repl, CommandStatus, Critical, command};
use std::path::PathBuf;
use std::net::IpAddr;
use std::fs;
use std::env;
use clap::Parser;
use std::process;
use std::thread;
use std::time;
use regex::Regex;
use anyhow::{self, Context};
use std::time::Instant;
use std::time::SystemTime;
use clearscreen::ClearScreen;
use std::collections::HashMap;
use rand::Rng;
use chrono::Local;
use csurename::RunConfig;
const IRPL_VERS: &'static str = env!("CARGO_PKG_VERSION");

fn may_throw(description: String) -> Result<(), std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, description))
}

fn try_csurename(description: String) -> Result<(), std::io::Error> {
    #[derive(Parser, Debug)]
    #[command(
    author,
    version,
    about,
    long_about = None,
    after_help = "Full documentation available here: https://github.com/csunibo/csurename"
    )]
    pub struct Args {
        /// Specifies a target directory, working dir if none
        target_dir: Option<String>,

        /// Makes renaming recursive, renaming files in subfolders as well
        #[arg(short, long)]
        recursive: bool,

        /// Renames directories as well
        #[arg(short = 'D', long = "dir")]
        include_dir: bool,

        /// Suppress output
        #[arg(short, long)]
        quiet: bool,

        /// Reads lines from stdin and translates them to the given convention in stdout until the first empty line
        #[arg(short = 'T', long)]
        text: bool,
    }
    let config = Args::parse();
    let target_dir = config
        .target_dir
        .map_or_else(env::current_dir, |p| Ok(PathBuf::from(p)))
        .expect(
            "Could not read target directory. Please make sure you have the right permissions.",
        );
    let run_config = RunConfig {
        target_dir: target_dir,
        recursive: false,
        include_dir: false,
        quiet: false,
        from_stdin: true,
    };

    if let Err(e) = csurename::run(run_config) {
        eprintln!("[csurename]: {description}: {e}");
        process::exit(1);
    } else {
        //We ended the csurename::run() call successfully
        Ok(())
    }
}

fn collect_user_arguments() -> Vec<String> {
    env::args().collect()
}

fn check_args_count(args: &Vec<String>) -> bool {
    if args.len() == 1 {
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

fn build_irpl(name: String, load_symbols: &HashMap<String,String>) -> anyhow::Result<Repl> {
    let irpl_start = Instant::now();
    let irpl_start_secs = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
    //let irpl_date = Local::now();
    //let irpl_date_formatted = format!("{}", irpl_date.format("%Y-%m-%d %H:%M:%S"));
    let mut irpl_symbols = HashMap::new();
    // Iterate over load_symbols and copy them
    for (k, v) in load_symbols {
        let k_fmt = format!("{}", k.to_string());
        let v_fmt = format!("{}", v.to_string());
        irpl_symbols.insert(k_fmt.to_string(),v_fmt.to_string());
    }
    irpl_symbols.insert(
        "irpl_start_secs".to_string(),
        irpl_start_secs.to_string()
    );
    let mut outside_x = String::from("Out x");
    //let mut outside_y = String::from("Out y");
    let prompt = format!("[{}]> ", name);
    let cloned_prompt = prompt.clone();  // need to move it into closure

    let new = command! {
        "Enter new repl",
        (name:String) => |name: String| {
            let name = cloned_prompt.clone() + &name;
            let mut repl = build_irpl(name,load_symbols)?;
            println!("irpl - started at {:?}",irpl_start);
            repl.run()?;
            Ok(CommandStatus::Done)
        }
    };

    let cloned_prompt = prompt.clone();  // need to move it into closure
    let repl = Repl::builder()
	    .prompt(cloned_prompt)
	    .with_hints(true)
	    .add("new", new)
	    .add("echo", command! {
		    "Echoes back",
		    (name: String) => |name| {
			println!("{}", name);
			Ok(CommandStatus::Done)
		    }
	    })
	    .add("date", command! {
		    "Echoes current date and time",
        () => | | {
            let curr_date = Local::now();
                println!("{}", curr_date.format("%Y-%m-%d %H:%M:%S"));
                Ok(CommandStatus::Done)
            }
	    })
        .add("time", command! {
          "Echoes current time",
          () => | | {
              let curr_date = Local::now();
                  println!("{}", curr_date.format("%H:%M:%S"));
                  Ok(CommandStatus::Done)
              }
        })
	    .add("unixtime", command! {
		    "Echoes elapsed seconds since UNIX epoch",
		    () => | | {
            let secs = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => n.as_secs(),
                Err(_) => panic!("SystemTime before UNIX EPOCH!"),
            };
            println!("{}", secs);
			Ok(CommandStatus::Done)
		    }
	    })
	    .add("rand", command! {
		    "Echoes a random num between the two passed values",
		    (min: f64, max: f64) => |min: f64, max: f64 | {
            let mut rng = rand::thread_rng();
            let mut r: f64 = rng.gen_range(min..max); // generates a float
            println!("{}", r);
			Ok(CommandStatus::Done)
		    }
	    })
	    .add("bc", command! {
		    "Basic calculator",
		    (expr: String) => |expr: String | {
            match meval::eval_str(expr.to_string()) {
                Ok(i) => {
                    println!("{} == {}", expr, i);
                },
                Err(e) => {
                    eprintln!("bc: {e}");
                }
            };
            Ok(CommandStatus::Done)
            }
	    })
	    .add("test[-f]", command! {
		    "Test if arg is file or dir",
		    (arg: PathBuf) => |arg: PathBuf| {
		        let file = "File";
		        let dir = "Directory";
			    let filepath = format!("{}", arg.as_path().to_string_lossy());
		        let re = Regex::new(r"/").unwrap();
		        if re.is_match(&filepath) {
			        println!("{} is a {}", filepath, file);
		        } else {
			        println!("{} is a {}", filepath, dir);
		        }
			    Ok(CommandStatus::Done)
            }
	    })
        .add("du", command! {
                "Shows file size",
                (arg: PathBuf) => |arg: PathBuf| {
                let filepath = format!("{}", arg.as_path().to_string_lossy());

                let re = Regex::new(r"/").unwrap();
                match fs::metadata(&filepath) {
                    Ok(i) => {
                        let filesize = i.len();
                        if re.is_match(&filepath) {
                                  //arg is a file
                            println!("Size for {} is {}", filepath, filesize);
                        } else {
                                  //arg is a dir
                            println!("Size for {} is {}", filepath, filesize);
                        }
                    },
                    Err(e) => {
                        eprintln!("du: {}", e);
                    }
                };
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
	    .add("ok", command! {
		"Run a command that just succeeds",
		() => || Ok(CommandStatus::Done)
	    })
	    .add("error", command! {
		"Command with recoverable error handled by the REPL",
		(text:String) => |text| {
		    may_throw(text)?;
		    Ok(CommandStatus::Done)
		},
	    })
	    .add("critical", command! {
		"Command returns a critical error that must be handled outside of REPL",
		(text:String) => |text| {
		    // Short notation using the Critical trait
		    may_throw(text).into_critical()?;
		    // More explicitly it could be:
		    //   if let Err(err) = may_throw(text) {
		    //       Err(easy_repl::CriticalError::Critical(err.into()))?;
		    //   }
		    // or even:
		    //   if let Err(err) = may_throw(text) {
		    //       return Err(easy_repl::CriticalError::Critical(err.into())).into();
		    //   }
		    Ok(CommandStatus::Done)
		},
	    })
	    .add("roulette", command! {
		"Feeling lucky?",
		() => || {
		    let ns = Instant::now().duration_since(irpl_start).as_nanos();
		    let cylinder = ns % 6;
		    match cylinder {
			0 => may_throw("Bang!".into()).into_critical()?,
			1..=2 => may_throw("Blank cartridge?".into())?,
			_ => (),
		    }
		    Ok(CommandStatus::Done)
		},
	    })
	    .add("csurename", command! {
		"Convert lines to kebab case",
		() => || {
            let one_sec = time::Duration::from_millis(1000);
            println!("\nUsing csurename v1.3.0");
            println!("\n\n    origin at:     git@github.com/csunibo/csurename.git\n");
            println!("    Parse input according to csunibo org rules for filenames.\n");
            println!("    Enter empty line to quit.\n");
            let now = time::Instant::now();
            thread::sleep(one_sec);
            assert!(now.elapsed() >= one_sec);

            //May fail or cause damage, from what I saw... read-only filesystem made me trip
            try_csurename("Error on csurename command.".into()).into_critical()?;

		    Ok(CommandStatus::Done)
		},
	    })
        .add("memdump", command! {
		    "Display irpl_symbols",
		    () => | | {
            for (symbol, value) in &irpl_symbols {
                println!("{symbol}: \"{value}\"");
            }
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
	    .add("ls", command! {
		"List files in a directory",
		(dir: PathBuf) => |dir: PathBuf| {
		    for entry in dir.read_dir()? {
			println!("{}", entry?.path().to_string_lossy());
		    }
		    Ok(CommandStatus::Done)
		}
	    })
	    .add("ipaddr", command! {
		"Just parse and print the given IP address",
		(ip: IpAddr) => |ip: IpAddr| {
		    println!("{}", ip);
		    Ok(CommandStatus::Done)
		}
	    })
            .add("count", command! {
		"Count from X to Y",
		(X:i32, Y:i32) => |x, y| {
		    for i in x..=y {
			print!(" {}", i);
		    }
		    println!();
		    Ok(CommandStatus::Done)
		}
	    })
            .add("clear", command! {
                "Clear the screen",
                () => | | {
                    ClearScreen::default().clear().expect("failed to clear the screen");
		    Ok(CommandStatus::Done)
		}
            })
	    .add("say", command! {
		"Say X",
		(:f32) => |x| {
		    println!("x is equal to {}", x);
		    Ok(CommandStatus::Done)
		},
	    })
	    .add("outx", command! {
		"Use mutably outside var x. This command has a really long description so we need to wrap it somehow, it is interesting how actually the wrapping will be performed.",
		() => || {
		    outside_x += "x";
		    println!("{}", outside_x);
		    Ok(CommandStatus::Done)
		},
	    })
            /*
	    // this shows how to create Command manually with the help of the validator! macro
	    // one could also implement arguments validation manually
	    .add("outy", easy_repl::Command {
		description: "Use mutably outside var y".into(),
		args_info: vec!["appended".into()],
		handler: Box::new(|args| {
		    let validator = validator!(i32);
		    validator(args)?;
		    outside_y += args[0];
		    println!("{}", outside_y);
		    Ok(CommandStatus::Done)
		}),
	    })
            */
	    .build()?;

	Ok(repl)
}
/*
// Iterate over all symbols and print them.
for (symbol, value) in &irpl_symbols {
    println!("{symbol}: \"{value}\"");
}
*/


fn main() -> anyhow::Result<()>  {
    let mut main_irpl_symbols = HashMap::<String,String>::new();
    let main_start_secs = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
    //let main_date = Local::now();
    //let main_date_formatted = format!("{}", main_date.format("%Y-%m-%d %H:%M:%S"));

    main_irpl_symbols.insert(
        "irpl_vers".to_string(),
        IRPL_VERS.to_string()
    );

    main_irpl_symbols.insert(
        "main_start_secs".to_string(),
        main_start_secs.to_string()
    );

    //let mut outside_y = String::from("Out y");
    let mut working_path = get_current_working_dir();
    println!("Work path is: [{}]", working_path.as_mut().expect("I guess a program can have no working path?").display());
    main_irpl_symbols.insert(
        "main_workpath".to_string(),
        working_path.as_mut().expect("I guess a program can have no working path?").display().to_string()
    );

    let prompt = format!("irpl ");

    //let mut repl = matryoshka("".into())?;
	//let mut repl = matryoshka(prompt.into())?;

    let args: Vec<String> = collect_user_arguments();

    let mut args_num = 0;
    for arg in &args {
        main_irpl_symbols.insert(
            (format!("main_arg{}", args_num)).to_string(),
            arg.to_string()
        );
        args_num += 1 ;
    }
    let mut repl = build_irpl(prompt, &main_irpl_symbols)?;

    if check_args_count(&args) {
       	//let arg2 = &args[2];
       	//println!("Arg1 is a: {:#?}", check_is_file_or_dir(&arg1));
		repl.run().context("Critical REPL error")?;
		Ok(())
    	} else {
        	println!("Wrong number of args: {}, expected {}. Quitting.",args.len(),1);
		//TODO: make this thing fail
		Ok(())
	}

}
