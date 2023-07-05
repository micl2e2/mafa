// Copyright (C) 2023 Michael Lee <imichael2e2@proton.me/...@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 or any later
// version <LICENSE-GPL or https://www.gnu.org/licenses/gpl-3.0.txt>.
//
// This file may not be copied, modified, or distributed except in compliance
// with the license.
//

#[allow(unused_macros)]
macro_rules! lock_or_rtn {
    ($lck:expr) => {{
        let grd = $lck.lock();
        if grd.is_err() {
            return 7;
        }
        let grd = grd.expect("buggy");
        grd
    }};
}

#[allow(unused_macros)]
macro_rules! lock_or_err {
    ($lck:expr) => {
        $lck.lock().or_else(|_| Err(MafaError::BugFound(7890)))?
    };
}

// debug only //

#[allow(unused_macros)]
macro_rules! dbgmsg {
    ($fmtstr:expr) => {
        #[cfg(feature = "dev")]
        let dbgmsg = format!($fmtstr);
        #[cfg(feature = "dev")]
        dbg!(dbgmsg);
    };
    ($fmtstr:expr, $($val:expr),+ $(,)?) => {
        #[cfg(feature = "dev")]
        let dbgmsg = format!($fmtstr, $($val),+);
        #[cfg(feature = "dev")]
        dbg!(dbgmsg);
    };
}

#[allow(unused_macros)]
macro_rules! dbgg {
    () => {
        #[cfg(feature = "dev")]
        dbg!();
    };
    ($val:expr $(,)?) => {
        #[cfg(feature = "dev")]
        dbg!($val);
    };
    ($($val:expr),+ $(,)?) => {
        #[cfg(feature = "dev")]
        ($(dbg!($val)),+);
    };
}
