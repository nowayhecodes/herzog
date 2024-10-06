mod core;

use core::Container;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let container = Container::new("/home/user/rootfs".to_string(), "ls -l".to_string());
    container.run()?;

    Ok(())
}
