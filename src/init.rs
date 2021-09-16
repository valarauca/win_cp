

use super::bindings::{
    Windows::Win32::System::Com::CoInitialize,
};

use std::sync::{Arc,Mutex};

lazy_static! {
    static ref INIT: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}


pub fn co_initialize() -> Result<(),Box<dyn std::error::Error>> {

    let mut flag = INIT.lock()?;
    if !*flag {
        unsafe { CoInitialize(std::ptr::null_mut())? };
        *flag = true;
    }
    Ok(())
}
