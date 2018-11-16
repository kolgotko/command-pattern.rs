### Example:

```rust
extern crate command_pattern;

use std::error::Error;
use std::any::Any;
use command_pattern::*;

fn main() -> Result<(), Box<Error>> {

    let mut inv: Invoker<Box<dyn Any>> = Invoker::new();

    let result = exec_or_undo_all!(inv, {

        exec: move {

            println!("exec 1");
            Ok(Box::new("i am result") as Box<Any>)

        },
        unexec: move {

            println!("unexec 1");
            Ok(())

        }

    })?;

    let result: &str = result.downcast_ref::<&str>()
        .ok_or("downcast error")?
        .to_owned();

    println!("received: {:?}", result);

    let result = exec_or_undo_all!(inv, move {

        println!("exec 2");
        Err("i am error")?

    })?;

    println!("received: {:?}", result);

    Ok(())

}
```
