use std::ffi::{CStr, CString};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub struct Resources {
    root_path: PathBuf,
}

impl Resources {
    pub fn from_relative_exe_path(rel_path: &Path) -> Result<Resources, String> {
        let exe_file_name =
            std::env::current_exe().map_err(|err| format!("Can't get current exe name {}", err))?;
        let exe_path = exe_file_name.parent().unwrap();

        Ok(Resources {
            root_path: exe_path.join(rel_path),
        })
    }

    pub fn load_cstring(&self, resource_name: &str) -> Result<CString, String> {
        let mut file = fs::File::open(resource_name_to_path(&self.root_path, resource_name))
            .map_err(|err| format!("Can't open path {}", err))?;

        let mut buffer: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize + 1);
        file.read_to_end(&mut buffer);

        Ok(unsafe { CString::from_vec_unchecked(buffer) })
    }
}

fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.split("/") {
        path = path.join(part);
    }

    path
}
