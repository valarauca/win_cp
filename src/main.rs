
#[macro_use]
extern crate lazy_static;

use std::path::Path;

pub mod bindings {
    windows::include_bindings!();
}

pub use bindings::{
    Windows::Win32::Storage::FileSystem::CopyFileW,
};

pub mod compat;
pub mod cli;
pub mod init;


fn copy<A,B>(existing: A, new: B, fail_if_exists: bool) -> std::io::Result<()>
where
    A: AsRef<Path>,
    B: AsRef<Path>
{

    let err = unsafe { CopyFileW(existing.as_ref().to_string_lossy().as_ref(), new.as_ref().to_string_lossy().as_ref(), fail_if_exists) };
    if err.0 != 0i32 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

fn main() -> Result<(),Box<dyn std::error::Error>> {
    init::co_initialize()?;

    let todo = cli::Todo::new()?;
    let overwrite = todo.overwrite;
    for (dest, src) in todo.copy_pairs {
        copy(&src, &dest, overwrite)?;
    }
    Ok(())    
}
