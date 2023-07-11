// Copyright (C) 2023 Michael Lee <imichael2e2@proton.me/...@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 or any later
// version <LICENSE-GPL or https://www.gnu.org/licenses/gpl-3.0.txt>.
//
// This file may not be copied, modified, or distributed except in compliance
// with the license.
//

use clap::Arg as ClapArg;
use clap::ArgAction as ClapArgAction;
use clap::ArgMatches as ClapArgMatches;
use clap::Command as ClapCommand;

pub mod error;

pub use error::MafaError;
use error::Result;

#[macro_use]
mod private_macros;

pub mod mafadata;

pub mod ev_ntf;

#[cfg(any(feature = "twtl", feature = "gtrans", feature = "camd"))]
mod comm;

#[cfg(feature = "twtl")]
pub mod twtl;

#[cfg(feature = "gtrans")]
pub mod gtrans;

#[cfg(feature = "camd")]
pub mod camd;

#[derive(Debug, Default)]
pub struct MafaInput {
    pub silent: bool,
    pub nocolor: bool,
    pub tout_page_load: u32,
    pub tout_script: u32,
    pub socks5: String,
    pub gui: bool,
}

impl MafaInput {
    pub fn from_ca_matched(ca_matched: &ClapArgMatches) -> Result<Self> {
        let mut mafa_in = MafaInput::default();

        // silent
        if ca_matched.get_flag(opts::SilentMode::id()) {
            mafa_in.silent = true;
        }

        // nocolor
        if ca_matched.get_flag(opts::NoColorMode::id()) {
            mafa_in.nocolor = true;
        }

        // page load timeout
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::TimeoutPageLoad::id()) {
            mafa_in.tout_page_load = u32::from_str_radix(&optval, 10)
                .or_else(|_| Err(MafaError::InvalidTimeoutPageLoad))?;
        } else {
            mafa_in.tout_page_load = u32::from_str_radix(opts::TimeoutPageLoad::def_val(), 10)
                .or_else(|_| Err(MafaError::Buggy))?;
        }

        // script timeout
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::TimeoutScript::id()) {
            mafa_in.tout_script = u32::from_str_radix(&optval, 10)
                .or_else(|_| Err(MafaError::InvalidTimeoutScript))?;
        } else {
            mafa_in.tout_script = u32::from_str_radix(opts::TimeoutScript::def_val(), 10)
                .or_else(|_| Err(MafaError::Buggy))?;
        }

        // socks5
        if let Ok(Some(val)) = ca_matched.try_get_one::<String>(opts::Socks5Proxy::id()) {
            mafa_in.socks5 = val.clone();
        }

        // gui
        if ca_matched.get_flag(opts::GuiMode::id()) {
            mafa_in.gui = true;
        }

        dbgg!(&mafa_in);

        Ok(mafa_in)
    }
}

// opts //

pub mod opts {
    use core::ops::Range;
    #[allow(unused)]
    use core::ops::RangeFrom;

    pub struct SilentMode;
    impl SilentMode {
        #[inline]
        pub fn id() -> &'static str {
            "SILENT_MODE"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "silent"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Enable silent mode"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = "Enable silent mode

Any insignificant information redirected to STDOUT will be hidden.

NOTE: subcommands can override this option if this one is not specified.";

            let mut af_buf = [0u8; 256];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct NoColorMode;
    impl NoColorMode {
        #[inline]
        pub fn id() -> &'static str {
            "NOCOLOR_MODE"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "nocolor"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Print without color."
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = "Print without color

Any information will be printed without using ANSI escape codes

NOTE: subcommands can override this option if this one is not specified.";

            let mut af_buf = [0u8; 256];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct GuiMode;
    impl GuiMode {
        #[inline]
        pub fn id() -> &'static str {
            "GUI_MODE"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "gui"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Enable GUI mode"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = "Enable GUI mode

Run the underlying web browser in GUI mode.

USED WITH CAUTION: when GUI mode is on, any user operation on web browser interface MAY affect mafa's correctness.

NOTE: subcommands can override this option.";

            let mut af_buf = [0u8; 256];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct Socks5Proxy;
    impl Socks5Proxy {
        #[inline]
        pub fn id() -> &'static str {
            "SOCKS5_PROXY"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "socks5"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Fetch with SOCKS5 proxy                                        "
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = "Fetch with SOCKS5 proxy

The default SOCKS5 proxy used by the underlying web browser to fetch data. This is also used by mafa to fetch webdriver server binaries at first initialization.

NOTE: subcommands can override this option.";

            let mut af_buf = [0u8; 256];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct TimeoutPageLoad;
    impl TimeoutPageLoad {
        #[inline]
        pub fn id() -> &'static str {
            "TIMEOUT_PAGE_LOAD"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "timeout-pageload"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn def_val() -> &'static str {
            "30000"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Timeout for page loading(ms)"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = "Timeout for page loading(ms)

The default timeout for loading web page(i.e., opening a website). Refer to WebDriver standard(https://www.w3.org/TR/webdriver2/#timeouts) for more details.

NOTE: subcommands can override this option.";
            let mut af_buf = [0u8; 512];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct TimeoutScript;
    impl TimeoutScript {
        #[inline]
        pub fn id() -> &'static str {
            "TIMEOUT_SCRIPT"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "timeout-script"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn def_val() -> &'static str {
            "30000"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Timeout for script evaluation(ms)"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = "Timeout for script evaluation(ms)

The default timeout for script evaluation(i.e., evaluating JavaScript synchronously or asynchronously). Refer to WebDriver standard(https://www.w3.org/TR/webdriver2/#timeouts) for more details.

NOTE: subcommands can override this option.";
            let mut af_buf = [0u8; 512];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }
}

pub fn get_cmd() -> ClapCommand {
    let opt_silient = {
        type O = opts::SilentMode;
        ClapArg::new(O::id())
            .long(O::longopt())
            .action(ClapArgAction::SetTrue)
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_nocolor = {
        type O = opts::NoColorMode;
        ClapArg::new(O::id())
            .long(O::longopt())
            .action(ClapArgAction::SetTrue)
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_gui = {
        type O = opts::GuiMode;
        ClapArg::new(O::id())
            .long(O::longopt())
            .action(ClapArgAction::SetTrue)
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_socks5 = {
        type O = opts::Socks5Proxy;
        ClapArg::new(O::id())
            .long(O::longopt())
            .num_args(O::n_args())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_tout_pageload = {
        type O = opts::TimeoutPageLoad;
        ClapArg::new(O::id())
            .long(O::longopt())
            .num_args(O::n_args())
            .default_value(O::def_val())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_tout_script = {
        type O = opts::TimeoutScript;
        ClapArg::new(O::id())
            .long(O::longopt())
            .num_args(O::n_args())
            .default_value(O::def_val())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    static HELPER_TXT: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
        let mut s = format!("{}\nfeatures: ", clap::crate_version!());
        #[cfg(feature = "imode")]
        {
            s += "IMODE ";
        }
        #[cfg(feature = "gtrans")]
        {
            s += "GTRANS ";
        }
        #[cfg(feature = "twtl")]
        {
            s += "TWTL ";
        }

        s += "\n\n";
        s += &format!("Copyright (C) 2023 {}.", clap::crate_authors!());
        s += "
This is free software.  You may redistribute copies of it under the terms of
the GNU General Public License <https://www.gnu.org/licenses/gpl.html>.
There is NO WARRANTY, to the extent permitted by law.";

        s
    });

    let cmd_mafa = ClapCommand::new("mafa")
        .version(clap::crate_version!())
        .long_version(HELPER_TXT.as_str())
        .about("Mafa is a convenient tool for fetching web page data.");

    #[cfg(feature = "imode")]
    let cmd_mafa = cmd_mafa.subcommand(ClapCommand::new("i").about("Enter interactive mode"));

    #[cfg(feature = "twtl")]
    let cmd_mafa = cmd_mafa.subcommand(twtl::get_cmd());

    #[cfg(feature = "gtrans")]
    let cmd_mafa = cmd_mafa.subcommand(gtrans::get_cmd());

    #[cfg(feature = "camd")]
    let cmd_mafa = cmd_mafa.subcommand(camd::get_cmd());

    let cmd_mafa = cmd_mafa
        .arg(opt_silient)
        .arg(opt_nocolor)
        .arg(opt_gui)
        .arg(opt_socks5)
        .arg(opt_tout_pageload)
        .arg(opt_tout_script);

    cmd_mafa
}
