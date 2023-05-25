use easy_repl::{Repl, CommandStatus, command};

fn main() {
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
	    .build().expect("Failed to create repl");

	repl.run().expect("Critical REPL error");
}
