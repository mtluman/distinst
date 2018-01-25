use libc::{c_ulong, c_void, mount, umount2, MNT_DETACH, MS_BIND, MS_SYNCHRONOUS};
use std::ffi::CString;
use std::io::{Error, ErrorKind, Result};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr;

pub const BIND: c_ulong = MS_BIND;
pub const SYNC: c_ulong = MS_SYNCHRONOUS;

pub struct Mounts(pub Vec<Mount>);

impl Drop for Mounts {
    fn drop(&mut self) {
        for mount in self.0.drain(..).rev() {
            drop(mount);
        }
    }
}

#[derive(Debug)]
pub struct Mount {
    source:  PathBuf,
    dest:    PathBuf,
    mounted: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum MountOption {
    Bind,
    Synchronize,
}

impl Mount {
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(
        source: P,
        dest: Q,
        options: &[MountOption],
    ) -> Result<Mount> {
        let source = source.as_ref().canonicalize()?;
        let dest = dest.as_ref().canonicalize()?;

        let mut command = Command::new("mount");

        let mut option_strings = Vec::new();
        for &option in options.iter() {
            match option {
                MountOption::Bind => {
                    command.arg("--bind");
                }
                MountOption::Synchronize => {
                    option_strings.push("sync");
                }
            }
        }

        option_strings.sort();
        option_strings.dedup();
        if !option_strings.is_empty() {
            command.arg("-o");
            command.arg(option_strings.join(","));
        }

        command.arg(&source);
        command.arg(&dest);

        debug!("{:?}", command);

        let status = command.status()?;
        if status.success() {
            Ok(Mount {
                source:  source,
                dest:    dest,
                mounted: true,
            })
        } else {
            Err(Error::new(
                ErrorKind::Other,
                format!("mount failed with status: {}", status),
            ))
        }
    }

    pub fn mount_part<P: AsRef<Path>>(
        src: P,
        target: P,
        fstype: &str,
        flags: c_ulong,
        options: Option<&str>,
    ) -> Result<Mount> {
        let c_src = CString::new(src.as_ref().as_os_str().as_bytes().to_owned());
        let c_target = CString::new(target.as_ref().as_os_str().as_bytes().to_owned());
        let c_fstype = CString::new(fstype.to_owned());
        let c_options = options.and_then(|options| CString::new(options.to_owned()).ok());

        let c_src = c_src
            .as_ref()
            .ok()
            .map_or(ptr::null(), |cstr| cstr.as_ptr());
        let c_target = c_target
            .as_ref()
            .ok()
            .map_or(ptr::null(), |cstr| cstr.as_ptr());
        let c_fstype = c_fstype
            .as_ref()
            .ok()
            .map_or(ptr::null(), |cstr| cstr.as_ptr());
        let c_options = c_options.as_ref().map_or(ptr::null(), |cstr| cstr.as_ptr());

        match unsafe { mount(c_src, c_target, c_fstype, flags, c_options as *const c_void) } {
            0 => Ok(Mount {
                source:  src.as_ref().to_path_buf(),
                dest:    target.as_ref().to_path_buf(),
                mounted: true,
            }),
            _err => Err(Error::last_os_error()),
        }
    }

    pub fn unmount(&mut self, lazy: bool) -> Result<()> {
        if self.mounted {
            unsafe {
                let mount = CString::new(self.dest().as_os_str().as_bytes().to_owned());
                let mount_ptr = mount
                    .as_ref()
                    .ok()
                    .map_or(ptr::null(), |cstr| cstr.as_ptr());
                match umount2(mount_ptr, if lazy { MNT_DETACH } else { 0 }) {
                    0 => Ok(()),
                    _err => Err(Error::last_os_error()),
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn dest(&self) -> &Path { &self.dest }
}

impl Drop for Mount {
    fn drop(&mut self) { let _ = self.unmount(true); }
}
