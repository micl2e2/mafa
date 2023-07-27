// Copyright (C) 2023 Michael Lee <micl2e2@proton.me>
//
// Licensed under the GNU General Public License, Version 3.0 or any later
// version <LICENSE-GPL or https://www.gnu.org/licenses/gpl-3.0.txt>.
//
// This file may not be copied, modified, or distributed except in compliance
// with the license.
//

use std::fs::create_dir_all;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::error::MafaError;
use crate::error::Result;

#[cfg(target_family = "unix")]
fn get_home_dir() -> String {
    use std::env;
    for (k, v) in env::vars() {
        if k == "HOME" {
            return v;
        }
    }
    return "".to_owned();
}

#[cfg(target_family = "unix")]
mod lock {
    use std::fs::File;
    use std::os::fd::AsRawFd;

    fn flock(file: &File, flag: libc::c_int) -> std::io::Result<()> {
        let ret = unsafe { libc::flock(file.as_raw_fd(), flag) };
        if ret < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn acquire(lock: &File) -> Result<(), u8> {
        flock(&lock, libc::LOCK_EX).unwrap();

        Ok(())
    }

    pub fn release(lock: &File) -> Result<(), u8> {
        flock(&lock, libc::LOCK_UN).unwrap();

        Ok(())
    }
}

use lock::acquire as lock_acquire;
use lock::release as lock_release;

#[derive(Debug)]
pub struct MafaData {
    home_pbuf: PathBuf,
    data_root: &'static str,
    sver: &'static str, // structure version
    lock_dir: &'static str,
    cache_dir: &'static str,
}

impl MafaData {
    pub fn init() -> MafaData {
        let home_dir = get_home_dir();
        let data_root = ".mafa";
        let sver = "v1"; // currently v1 structure in use
        let cache_dir = "cache";
        let lock_dir = "lock";

        // manually delete data_root to reset all setting

        let home_pbuf = PathBuf::new().join(&home_dir);

        create_dir_all(home_pbuf.join(data_root).join(sver)).unwrap();
        create_dir_all(home_pbuf.join(data_root).join(sver).join(cache_dir)).unwrap();
        create_dir_all(home_pbuf.join(data_root).join(sver).join(lock_dir)).unwrap();

        MafaData {
            home_pbuf: home_pbuf,
            data_root: data_root,
            sver,
            lock_dir,
            cache_dir: cache_dir,
        }
    }

    pub fn pathto_exist_cache(&self, cache_id: &str) -> Result<PathBuf> {
        let pbuf = self
            .home_pbuf
            .join(self.data_root)
            .join(self.sver)
            .join(self.cache_dir)
            .join(cache_id);

        if let Ok(flag) = Path::new(&pbuf).try_exists() {
            if !flag {
                return Err(MafaError::MafaDataCacheNotFound);
            }
        } else {
            return Err(MafaError::Buggy);
        }

        Ok(pbuf)
    }

    ///
    /// current implementation does not need extra surrounded
    /// lock, since locks here all are assumed empty, it is
    /// completely safe for a large number of threads open
    /// a empty lock simutaneously.
    ///
    /// and another invariant makes it much more safe: most of
    /// time, users are requesting a existing lock, not a new one.
    fn cache_lock(&self, lock_name: &str) -> Result<File> {
        let pbuf = self
            .home_pbuf
            .join(self.data_root)
            .join(self.sver)
            .join(self.lock_dir)
            .join(lock_name);

        if let Ok(flag) = Path::new(&pbuf).try_exists() {
            if !flag {
                let lock_f = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&pbuf)
                    .or_else(|_| Err(MafaError::BugFound(3456)))?;

                return Ok(lock_f);
            }
        } else {
            return Err(MafaError::BugFound(1234));
        }

        // if exist
        let lock_f = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&pbuf)
            .or_else(|_| Err(MafaError::BugFound(3456)))?;

        Ok(lock_f)
    }

    pub fn cache_append(
        &self,
        cache_id: &str,
        data_if_exis: &str,
        data_if_nexis: &str,
    ) -> Result<()> {
        let mut is_cache_exist = false;
        let pbuf = self
            .home_pbuf
            .join(self.data_root)
            .join(self.sver)
            .join(self.cache_dir)
            .join(cache_id);

        if let Ok(flag) = Path::new(&pbuf).try_exists() {
            if flag {
                is_cache_exist = true;
            }
        } else {
            return Err(MafaError::Buggy);
        }

        if is_cache_exist {
            lock_acquire(&self.cache_lock(cache_id)?).expect("buggy");
            let mut f = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&pbuf)
                .or_else(|_| Err(MafaError::BugFound(3456)))?;
            let mut buf_olddata = [0u8; 4096];
            let mut total_read = 0;
            let mut nread = 1;
            while nread != 0 {
                nread = f.read(&mut buf_olddata).unwrap();
                total_read += nread;
            }
            f.rewind().unwrap();
            f.write_all(data_if_exis.as_bytes()).unwrap();
            f.write_all(&buf_olddata[0..total_read]).unwrap();
            lock_release(&self.cache_lock(cache_id)?).expect("buggy");
        } else {
            self.init_cache(cache_id, data_if_nexis)?;
        }

        Ok(())
    }

    pub fn try_init_cache(&self, cache_id: &str, data: &str) -> Result<()> {
        let pbuf = self
            .home_pbuf
            .join(self.data_root)
            .join(self.sver)
            .join(self.cache_dir)
            .join(cache_id);

        lock_acquire(&self.cache_lock(cache_id)?).expect("buggy");

        if let Ok(flag) = Path::new(&pbuf).try_exists() {
            if flag {
                return Ok(());
            }
        } else {
            return Err(MafaError::Buggy);
        }

        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&pbuf)
            .or_else(|_| Err(MafaError::BugFound(3456)))?;

        if let Err(_err_io) = f.write_all(&data.as_bytes()) {
            dbgg!((&pbuf, _err_io));
            return Err(MafaError::Buggy);
        }

        lock_release(&self.cache_lock(cache_id)?).expect("buggy");

        Ok(())
    }

    ///
    /// whether cache exists or not, write data into cache_id, create
    /// before write if not exist
    pub fn init_cache(&self, cache_id: &str, data: &str) -> Result<()> {
        let pbuf = self
            .home_pbuf
            .join(self.data_root)
            .join(self.sver)
            .join(self.cache_dir)
            .join(cache_id);

        lock_acquire(&self.cache_lock(cache_id)?).expect("buggy");

        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&pbuf)
            .or_else(|_| Err(MafaError::BugFound(3456)))?;

        if let Err(_err_io) = f.write_all(&data.as_bytes()) {
            dbgg!((&pbuf, _err_io));
            return Err(MafaError::Buggy);
        }

        lock_release(&self.cache_lock(cache_id)?).expect("buggy");

        Ok(())
    }
}
