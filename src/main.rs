extern crate command_pattern;

use std::error::Error;
use command_pattern::*;

fn main() -> Result<(), Box<Error>> {

    let mut inv = Invoker::new();

    let result = exec_or_undo_all!(inv, {

        exec: {

            println!("exec 1");
            Ok("i am result")

        },
        unexec: {

            println!("unexec 1");
            Ok(())

        }

    })?;

    println!("received: {}", result);

    let result = exec_or_undo_all!(inv, {

        println!("exec 2");
        Err("i am error")?

    })?;

    println!("received: {}", result);

    Ok(())

}
