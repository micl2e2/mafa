// Copyright (C) 2023 Michael Lee <imichael2e2@proton.me/...@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 or any later
// version <LICENSE-GPL or https://www.gnu.org/licenses/gpl-3.0.txt>.
//
// This file may not be copied, modified, or distributed except in compliance
// with the license.
//

use std::borrow::Cow;
use std::sync::Arc;
use std::sync::Mutex;
use wda::{GeckoDriver, WdaError, WdaSett, WdcError, WebDrvAstn};

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
use mafadata::MafaData;

pub mod ev_ntf;
use ev_ntf::EurKind;
use ev_ntf::EventNotifier;

#[cfg(any(feature = "twtl", feature = "gtrans", feature = "camd"))]
mod comm;
#[cfg(any(feature = "twtl", feature = "gtrans", feature = "camd"))]
use comm::CacheMechanism;

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
    pub ascii: bool,
    pub wrap_width: u16,
    pub wrap_may_break: bool,
    pub tout_page_load: u32,
    pub tout_script: u32,
    pub socks5: String,
    pub gui: bool,
    pub list_profile: bool,
    pub use_profile: String,
    cachm: CacheMechanism,
    pub elap: bool,
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

        // ascii
        if ca_matched.get_flag(opts::AsciiMode::id()) {
            mafa_in.ascii = true;
        }

        // wrap-width
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::WrapWidth::id()) {
            let intval =
                u16::from_str_radix(&optval, 10).map_err(|_| MafaError::InvalidWrapWidth)?;
            mafa_in.wrap_width = intval;
        }

        // wrap-may-break
        if ca_matched.get_flag(opts::WrapMayBreak::id()) {
            mafa_in.wrap_may_break = true;
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

        // elap
        if ca_matched.get_flag(opts::Elapsed::id()) {
            mafa_in.elap = true;
        }

        // cachm
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::CacheMech::id()) {
            mafa_in.cachm = CacheMechanism::from_str(optval);
        }

        // list profile
        if ca_matched.get_flag(opts::ListProfile::id()) {
            mafa_in.list_profile = true;
        }

        // use profile
        if let Ok(Some(val)) = ca_matched.try_get_one::<String>(opts::UseProfile::id()) {
            mafa_in.use_profile = val.clone();
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

Any insignificant information redirected to STDOUT will be hidden.";

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

Any information will be printed without using ANSI escape codes.";

            let mut af_buf = [0u8; 256];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct AsciiMode;
    impl AsciiMode {
        #[inline]
        pub fn id() -> &'static str {
            "ASCIIMODE"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "ascii"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Use classical ASCII style"
        }
    }

    pub struct WrapWidth;
    impl WrapWidth {
        #[inline]
        pub fn id() -> &'static str {
            "WRAPWIDTH"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "wrap-width"
        }
        #[inline]
        pub fn def_val() -> &'static str {
            "80"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Wrap output with width limit"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = r#"Wrap output with width limit

NOTE: the minimum is 18, any value smaller than 18 will fallback to 80."#;
            let mut af_buf = [0u8; 128];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct WrapMayBreak;
    impl WrapMayBreak {
        #[inline]
        pub fn id() -> &'static str {
            "WRAPMAYBREAK"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "wrap-may-break"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Wrap output in MayBreak style"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = r#"Wrap output in MayBreak style

Default is "NoBreak" style. This change the style to "MayBreak".

NOTE: This is a hint option, components may ignore it.

NOTE: "NoBreak" suits for languages that rely on ASCII SPACE to delimit words. "MayBreak" suits for otherwise languages. See more details in Bwrap documentation: https://docs.rs/bwrap/latest/bwrap/enum.WrapStyle.html."#;
            let mut af_buf = [0u8; 512];

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

USED WITH CAUTION: when GUI mode is on, any user operation on web browser interface MAY affect mafa's correctness.";

            let mut af_buf = [0u8; 256];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct ListProfile;
    impl ListProfile {
        #[inline]
        pub fn id() -> &'static str {
            "LIST_PROFILE"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "list-p"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "List all existing browser profiles"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = "List all existing browser profiles

Note that....";

            let mut af_buf = [0u8; 256];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct UseProfile;
    impl UseProfile {
        #[inline]
        pub fn id() -> &'static str {
            "USE_PROFILE"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "profile"
        }
        #[inline]
        pub fn shortopt() -> char {
            'p'
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Use specific browser profile"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = "Use specific browser profile

NOTE: the profile will be created if not existing.

NOTE: the size of browser profiles is non-trivial, use with caution!
";

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

The default SOCKS5 proxy used by the underlying web browser to fetch data. This is also used by mafa to fetch webdriver server binaries at first initialization.";

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

The default timeout for loading web page(i.e., opening a website). Refer to WebDriver standard(https://www.w3.org/TR/webdriver2/#timeouts) for more details.";
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

The default timeout for script evaluation(i.e., evaluating JavaScript synchronously or asynchronously). Refer to WebDriver standard(https://www.w3.org/TR/webdriver2/#timeouts) for more details.";
            let mut af_buf = [0u8; 512];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct CacheMech;
    impl CacheMech {
        #[inline]
        pub fn id() -> &'static str {
            "CACHE_MECHNISM"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "cache"
        }
        #[inline]
        pub fn def_val() -> &'static str {
            "LOCAL"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "The caching mechanism"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = r#"The caching mechanism

Available values are: LOCAL, REMOTE, NO.

LOCAL instructs mafa to use local cache, typically located in mafa's dedicated cache directory; REMOTE instructs mafa to use remote cache, which is stored on the internet and can be readily accessed and fetched, note that this option will override the corresponding cache; NO instructs mafa to build cache freshly, this usually needs more time, compared to other mechanisms."#;
            let mut af_buf = [0u8; 512];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct Elapsed;
    impl Elapsed {
        #[inline]
        pub fn id() -> &'static str {
            "ELAPSED"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "elap"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Report the time cost in major phases"
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

    let opt_ascii = {
        type O = opts::AsciiMode;
        ClapArg::new(O::id())
            .long(O::longopt())
            .action(ClapArgAction::SetTrue)
            .help(O::helper())
    };

    let opt_wrapwidth = {
        type O = opts::WrapWidth;
        ClapArg::new(O::id())
            .long(O::longopt())
            .default_value(O::def_val())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_wrapmaybreak = {
        type O = opts::WrapMayBreak;
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

    let opt_cachm = {
        type O = opts::CacheMech;
        ClapArg::new(O::id())
            .long(O::longopt())
            .default_value(O::def_val())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_elapsed = {
        type O = opts::Elapsed;
        ClapArg::new(O::id())
            .long(O::longopt())
            .action(ClapArgAction::SetTrue)
            .help(O::helper())
    };

    let opt_list_profile = {
        type O = opts::ListProfile;
        ClapArg::new(O::id())
            .long(O::longopt())
            .action(ClapArgAction::SetTrue)
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_use_profile = {
        type O = opts::UseProfile;
        ClapArg::new(O::id())
            .long(O::longopt())
            .short(O::shortopt())
            .num_args(O::n_args())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    static HELPER_TXT: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
        let mut s = format!("{}\ncomponents: ", clap::crate_version!());
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
        #[cfg(feature = "camd")]
        {
            s += "CAMD ";
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
        .about(clap::crate_description!());

    #[cfg(feature = "imode")]
    let cmd_mafa = cmd_mafa.subcommand(
        ClapCommand::new("i")
            .about("Enter interactive mode")
            .long_about(
                "Enter interactive mode

With interactive mode, Mafa's components interact with websites statefully,
performing tasks without full initializtion of the underlying WebDriver,
this usually results in faster performance.

Note that under interactive mode, components' options are identical to
ones under normal mode, i.e., -h for short help, --help for long help.
",
            ),
    );

    #[cfg(feature = "twtl")]
    let cmd_mafa = cmd_mafa.subcommand(twtl::get_cmd());

    #[cfg(feature = "gtrans")]
    let cmd_mafa = cmd_mafa.subcommand(gtrans::get_cmd());

    #[cfg(feature = "camd")]
    let cmd_mafa = cmd_mafa.subcommand(camd::get_cmd());

    let cmd_mafa = cmd_mafa
        .arg(opt_silient)
        .arg(opt_nocolor)
        .arg(opt_ascii)
        .arg(opt_wrapwidth)
        .arg(opt_wrapmaybreak)
        .arg(opt_gui)
        .arg(opt_socks5)
        .arg(opt_tout_pageload)
        .arg(opt_tout_script)
        .arg(opt_cachm)
        .arg(opt_elapsed)
        .arg(opt_list_profile)
        .arg(opt_use_profile);

    cmd_mafa
}

//

#[derive(Debug)]
pub struct MafaClient<'a, 'b, 'c, I, C> {
    mafad: &'a MafaData,
    ntf: Arc<Mutex<EventNotifier>>,
    input: &'b MafaInput,
    sub_input: I,
    wda: &'c WebDrvAstn<GeckoDriver>,
    caches: Vec<C>,
}

impl<'a, 'b, 'c, I, C: Default> MafaClient<'a, 'b, 'c, I, C> {
    pub fn new(
        mafad: &'a MafaData,
        ntf: Arc<Mutex<EventNotifier>>,
        mafa_in: &'b MafaInput,
        sub_in: I,
        wda_inst: &'c WebDrvAstn<GeckoDriver>,
    ) -> Self {
        MafaClient {
            mafad,
            ntf,
            input: mafa_in,
            sub_input: sub_in,
            wda: wda_inst,
            caches: Default::default(),
        }
    }
}

fn get_wda_setts(mafa_in: &MafaInput) -> Vec<WdaSett> {
    let mut wda_setts = vec![];

    // gui
    if !mafa_in.gui {
        wda_setts.push(WdaSett::NoGui);
    }

    // socks5
    if comm::is_valid_socks5(&mafa_in.socks5) {
        wda_setts.push(WdaSett::PrepareUseSocksProxy(Cow::Borrowed(
            &mafa_in.socks5,
        )));
        wda_setts.push(WdaSett::Socks5Proxy(Cow::Borrowed(&mafa_in.socks5)));
        wda_setts.push(WdaSett::ProxyDnsSocks5);
    }

    // tout pageload
    wda_setts.push(WdaSett::PageLoadTimeout(mafa_in.tout_page_load));

    // tout script
    wda_setts.push(WdaSett::ScriptTimeout(mafa_in.tout_script));

    // profile
    if mafa_in.use_profile.len() > 0 {
        wda_setts.push(WdaSett::BrowserProfileId(Cow::Borrowed(
            &mafa_in.use_profile,
        )))
    }

    dbgg!(&wda_setts);

    wda_setts
}

pub fn init_wda(mafa_in: &MafaInput) -> Result<WebDrvAstn<GeckoDriver>> {
    let wda_inst: WebDrvAstn<GeckoDriver>;
    let wda_setts = get_wda_setts(&mafa_in);
    dbgg!(&wda_setts);
    match WebDrvAstn::<GeckoDriver>::new(wda_setts) {
        Ok(ret) => wda_inst = ret,
        Err(err_wda) => match err_wda {
            WdaError::WdcNotReady(WdcError::BadDrvCmd(err, msg), _) => {
                if msg.contains("socksProxy is not a valid URL") {
                    return Err(MafaError::InvalidSocks5Proxy);
                } else {
                    return Err(MafaError::WebDrvCmdRejected(err, msg));
                }
            }
            WdaError::InvalidBrowserProfileId => return Err(MafaError::InvalidUseProfile),
            _ => {
                return Err(MafaError::UnexpectedWda(err_wda));
            }
        },
    };

    Ok(wda_inst)
}
