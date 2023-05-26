use easy_repl::{Repl, CommandStatus, Critical, validator, command};
use std::path::PathBuf;
use std::net::IpAddr;
use std::fs;
use std::env;
use regex::Regex;
use anyhow::{self, Context};
use std::time::Instant;
use clearscreen::ClearScreen;

const IRPL_VERS: &'static str = "0.1.4";

fn may_throw(description: String) -> Result<(), std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, description))
}

fn matryoshka(name: String) -> anyhow::Result<Repl<'static>> {
    let start = Instant::now();
    let prompt = format!("irpl [{}> ", name);
    let mut outside_x = String::from("Out x");

    let cloned_prompt = prompt.clone();  // need to move it into closure
    let new = command! {
        "Enter new repl",
        (name:String) => |name: String| {
            let name = cloned_prompt.clone() + &name;
            let mut repl = matryoshka(name)?;
            repl.run()?;
            Ok(CommandStatus::Done)
        }
    };

    let repl = Repl::builder()
	.prompt(prompt)
	.add("new", new)
	.add("echo", command! {
		"Echoes back",
		(name: String) => |name| {
		    println!("{}", name);
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
		      println!("{} is a {}", filepath, dir);
		    } else {
		      println!("{} is a {}", filepath, file);
		    }
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
		let ns = Instant::now().duration_since(start).as_nanos();
		let cylinder = ns % 6;
		match cylinder {
		    0 => may_throw("Bang!".into()).into_critical()?,
		    1..=2 => may_throw("Blank cartridge?".into())?,
		    _ => (),
		}
		Ok(CommandStatus::Done)
	    },
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
	.build()?;

    Ok(repl)
}

fn find_file_size(file: &str) -> u64 {
    fs::metadata(file).unwrap().len()
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

fn main() -> anyhow::Result<()>  {
    let start = Instant::now();

    let mut outside_x = String::from("Out x");
    let mut outside_y = String::from("Out y");
    let working_path = get_current_working_dir();
    println!("Work path is: [{}]", working_path.expect("I guess a program can have no working path?").display());

    	let prompt = format!("irpl]>");
	let cloned_prompt = prompt.clone();  // need to move it into closure
	let new_repl = command! {
	    "Enter new repl",
	    (name:String) => |name: String| {
		let name = cloned_prompt.clone() + &name;
		let mut repl = matryoshka(name)?;
		repl.run()?;
		Ok(CommandStatus::Done)
	    }
	};

    	//let mut repl = matryoshka("".into())?;
	//let mut repl = matryoshka(prompt.into())?;
	let mut repl = Repl::builder()
	    .prompt(prompt)
	    .add("new", new_repl)
	    .add("echo", command! {
		    "Echoes back",
		    (name: String) => |name| {
			println!("{}", name);
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
		    let ns = Instant::now().duration_since(start).as_nanos();
		    let cylinder = ns % 6;
		    match cylinder {
			0 => may_throw("Bang!".into()).into_critical()?,
			1..=2 => may_throw("Blank cartridge?".into())?,
			_ => (),
		    }
		    Ok(CommandStatus::Done)
		},
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
	    .build()?;

    	let args: Vec<String> = collect_user_arguments();

    	if check_args_count(&args) {
        	let arg0 = &args[0];
        	//let arg2 = &args[2];
        	//println!("Arg1 is a: {:#?}", check_is_file_or_dir(&arg1));
		println!("Arg0: [{}]",&arg0);
		repl.run().context("Critical REPL error")?;
		Ok(())
    	} else {
        	println!("Wrong number of args: {}, expected {}. Quitting.",args.len(),1);
		//TODO: make this thing fail
		Ok(())
	}

}
