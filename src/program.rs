use super::resources::*;
use super::shader::*;
use super::string_utils::*;
use gl::types::*;
use std::ffi::{CStr, CString};

pub struct Program {
    id: GLuint,
}

impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe {
                gl::AttachShader(id, shader.id());
            }
        }

        unsafe {
            gl::LinkProgram(id);
        }

        let mut success: GLint = 1;

        unsafe {
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: GLint = 0;

            unsafe {
                gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetProgramInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
            }

            for shader in shaders {
                unsafe {
                    gl::DetachShader(id, shader.id());
                }
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe {
                gl::DetachShader(id, shader.id());
            }
        }

        Ok(Program { id })
    }

    pub fn from_res(res: &Resources, name: &str) -> Result<Program, String> {
        const POSSIBLE_EXT: [&str; 2] = [".vert", ".frag"];

        let shaders = POSSIBLE_EXT
            .iter()
            .map(|file_extension| Shader::from_res(res, &format!("{}{}", name, file_extension)))
            .collect::<Result<Vec<Shader>, String>>()?;

        Program::from_shaders(&shaders[..])
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn mark_as_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}
