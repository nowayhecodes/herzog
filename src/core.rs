use nix::sched::{clone, CloneFlags};
use nix::sys::wait::waitpid;
use nix::unistd::{chroot, pivot_root};
use std::error::Error;
use std::path::Path;
use std::process::Command;

pub struct Container {
    root_path: String,
    command: String,
}

impl Container {
    pub fn new(root_path: String, command: String) -> Self {
        Container { root_path, command }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let flags = CloneFlags::CLONE_NEWNS
            | CloneFlags::CLONE_NEWUTS
            | CloneFlags::CLONE_NEWIPC
            | CloneFlags::CLONE_NEWPID
            | CloneFlags::CLONE_NEWNET
            | CloneFlags::CLONE_NEWUSER;

        let child_pid = clone(
            Box::new(|| self.child_func()),
            &mut [0u8; 1024 * 1024],
            flags,
            None,
        )?;
        waitpid(child_pid, None)?;

        Ok(())
    }

    fn child_func(&self) -> isize {
        if let Err(e) = self.setup_container() {
            eprintln!("Failed to setup container: {}", e);
            return -1;
        }

        if let Err(e) = self.run_command() {
            eprintln!("Failed to run command: {}", e);
            return -1;
        }

        0
    }
    fn setup_container(&self) -> Result<(), Box<dyn Error>> {
        pivot_root(
            &Path::new(&self.root_path).join("root"),
            &Path::new(&self.root_path).join("old_root"),
        )?;
        chroot(".")?;

        std::env::set_current_dir("/")?;

        Ok(())
    }

    fn run_command(&self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("sh").arg("-c").arg(&self.command).output()?;

        if !output.status.success() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            )));
        }

        Ok(())
    }
}
