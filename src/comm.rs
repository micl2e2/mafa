// Copyright (C) 2023 Michael Lee <imichael2e2@proton.me/...@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 or any later
// version <LICENSE-GPL or https://www.gnu.org/licenses/gpl-3.0.txt>.
//
// This file may not be copied, modified, or distributed except in compliance
// with the license.
//

#[derive(Debug, Default, Copy, Clone)]
pub(crate) enum CacheMechanism {
    #[default]
    Local,
    Remote,
    No,
}

impl CacheMechanism {
    pub(crate) fn from_str(s: &str) -> Self {
        match s {
            "LOCAL" | "local" => Self::Local,
            "REMOTE" | "remote" => Self::Remote,
            "NO" | "no" => Self::No,
            _ => Self::Local,
        }
    }
}

pub(crate) fn replicate(part: &str, times: usize) -> String {
    let mut ret = String::from("");
    for _i in 0..times {
        ret += part;
    }

    ret
}

pub(crate) fn is_valid_socks5(v: &str) -> bool {
    if v.len() > 0 {
        true
    } else {
        false
    }
}

pub(crate) fn percent_encode(orig: &[u8]) -> Vec<u8> {
    let mut after = vec![];

    for ele in orig {
        match ele {
            b' ' => after.extend(b"%20"),
            b'!' => after.extend(b"%21"),
            b'"' => after.extend(b"%22"),
            b'#' => after.extend(b"%23"),
            b'$' => after.extend(b"%24"),
            b'%' => after.extend(b"%25"),
            b'&' => after.extend(b"%26"),
            b'\'' => after.extend(b"%27"),
            b'(' => after.extend(b"%28"),
            b')' => after.extend(b"%29"),
            b'*' => after.extend(b"%2A"),
            b'+' => after.extend(b"%2B"),
            b',' => after.extend(b"%2C"),
            b'/' => after.extend(b"%2F"),
            b':' => after.extend(b"%3A"),
            b';' => after.extend(b"%3B"),
            b'=' => after.extend(b"%3D"),
            b'?' => after.extend(b"%3F"),
            b'@' => after.extend(b"%40"),
            b'[' => after.extend(b"%5B"),
            b']' => after.extend(b"%5D"),
            other => after.push(*other),
        }
    }
    after
}

fn del_redun_escapes(s: &str) -> String {
    s.replace(r#"\""#, r#"""#)
}

pub(crate) fn make_readable(s: &str) -> String {
    let s1 = s.trim();
    let s2 = s1.replace(r#"\""#, r#"""#);

    s2
}
