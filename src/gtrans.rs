// Copyright (C) 2023 Michael Lee <imichael2e2@proton.me/...@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 or any later
// version <LICENSE-GPL or https://www.gnu.org/licenses/gpl-3.0.txt>.
//
// This file may not be copied, modified, or distributed except in compliance
// with the license.
//

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

use wda::BasicAutomation;
use wda::WdaError;
use wda::WdaSett;
use wda::WebDrvAstn;

use wda::GeckoDriver;
use wda::WdcError;

use crate::error::CacheRebuildFailKind;
use crate::error::MafaError;
use crate::error::Result;

use crate::mafadata::MafaData;

use crate::ev_ntf::Category;
use crate::ev_ntf::EurKind;
use crate::ev_ntf::EventNotifier;
use crate::ev_ntf::MafaEvent;

use crate::MafaInput;

use crate::comm;
use crate::comm::CacheMechanism;

use clap::Arg as ClapArg;
use clap::ArgAction as ClapArgAction;
use clap::ArgMatches as ClapArgMatches;
use clap::Command as ClapCommand;

#[derive(Debug, Default)]
pub struct GtransInput {
    words: String,
    list_lang: bool,
    src_lang: String,
    tgt_lang: String,
    cachm: CacheMechanism,
    elap: bool,
    ascii: bool,
    wrap_width: u16,
    // below are optional ones, bc mafa would provide these fields anyway;
    // once specified in gtrans, mafa's corresponding ones shall be ignored
    silent: Option<bool>,
    nocolor: Option<bool>,
    // webdrv
    tout_page_load: Option<u32>,
    tout_script: Option<u32>,
    socks5: Option<String>,
    gui: Option<bool>,
}

impl GtransInput {
    pub fn from_ca_matched(ca_matched: &ClapArgMatches) -> Result<Self> {
        let mut gtrans_in = GtransInput::default();

        if let Ok(Some(optval)) = ca_matched.try_get_many::<String>(opts::Words::id()) {
            let words = optval
                .collect::<Vec<&String>>()
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .join(" ");

            if !is_valid_words(&words) {
                return Err(MafaError::InvalidWords);
            }
            gtrans_in.words = words;
        }

        if ca_matched.get_flag(opts::ListLang::id()) {
            gtrans_in.list_lang = true;
        }

        // sl
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::SourceLang::id()) {
            if !is_valid_lang(optval) {
                return Err(MafaError::InvalidSourceLang);
            }
            gtrans_in.src_lang = optval.clone();
        }

        // tl
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::TargetLang::id()) {
            if !is_valid_lang(optval) {
                return Err(MafaError::InvalidTargetLang);
            }
            gtrans_in.tgt_lang = optval.clone();
        }

        // cachm
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::CacheMech::id()) {
            gtrans_in.cachm = CacheMechanism::from_str(optval);
        }

        // elap
        if ca_matched.get_flag(opts::Elapsed::id()) {
            gtrans_in.elap = true;
        }

        // silent
        if ca_matched.get_flag(opts::SilentMode::id()) {
            gtrans_in.silent = Some(true);
        }

        // nocolor
        if ca_matched.get_flag(opts::NoColorMode::id()) {
            gtrans_in.nocolor = Some(true);
        }

        // ascii
        if ca_matched.get_flag(opts::AsciiMode::id()) {
            gtrans_in.ascii = true;
        }

        // wrap-width
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::WrapWidth::id()) {
            let intval =
                u16::from_str_radix(&optval, 10).map_err(|_| MafaError::InvalidWrapWidth)?;
            gtrans_in.wrap_width = intval;
        }

        // gui
        if ca_matched.get_flag(opts::GuiMode::id()) {
            gtrans_in.gui = Some(true);
        }

        // socks5
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::Socks5Proxy::id()) {
            gtrans_in.socks5 = Some(optval.clone());
        }

        // page load
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::TimeoutPageLoad::id()) {
            dbgg!(123);
            let intval =
                u32::from_str_radix(&optval, 10).map_err(|_| MafaError::InvalidTimeoutPageLoad)?;
            gtrans_in.tout_page_load = Some(intval);
            dbgg!(123);
        }

        // script
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::TimeoutScript::id()) {
            gtrans_in.tout_script = Some(
                u32::from_str_radix(&optval, 10).map_err(|_| MafaError::InvalidTimeoutScript)?,
            );
        }

        dbgg!(&gtrans_in);

        Ok(gtrans_in)
    }

    pub fn from_i_mode(mafa_in: &MafaInput, args: Vec<&str>) -> Result<GtransInput> {
        let cmd_gtrans = get_cmd();

        let m = cmd_gtrans.try_get_matches_from(args);

        match m {
            Ok(ca_matched) => {
                let gtrans_in = GtransInput::from_ca_matched(&ca_matched)?;
                let merged_in = GtransInput::merge(gtrans_in, mafa_in)?;

                Ok(merged_in)
            }
            // this will print helper
            Err(err_match) => Err(MafaError::ClapMatchError(err_match.render())),
        }
    }

    fn merge(mut gtrans_in: GtransInput, mafa_in: &MafaInput) -> Result<Self> {
        // mafa wins
        if mafa_in.silent {
            gtrans_in.silent = Some(true);
        }
        if mafa_in.nocolor {
            gtrans_in.nocolor = Some(true);
        }

        // pick one between mafa_in and gtrans_in
        if gtrans_in.gui.is_none() {
            gtrans_in.gui = Some(mafa_in.gui);
        }

        if gtrans_in.socks5.is_none() {
            gtrans_in.socks5 = Some(mafa_in.socks5.to_string());
        }

        if gtrans_in.tout_page_load.is_none() {
            gtrans_in.tout_page_load = Some(mafa_in.tout_page_load);
        }

        if gtrans_in.tout_script.is_none() {
            gtrans_in.tout_script = Some(mafa_in.tout_script);
        }

        Ok(gtrans_in)
    }

    pub fn is_list_lang(&self) -> bool {
        self.list_lang
    }

    pub fn is_silent(&self) -> bool {
        self.silent.is_some()
    }

    pub fn is_nocolor(&self) -> bool {
        self.nocolor.is_some()
    }
}

// opts //

pub mod opts {
    use core::ops::Range;
    use core::ops::RangeFrom;

    pub struct Words;
    impl Words {
        #[inline]
        pub fn id() -> &'static str {
            "WORDS"
        }
        #[inline]
        pub fn n_args() -> RangeFrom<usize> {
            1..
        }
        #[inline]
        pub fn helper() -> &'static str {
            "The words you want to translate"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = r#"The words you want to translate

WORDS can be single or multiple, for example:

$ mafa gtrans thanks

or 

$ mafa gtrans thank you"#;
            let mut af_buf = [0u8; 512];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct ListLang;
    impl ListLang {
        #[inline]
        pub fn id() -> &'static str {
            "LIST_LANG"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "list-lang"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "List all supported languages"
        }
    }

    pub struct TargetLang;
    impl TargetLang {
        #[inline]
        pub fn id() -> &'static str {
            "TARGET_LANG"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "tl"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn def_val() -> &'static str {
            "auto"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Target language"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = r#"Target language

The target language is the language you want to translate into, for example, translate auto-detected language "thank you" into French(fr):

$ mafa gtrans --tl fr thank you
-> merci

Check --list-lang for all supported languages."#;
            let mut af_buf = [0u8; 512];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct SourceLang;
    impl SourceLang {
        #[inline]
        pub fn id() -> &'static str {
            "SOURCE_LANG"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "sl"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn def_val() -> &'static str {
            "auto"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Source language"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = r#"Source language

The source language is the language you want to translate from, for example, translate Spanish(es) "gracias" into English(en):

$ mafa gtrans gracias --sl es --tl en
-> thank you

Check --list-lang for all supported languages."#;
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
            "Wrap width for translation result"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = r#"Wrap width for translation result

NOTE: the minimum is 18, any value smaller than 18 will fallback to 80."#;
            let mut af_buf = [0u8; 128];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

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
            "Enable silent mode                                              "
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
            "Print without color"
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
            "Fetch with SOCKS5 proxy"
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
        pub fn helper() -> &'static str {
            "Timeout for page loading(ms)"
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
        pub fn helper() -> &'static str {
            "Timeout for script evaluation(ms)"
        }
    }
}

pub fn get_cmd() -> ClapCommand {
    let opt_words = {
        type O = opts::Words;
        ClapArg::new(O::id())
            // .required(true)
            .num_args(O::n_args())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_list_lang = {
        type O = opts::ListLang;
        ClapArg::new(O::id())
            .long(O::longopt())
            .action(ClapArgAction::SetTrue)
            .help(O::helper())
    };

    let opt_tl = {
        type O = opts::TargetLang;
        ClapArg::new(O::id())
            .long(O::longopt())
            .num_args(O::n_args())
            .default_value(O::def_val())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_sl = {
        type O = opts::SourceLang;
        ClapArg::new(O::id())
            .long(O::longopt())
            .num_args(O::n_args())
            .default_value(O::def_val())
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

    let opt_cachemech = {
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

    let opt_silient = {
        type O = opts::SilentMode;
        ClapArg::new(O::id())
            .long(O::longopt())
            .action(ClapArgAction::SetTrue)
            .help(O::helper())
    };

    let opt_nocolor = {
        type O = opts::NoColorMode;
        ClapArg::new(O::id())
            .long(O::longopt())
            .action(ClapArgAction::SetTrue)
            .help(O::helper())
    };

    let opt_gui = {
        type O = opts::GuiMode;
        ClapArg::new(O::id())
            .long(O::longopt())
            .action(ClapArgAction::SetTrue)
            .help(O::helper())
    };

    let opt_socks5 = {
        type O = opts::Socks5Proxy;
        ClapArg::new(O::id())
            .long(O::longopt())
            .num_args(O::n_args())
            .help(O::helper())
    };

    let opt_tout_pageload = {
        type O = opts::TimeoutPageLoad;
        ClapArg::new(O::id())
            .long(O::longopt())
            .num_args(O::n_args())
            .help(O::helper())
    };

    let opt_tout_script = {
        type O = opts::TimeoutScript;
        ClapArg::new(O::id())
            .long(O::longopt())
            .num_args(O::n_args())
            .help(O::helper())
    };

    let cmd_gtrans = ClapCommand::new("gtrans")
        .about("Query translation from Google Translate")
        .arg(opt_words)
        .arg(opt_list_lang)
        .arg(opt_tl)
        .arg(opt_sl)
        .arg(opt_ascii)
        .arg(opt_wrapwidth)
        .arg(opt_cachemech)
        .arg(opt_elapsed)
        .arg(opt_silient)
        .arg(opt_nocolor)
        .arg(opt_gui)
        .arg(opt_socks5)
        .arg(opt_tout_pageload)
        .arg(opt_tout_script);

    cmd_gtrans
}

#[derive(Debug, Default)]
struct Upath(Vec<u8>);

#[derive(Debug, Default)]
struct UpathCache(Vec<Upath>);

impl UpathCache {
    fn from_pbuf(pb: PathBuf) -> Result<Self> {
        let rawdata = std::fs::read_to_string(&pb).unwrap(); // FIXME: no unwrap

        let mut ret = vec![];

        let bytes = rawdata.as_bytes();

        let mut previ;
        let mut curri = 0;

        let mut caches = Vec::<&[u8]>::new();

        for i in 0..bytes.len() {
            if bytes[i] == 0xa {
                previ = if curri == 0 { curri } else { curri + 1 };
                curri = i;
                caches.push(&bytes[previ..curri]);
            }
        }

        for ele in caches {
            let deser = serde_json::from_slice::<Vec<u8>>(ele).expect("buggy");
            if deser.len() == 0 {
                return Err(MafaError::CacheCorrupted);
            }
            let mut nr = Upath::default();
            nr.0 = deser;
            ret.push(nr);
        }

        Ok(Self(ret))
    }
}

#[derive(Debug)]
pub struct GtransClient<'a> {
    mafad: &'a MafaData,
    ntf: Arc<Mutex<EventNotifier>>,
    input: GtransInput,
    wda: WebDrvAstn<GeckoDriver>,
    upaths: Vec<Upath>,
}

impl GtransClient<'_> {
    #[inline]
    pub fn is_elap_req(&self) -> bool {
        self.input.elap
    }

    #[inline]
    pub fn is_list_lang_req(&self) -> bool {
        self.input.list_lang
    }

    #[inline]
    pub fn is_silent_req(&self) -> bool {
        self.input.silent.is_some()
    }

    //

    pub fn absorb_minimal(&mut self, another_in: &GtransInput) {
        self.input.words = another_in.words.clone();
        self.input.list_lang = another_in.list_lang;
        self.input.src_lang = another_in.src_lang.clone();
        self.input.tgt_lang = another_in.tgt_lang.clone();
        self.input.cachm = another_in.cachm;
        self.input.elap = another_in.elap;
        self.input.silent = another_in.silent;
        self.input.nocolor = another_in.nocolor;
    }

    pub fn need_reprepare(&self, another_in: &GtransInput) -> bool {
        let this_in = &self.input;

        dbgg!((&this_in, another_in));

        if another_in.gui.is_some() {
            if this_in.gui.is_none() {
                return true;
            } else if this_in.gui.as_ref().expect("buggy")
                != another_in.gui.as_ref().expect("buggy")
            {
                return true;
            }
        } else if this_in.gui.is_some() {
            return true;
        }

        if another_in.socks5.is_some() {
            if this_in.socks5.is_none() {
                return true;
            } else if this_in.socks5.as_ref().expect("buggy")
                != another_in.socks5.as_ref().expect("buggy")
            {
                return true;
            }
        } else if this_in.socks5.is_some() {
            return true;
        }

        if another_in.tout_page_load.is_some() {
            if this_in.tout_page_load.is_none() {
                return true;
            } else if this_in.tout_page_load.as_ref().expect("buggy")
                != another_in.tout_page_load.as_ref().expect("buggy")
            {
                return true;
            }
        } else if this_in.tout_page_load.is_some() {
            return true;
        }

        if another_in.tout_script.is_some() {
            if this_in.tout_script.is_none() {
                return true;
            } else if this_in.tout_script.as_ref().expect("buggy")
                != another_in.tout_script.as_ref().expect("buggy")
            {
                return true;
            }
        } else if this_in.tout_script.is_some() {
            return true;
        }

        false
    }

    fn get_wda_setts(gtrans_in: &GtransInput) -> Vec<WdaSett> {
        let mut wda_setts = vec![];

        // these opts gurantee to be not none
        let sett_gui: bool;
        let sett_socks5: &str;
        let sett_tout_page_load: u32;
        let sett_tout_script: u32;

        sett_gui = gtrans_in.gui.expect("buggy");

        sett_socks5 = gtrans_in.socks5.as_ref().unwrap();

        sett_tout_page_load = gtrans_in.tout_page_load.unwrap();

        sett_tout_script = gtrans_in.tout_script.unwrap();

        //
        if !sett_gui {
            wda_setts.push(WdaSett::NoGui);
        }

        if comm::is_valid_socks5(&sett_socks5) {
            wda_setts.push(WdaSett::PrepareUseSocksProxy(Cow::from(sett_socks5)));
            wda_setts.push(WdaSett::Socks5Proxy(Cow::from(sett_socks5)));
            wda_setts.push(WdaSett::ProxyDnsSocks5);
        }
        wda_setts.push(WdaSett::PageLoadTimeout(sett_tout_page_load));
        wda_setts.push(WdaSett::ScriptTimeout(sett_tout_script));

        dbgg!(&wda_setts);

        wda_setts
    }

    pub fn new<'a>(
        mafad: &'a MafaData,
        ntf: Arc<Mutex<EventNotifier>>,
        mafa_in: &MafaInput,
        gtrans_in: GtransInput,
    ) -> Result<GtransClient<'a>> {
        let merged_in = GtransInput::merge(gtrans_in, mafa_in)?;
        dbgg!(&merged_in);

        if !is_valid_words(&merged_in.words) && !merged_in.list_lang {
            return Err(MafaError::InvalidWords);
        }

        let wda_setts = Self::get_wda_setts(&merged_in);

        dbgg!(&wda_setts);

        let wda: Option<WebDrvAstn<GeckoDriver>>;
        match WebDrvAstn::<GeckoDriver>::new(wda_setts) {
            Ok(ret) => wda = Some(ret),
            Err(err_wda) => match err_wda {
                WdaError::WdcNotReady(WdcError::BadDrvCmd(err, msg), _) => {
                    if msg.contains("socksProxy is not a valid URL") {
                        return Err(MafaError::InvalidSocks5Proxy);
                    } else {
                        dbgg!(123);
                        return Err(MafaError::WebDrvCmdRejected(err, msg));
                    }
                }
                _ => {
                    return Err(MafaError::UnexpectedWda(err_wda));
                }
            },
        };

        if wda.is_none() {
            return Err(MafaError::BugFound(2345));
        }

        let wda = wda.unwrap();

        Ok(GtransClient {
            mafad,
            ntf,
            input: merged_in,
            wda,
            upaths: Default::default(),
        })
    }

    fn upaths_locate(
        &self,
        en_words: &str,
        tc_words: &str,
        wait_before_extract: u64,
    ) -> Result<Vec<u8>> {
        let url = format!(
            "https://translate.google.com/?sl=en&tl=zh-TW&text={}&op=translate",
            en_words
        );

        if let Err(err_navi) = self.wda.go_url(&url) {
            if let WdaError::WdcFail(WdcError::BadDrvCmd(err, msg)) = err_navi {
                dbgg!(123);
                return Err(MafaError::WebDrvCmdRejected(err, msg));
            } else {
                return Err(MafaError::UnexpectedWda(err_navi));
            }
        }

        sleep(Duration::from_millis(wait_before_extract));

        // script/gtrans-upath.js
        let js_in="console.log=function(){};function locate_elem(e){var o=[];function l(e,n,t){let c=e.childNodes.length;for(let d=0;d<c;d++){let c=e.childNodes[d];if(c.innerText&&c.innerText==n){console.log('yes',c);o=[...t,d]}else{l(c,n,[...t,d])}}}let n=e;l(document.body,n,[]);console.log(o);let t=o.map((()=>document.body));console.log(t);for(let e=0;e<o.length;e++){for(let l=0;l<o[e].length;l++){t[e]=t[e].childNodes[o[e][l]]}}return o}return locate_elem(arguments[0]);";

        let js_out;
        match self.wda.eval(&js_in, vec![tc_words]) {
            Ok(ret) => js_out = ret,
            Err(err_eval) => {
                if let WdaError::WdcFail(WdcError::BadDrvCmd(err, msg)) = err_eval {
                    dbgg!(123);
                    return Err(MafaError::WebDrvCmdRejected(err, msg));
                } else {
                    return Err(MafaError::UnexpectedWda(err_eval));
                }
            }
        }

        dbgg!(&js_out);

        let obj_out = serde_json::from_str::<Vec<u8>>(&js_out).unwrap();

        Ok(obj_out)
    }

    fn notify(&self, ev: MafaEvent) -> Result<()> {
        self.ntf
            .lock()
            .map_err(|_| MafaError::BugFound(7890))?
            .notify(ev);

        Ok(())
    }

    fn refresh_upath(&mut self, rebuild_cache: bool) -> Result<()> {
        if !rebuild_cache {
            let caches_from_files =
                UpathCache::from_pbuf(self.mafad.pathto_exist_cache("gtrans")?)?;
            self.upaths = caches_from_files.0;
            return Ok(());
        }

        let mut upath1: Option<Vec<u8>> = None;
        let mut upath2: Option<Vec<u8>> = None;

        // let mut time_before = 50; // test purpose
        let mut time_before = 500; // in millis

        // let mut try_times = 3; // test purpose
        let mut try_times = 5;

        while try_times > 0 {
            if upath1.is_none() {
                match self.upaths_locate("OMG", "\"我的天啊\"", time_before) {
                    Ok(ret) => {
                        if ret.len() > 0 {
                            upath1 = Some(ret)
                        }
                    }
                    Err(err_loc) => match err_loc {
                        MafaError::WebDrvCmdRejected(ref err, _) => {
                            // only retry on timeout
                            if err.contains("timeout") {
                                dbgmsg!("upath1 timeout");
                            } else {
                                return Err(err_loc);
                            }
                        }
                        _ => return Err(err_loc),
                    },
                }
            }

            if upath2.is_none() {
                match self.upaths_locate("ASAP", "\"盡快\"", time_before) {
                    Ok(ret) => {
                        if ret.len() > 0 {
                            upath2 = Some(ret)
                        }
                    }
                    Err(err_loc) => match err_loc {
                        MafaError::WebDrvCmdRejected(ref err, _) => {
                            // only retry on timeout
                            if err.contains("timeout") {
                                dbgmsg!("upath2 timeout");
                            } else {
                                return Err(err_loc);
                            }
                        }
                        _ => return Err(err_loc),
                    },
                }
            }

            if upath1.is_some() && upath2.is_some() {
                self.notify(MafaEvent::CacheRetry {
                    cate: Category::Gtrans,
                    is_fin: true,
                })?;
                break;
            } else {
                self.notify(MafaEvent::CacheRetry {
                    cate: Category::Gtrans,
                    is_fin: false,
                })?;
                try_times -= 1;
                time_before += time_before;
                dbgmsg!("need retry {} {}", try_times, time_before);
            }
        }

        dbgg!(&try_times);

        if upath1.is_none() || upath2.is_none() {
            return Err(MafaError::CacheRebuildFail(
                CacheRebuildFailKind::UpathNotFound,
            ));
        }

        let upath1 = upath1.expect("buggy");
        let upath2 = upath2.expect("buggy");

        let sig_upath_len = upath1.len();
        if upath2.len() != sig_upath_len {
            return Err(MafaError::CacheRebuildFail(
                CacheRebuildFailKind::UpathLenNotMatched,
            ));
        }

        for i in 0..sig_upath_len {
            if upath1[i] != upath2[i] {
                return Err(MafaError::CacheRebuildFail(
                    CacheRebuildFailKind::UpathValNotMatched,
                ));
            }
        }

        let u_part = serde_json::to_string(&upath1).unwrap();
        let comb = format!("{}\n", u_part);
        dbgg!(&comb);

        self.mafad
            .cache_append("gtrans", &comb, &format!("{}-", &comb))?;

        self.upaths.push(Upath(upath1));

        Ok(())
    }

    fn cache_on_gh(&self, url: &str) -> Result<String> {
        match self.wda.go_url(url) {
            Ok(_) => {}
            Err(err_navi) => {
                if let WdaError::WdcFail(WdcError::BadDrvCmd(err, msg)) = err_navi {
                    dbgg!(123);
                    return Err(MafaError::WebDrvCmdRejected(err, msg));
                } else {
                    return Err(MafaError::UnexpectedWda(err_navi));
                }
            }
        }

        let jsout: String;
        let jsin = "return document.getElementsByTagName('pre')[0].innerText;";

        match self.wda.eval(&jsin, vec![]) {
            Ok(ret) => jsout = ret,
            Err(err_eval) => {
                if let WdaError::WdcFail(WdcError::BadDrvCmd(err, msg)) = err_eval {
                    dbgg!(123);
                    return Err(MafaError::WebDrvCmdRejected(err, msg));
                } else {
                    return Err(MafaError::UnexpectedWda(err_eval));
                }
            }
        }

        // let jsout = jsout.replace("\"", "").replace("\\n", "\n");
        let jsout = jsout.replace('"', "").replace("\\n", "\n");

        dbgg!(&jsout);

        Ok(jsout)
    }

    fn try_rebuild_cache(&mut self) -> Result<()> {
        let mut rebuild_cache = false;

        if let CacheMechanism::Remote = self.input.cachm {
            let remote_data = self.cache_on_gh(
                "https://raw.githubusercontent.com/imichael2e2/mafa-cache/master/gtrans",
            )?;

            self.mafad.init_cache("gtrans", &remote_data)?;
        } else if let CacheMechanism::Local = self.input.cachm {
            self.mafad.try_init_cache(
                "gtrans",
                "[4,0,1,0,1,0,1,1,2,1,1,9,0,2,0,0,1]\n[4,0,1,0,1,0,1,1,2,1,1,9,0,3,0,0,1]\n-",
            )?;
        } else if let CacheMechanism::No = self.input.cachm {
            rebuild_cache = true;
        }

        if rebuild_cache {
            self.notify(MafaEvent::BuildCache {
                cate: Category::Gtrans,
                is_fin: false,
            })?;
            self.refresh_upath(true)?;
            self.notify(MafaEvent::BuildCache {
                cate: Category::Gtrans,
                is_fin: true,
            })?;
        } else {
            self.refresh_upath(false)?;
        }

        Ok(())
    }

    ///
    /// Returned `String` is pretty-printed.
    pub fn handle(&mut self, pred_caches: Option<Vec<Vec<u8>>>) -> Result<(EurKind, String)> {
        if self.input.list_lang {
            return Ok((
                EurKind::GtransAllLang,
                GtransClient::list_all_lang().to_string(),
            ));
        }

        if pred_caches.is_none() {
            self.try_rebuild_cache()?;
        } else {
            let pred_caches = pred_caches.ok_or(MafaError::BugFound(4567))?;
            pred_caches
                .iter()
                .for_each(|v| self.upaths.push(Upath(v.clone())));
        }

        if self.upaths.len() == 0 {
            panic!("buggy");
        }

        let source_lang = &self.input.src_lang;
        let target_lang = &self.input.tgt_lang;

        let orig_words = &self.input.words;

        self.notify(MafaEvent::FetchResult {
            cate: Category::Gtrans,
            is_fin: false,
        })?;
        let translated = self.fetch(orig_words, source_lang, target_lang)?;
        self.notify(MafaEvent::FetchResult {
            cate: Category::Gtrans,
            is_fin: true,
        })?;

        let gtrans_res = GtransResult::from_str(source_lang, target_lang, orig_words, &translated)?;
        dbgg!(&gtrans_res);

        Ok((
            EurKind::GtransResult,
            gtrans_res.pretty_print(
                self.input.nocolor.is_some(),
                self.input.ascii,
                self.input.wrap_width,
            )?,
        ))
    }

    fn fetch(&self, orig_words: &str, sl: &str, tl: &str) -> Result<String> {
        let mut url = String::from("");
        url += &format!("https://translate.google.com/?sl={}&tl={}&text=", sl, tl);
        url += &String::from_utf8_lossy(&comm::percent_encode(orig_words.as_bytes()));
        url += "&op=translate";

        let mut translate_res = "???".to_string();

        dbgg!(&self.upaths);
        let mut upaths_i = 0;
        let upaths_len = self.upaths.len();

        // script/gtrans-transres.js
        let js_get_innertxt = "console.log=function(){};var send_back=arguments[arguments.length-1];var upath=arguments[0];clearInterval(window['gtrans-res']);window['gtrans-res']=setInterval((function(){var e=document.body;if(upath.length>0){for(let n=0;n<upath.length;n++){if(e==undefined){console.log(n);return}else{console.log(123)}e=e.childNodes[upath[n]]}console.log(e);send_back(e.innerText);clearInterval(window['gtrans-res'])}else{console.log(upath)}}),500);";

        let mut is_url_reached = false;

        // try_times = go_url + eval_js
        let mut try_times = 5; // sufficient to let try again succeed

        // let mut wait_before = 500;
        let mut wait_before = 100;

        while try_times > 0 {
            if let Err(err_navi) = self.wda.go_url(&url) {
                if let WdaError::WdcFail(WdcError::BadDrvCmd(err, msg)) = err_navi {
                    if err.contains("timeout") {
                        self.notify(MafaEvent::ConnectTimeoutRetry {
                            cate: Category::Gtrans,
                            is_fin: false,
                        })?;
                        try_times -= 1;
                        continue;
                    } else {
                        return Err(MafaError::WebDrvCmdRejected(err, msg));
                    }
                } else {
                    return Err(MafaError::UnexpectedWda(err_navi));
                }
            }

            is_url_reached = true;
            self.notify(MafaEvent::ConnectTimeoutRetry {
                cate: Category::Gtrans,
                is_fin: true,
            })?;

            let upath_curr = &self.upaths[upaths_i].0;
            let arg0_detect_err = serde_json::to_string(&upath_curr[..]).unwrap();

            sleep(Duration::from_millis(wait_before));

            match self
                .wda
                .eval_async(&js_get_innertxt, vec![&arg0_detect_err])
            {
                Ok(retstr) => {
                    if retstr.contains("Try again") {
                        dbgg!(&retstr);
                        self.notify(MafaEvent::SrvTempUnavRetry {
                            cate: Category::Gtrans,
                            is_fin: false,
                        })?;
                    } else {
                        dbgg!(&retstr);
                        // std::thread::sleep(std::time::Duration::from_secs(100));
                        translate_res = retstr;
                        self.notify(MafaEvent::SrvTempUnavRetry {
                            cate: Category::Gtrans,
                            is_fin: true,
                        })?;
                        break;
                    }
                }

                Err(err_eval) => {
                    if let WdaError::WdcFail(WdcError::BadDrvCmd(err, msg)) = err_eval {
                        if err.contains("timeout") {
                            upaths_i += 1;
                            if upaths_i < upaths_len {
                                self.notify(MafaEvent::TryNextCache {
                                    cate: Category::Gtrans,
                                    is_fin: false,
                                })?;
                                // continue;
                            } else {
                                self.notify(MafaEvent::TryNextCache {
                                    cate: Category::Gtrans,
                                    is_fin: true,
                                })?;
                                return Err(MafaError::AllCachesInvalid);
                            }
                        } else {
                            return Err(MafaError::WebDrvCmdRejected(err, msg));
                        }
                    } else {
                        return Err(MafaError::UnexpectedWda(err_eval));
                    }
                }
            }

            try_times -= 1;
            wait_before += wait_before;
            dbgmsg!("need retry {} {}", try_times, wait_before);
        }

        self.notify(MafaEvent::TryNextCache {
            cate: Category::Gtrans,
            is_fin: true,
        })?;

        if !is_url_reached {
            return Err(MafaError::DataFetchedNotReachable);
        }

        Ok(translate_res)
    }

    //

    pub fn list_all_lang() -> &'static str {
        r"All languages supported by Google Translate (<Language>: <code>):

Detect language: auto
Afrikaans: af
Albanian: sq
Amharic: am
Arabic: ar
Armenian: hy
Assamese: as
Aymara: ay
Azerbaijani: az
Bambara: bm
Basque: eu
Belarusian: be
Bengali: bn
Bhojpuri: bho
Bosnian: bs
Bulgarian: bg
Catalan: ca
Cebuano: ceb
Chichewa: ny
Chinese (Simplified): zh-CN
Chinese (Traditional): zh-TW
Corsican: co
Croatian: hr
Czech: cs
Danish: da
Dhivehi: dv
Dogri: doi
Dutch: nl
English: en
Esperanto: eo
Estonian: et
Ewe: ee
Filipino: tl
Finnish: fi
French: fr
Frisian: fy
Galician: gl
Georgian: ka
German: de
Greek: el
Guarani: gn
Gujarati: gu
Haitian Creole: ht
Hausa: ha
Hawaiian: haw
Hebrew: iw
Hindi: hi
Hmong: hmn
Hungarian: hu
Icelandic: is
Igbo: ig
Ilocano: ilo
Indonesian: id
Irish: ga
Italian: it
Japanese: ja
Javanese: jw
Kannada: kn
Kazakh: kk
Khmer: km
Kinyarwanda: rw
Konkani: gom
Korean: ko
Krio: kri
Kurdish (Kurmanji): ku
Kurdish (Sorani): ckb
Kyrgyz: ky
Lao: lo
Latin: la
Latvian: lv
Lingala: ln
Lithuanian: lt
Luganda: lg
Luxembourgish: lb
Macedonian: mk
Maithili: mai
Malagasy: mg
Malay: ms
Malayalam: ml
Maltese: mt
Maori: mi
Marathi: mr
Meiteilon (Manipuri): mni-Mtei
Mizo: lus
Mongolian: mn
Myanmar (Burmese): my
Nepali: ne
Norwegian: no
Odia (Oriya): or
Oromo: om
Pashto: ps
Persian: fa
Polish: pl
Portuguese: pt
Punjabi: pa
Quechua: qu
Romanian: ro
Russian: ru
Samoan: sm
Sanskrit: sa
Scots Gaelic: gd
Sepedi: nso
Serbian: sr
Sesotho: st
Shona: sn
Sindhi: sd
Sinhala: si
Slovak: sk
Slovenian: sl
Somali: so
Spanish: es
Sundanese: su
Swahili: sw
Swedish: sv
Tajik: tg
Tamil: ta
Tatar: tt
Telugu: te
Thai: th
Tigrinya: ti
Tsonga: ts
Turkish: tr
Turkmen: tk
Twi: ak
Ukrainian: uk
Urdu: ur
Uyghur: ug
Uzbek: uz
Vietnamese: vi
Welsh: cy
Xhosa: xh
Yiddish: yi
Yoruba: yo
Zulu: zu"
    }
}

#[derive(Debug, Default)]
struct GtransResult<'a, 'b> {
    sl: Cow<'a, str>,
    tl: Cow<'a, str>,
    orig_words: Cow<'a, str>,
    trans_words: Cow<'b, str>,
    trans_pronun: Cow<'b, str>,
}

impl<'a, 'b> GtransResult<'a, 'b> {
    fn from_str(
        sl: &'a str,
        tl: &'a str,
        orig_words: &'a str,
        trans_result: &'b str,
    ) -> Result<Self> {
        dbgg!(trans_result);

        let mut begi_trans = 0;
        let mut endi_trans = 0;

        if trans_result.len() > 2 {
            if trans_result.as_bytes()[0] == b'"' {
                begi_trans = 1;
            }
            if trans_result.as_bytes()[trans_result.len() - 1] == b'"' {
                endi_trans = trans_result.len() - 1;
            }
        }

        Ok(GtransResult {
            sl: sl.into(),
            tl: tl.into(),
            trans_words: Cow::Borrowed(&trans_result[begi_trans..endi_trans]),
            trans_pronun: Cow::Borrowed(&trans_result[0..0]),
            orig_words: Cow::Borrowed(orig_words),
        })
    }

    fn pretty_print(&self, nocolor: bool, asciiful: bool, wrap_width: u16) -> Result<String> {
        let wrap_width: usize = if wrap_width > 17 {
            wrap_width.into()
        } else {
            80
        };

        let header_part = if asciiful {
            format!(" Result |")
        } else {
            format!(" Result │")
        };

        let header_part_colorful = if asciiful {
            format!(" \x1b[36;1mResult\x1b[0m |")
        } else {
            format!(" \x1b[36;1mResult\x1b[0m │")
        };

        // dbgg!(&header_part);

        let cols_header_part = UnicodeWidthStr::width(header_part.as_str());

        let mut output = String::from("");

        // 0 for top, 1 for bottom
        let line_comp = if asciiful { "-" } else { "─" };
        let line_tail_comp = if asciiful { ("-", "-") } else { ("╮", "┴") };

        let cols_line_comp = UnicodeWidthStr::width(line_comp);
        let rtimes_line_comp = (cols_header_part / cols_line_comp) - 1;

        // top line
        let top_line = comm::replicate(line_comp, rtimes_line_comp);
        output += &top_line;
        output += line_tail_comp.0;
        output += "\n";

        output += if nocolor {
            &header_part
        } else {
            &header_part_colorful
        };
        output += "\n";

        // bottom line
        let bottom_line = comm::replicate(line_comp, rtimes_line_comp);
        output += &bottom_line;
        output += line_tail_comp.1;
        output += &comm::replicate(
            line_comp,
            (wrap_width
                - cols_line_comp * rtimes_line_comp
                - UnicodeWidthStr::width(line_tail_comp.1))
                / cols_line_comp,
        ); // bottom needs extra line_comp to reach 80
        output += "\n";

        // orig line
        let orig_line_hdr = if asciiful {
            "     Words     | "
        } else {
            "     Words     │ "
        };
        let wrap_append = if asciiful {
            "               | "
        } else {
            "               │ "
        };

        let w_orig_line_hdr = UnicodeWidthStr::width(orig_line_hdr);
        let w_orig_line_words = wrap_width - w_orig_line_hdr;

        let mut wrapper =
            bwrap::EasyWrapper::new(&self.orig_words, w_orig_line_words as usize).unwrap();
        let orig_line_words = if is_spc_delim(&self.sl) {
            wrapper
                .wrap_use_style(bwrap::WrapStyle::NoBreakAppend(
                    wrap_append,
                    bwrap::ExistNlPref::KeepTrailSpc,
                ))
                .unwrap()
        } else {
            wrapper
                .wrap_use_style(bwrap::WrapStyle::MayBreakAppend(wrap_append))
                .unwrap()
        };

        output += &format!("{}{}\n", orig_line_hdr, orig_line_words);

        // trans line
        let trans_line_hdr = if asciiful {
            "  Translation  | "
        } else {
            "  Translation  │ "
        };
        let w_trans_line_hdr = UnicodeWidthStr::width(trans_line_hdr);
        let w_trans_line_words = wrap_width - w_trans_line_hdr;

        let mut wrapper =
            bwrap::EasyWrapper::new(&self.trans_words, w_trans_line_words as usize).unwrap();
        let trans_line_words = if is_spc_delim(&self.tl) {
            wrapper
                .wrap_use_style(bwrap::WrapStyle::NoBreakAppend(
                    wrap_append,
                    bwrap::ExistNlPref::KeepTrailSpc,
                ))
                .unwrap()
        } else {
            wrapper
                .wrap_use_style(bwrap::WrapStyle::MayBreakAppend(wrap_append))
                .unwrap()
        };

        output += &format!("\n{}{}\n", trans_line_hdr, trans_line_words);

        // pron line
        let pron_line_hdr = if asciiful {
            " Pronunciation | "
        } else {
            " Pronunciation │ "
        };
        let w_pron_line_hdr = UnicodeWidthStr::width(pron_line_hdr);
        let w_pron_line_words = wrap_width - w_pron_line_hdr;

        if self.trans_pronun.trim().len() > 0 {
            let mut wrapper =
                bwrap::EasyWrapper::new(&self.trans_pronun.trim(), w_pron_line_words as usize)
                    .unwrap();
            let pron_line_words = wrapper
                .wrap_use_style(bwrap::WrapStyle::MayBreakAppend(wrap_append))
                .unwrap();

            output += &format!("\n{}{}\n", pron_line_hdr, pron_line_words);
        }

        Ok(output)
    }
}

fn is_spc_delim(lk: &str) -> bool {
    match lk {
        "de" | "en" | "fr" | "ru" => true,
        _ => false,
    }
}

fn is_valid_lang(lang_kind: &str) -> bool {
    match lang_kind {
        "auto" | "af" | "sq" | "am" | "ar" | "hy" | "as" | "ay" | "az" | "bm" | "eu" | "be"
        | "bn" | "bho" | "bs" | "bg" | "ca" | "ceb" | "ny" | "zh-CN" | "zh-TW" | "co" | "hr"
        | "cs" | "da" | "dv" | "doi" | "nl" | "en" | "eo" | "et" | "ee" | "tl" | "fi" | "fr"
        | "fy" | "gl" | "ka" | "de" | "el" | "gn" | "gu" | "ht" | "ha" | "haw" | "iw" | "hi"
        | "hmn" | "hu" | "is" | "ig" | "ilo" | "id" | "ga" | "it" | "ja" | "jw" | "kn" | "kk"
        | "km" | "rw" | "gom" | "ko" | "kri" | "ku" | "ckb" | "ky" | "lo" | "la" | "lv" | "ln"
        | "lt" | "lg" | "lb" | "mk" | "mai" | "mg" | "ms" | "ml" | "mt" | "mi" | "mr"
        | "mni-Mtei" | "lus" | "mn" | "my" | "ne" | "no" | "or" | "om" | "ps" | "fa" | "pl"
        | "pt" | "pa" | "qu" | "ro" | "ru" | "sm" | "sa" | "gd" | "nso" | "sr" | "st" | "sn"
        | "sd" | "si" | "sk" | "sl" | "so" | "es" | "su" | "sw" | "sv" | "tg" | "ta" | "tt"
        | "te" | "th" | "ti" | "ts" | "tr" | "tk" | "ak" | "uk" | "ur" | "ug" | "uz" | "vi"
        | "cy" | "xh" | "yi" | "yo" | "zu" => true,
        _ => false,
    }
}

fn is_valid_words(v: &str) -> bool {
    if v.len() > 0 {
        true
    } else {
        false
    }
}

#[cfg(test)]
mod utst_merged {
    use super::*;

    #[test]
    fn words_1() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "gtrans", "hello", "world"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");

                    assert_eq!(merged_in.words, "hello world");
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn words_2() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "gtrans", ""])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(_mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m);
                    if let Err(MafaError::InvalidWords) = gtrans_in {
                    } else {
                        assert!(false);
                    }
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn default() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "gtrans", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");
                    let wda_setts = GtransClient::get_wda_setts(&merged_in);

                    assert!(wda_setts.contains(&WdaSett::NoGui));
                    assert!(wda_setts.contains(&WdaSett::PageLoadTimeout(30000)));
                    assert!(wda_setts.contains(&WdaSett::ScriptTimeout(30000)));
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn silent_1() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "gtrans", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");

                    assert_eq!(merged_in.silent, None);
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn silent_2() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "--silent", "gtrans", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");

                    assert_eq!(merged_in.silent, Some(true));
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn silent_3() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "gtrans", "--silent", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");

                    assert_eq!(merged_in.silent, Some(true));
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn silent_4() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "--silent", "gtrans", "--silent", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");

                    assert_eq!(merged_in.silent, Some(true));
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn nocolor_1() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "gtrans", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");

                    assert_eq!(merged_in.nocolor, None);
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn nocolor_2() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "--nocolor", "gtrans", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");

                    assert_eq!(merged_in.nocolor, Some(true));
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn nocolor_3() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "gtrans", "--nocolor", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");

                    assert_eq!(merged_in.nocolor, Some(true));
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn nocolor_4() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "--nocolor", "gtrans", "--nocolor", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");

                    assert_eq!(merged_in.nocolor, Some(true));
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn imode_nocolor_1() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "i"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("i", _)) => {
                    let args = vec!["gtrans", "hello"];
                    let gtrans_in = GtransInput::from_i_mode(&mafa_in, args).expect("must ok");

                    assert_eq!(gtrans_in.nocolor, None);
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn imode_nocolor_2() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "i"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("i", _)) => {
                    let args = vec!["gtrans", "hello"];
                    let gtrans_in = GtransInput::from_i_mode(&mafa_in, args).expect("must ok");

                    assert_eq!(gtrans_in.nocolor, None);

                    let args = vec!["gtrans", "--nocolor", "hello"];
                    let gtrans_in = GtransInput::from_i_mode(&mafa_in, args).expect("must ok");

                    assert_eq!(gtrans_in.nocolor, Some(true));

                    let args = vec!["gtrans", "hello"];
                    let gtrans_in = GtransInput::from_i_mode(&mafa_in, args).expect("must ok");

                    assert_eq!(gtrans_in.nocolor, None);
                }

                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn imode_nocolor_3() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "--nocolor", "i"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("i", _)) => {
                    let args = vec!["gtrans", "hello"];
                    let gtrans_in = GtransInput::from_i_mode(&mafa_in, args).expect("must ok");

                    assert_eq!(gtrans_in.nocolor, Some(true));

                    let args = vec!["gtrans", "--nocolor", "hello"];
                    let gtrans_in = GtransInput::from_i_mode(&mafa_in, args).expect("must ok");

                    assert_eq!(gtrans_in.nocolor, Some(true));

                    let args = vec!["gtrans", "hello"];
                    let gtrans_in = GtransInput::from_i_mode(&mafa_in, args).expect("must ok");

                    assert_eq!(gtrans_in.nocolor, Some(true));
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn socks5_1() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "gtrans", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");
                    let wda_setts = GtransClient::get_wda_setts(&merged_in);

                    assert_eq!(mafa_in.socks5, "");
                    assert!(!wda_setts.contains(&WdaSett::Socks5Proxy(Cow::Borrowed(""))));
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn socks5_2() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec![
                "mafa",
                "--socks5",
                "127.0.0.1:1080",
                "gtrans",
                "hello",
            ])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");
                    let wda_setts = GtransClient::get_wda_setts(&merged_in);

                    assert_eq!(mafa_in.socks5, "127.0.0.1:1080");
                    assert!(
                        wda_setts.contains(&WdaSett::Socks5Proxy(Cow::Borrowed("127.0.0.1:1080")))
                    );
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn socks5_3() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec![
                "mafa",
                "gtrans",
                "--socks5",
                "127.0.0.1:1080",
                "hello",
            ])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");
                    let wda_setts = GtransClient::get_wda_setts(&merged_in);

                    assert_eq!(mafa_in.socks5, "");
                    assert!(
                        wda_setts.contains(&WdaSett::Socks5Proxy(Cow::Borrowed("127.0.0.1:1080")))
                    );
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn socks5_4() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec![
                "mafa",
                "--socks5",
                "127.0.0.1:1080",
                "gtrans",
                "--socks5",
                "127.0.0.1:1081",
                "hello",
            ])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");
                    let wda_setts = GtransClient::get_wda_setts(&merged_in);

                    assert_eq!(mafa_in.socks5, "127.0.0.1:1080");
                    assert!(
                        wda_setts.contains(&WdaSett::Socks5Proxy(Cow::Borrowed("127.0.0.1:1081")))
                    );
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn socks5_5() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec![
                "mafa",
                "--socks5",
                "127.0.0.1:1081",
                "gtrans",
                "--socks5",
                "127.0.0.1:1080",
                "hello",
            ])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");
                    let wda_setts = GtransClient::get_wda_setts(&merged_in);

                    assert_eq!(mafa_in.socks5, "127.0.0.1:1081");
                    assert!(
                        wda_setts.contains(&WdaSett::Socks5Proxy(Cow::Borrowed("127.0.0.1:1080")))
                    );
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn pageload_1() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec![
                "mafa",
                "--timeout-pageload",
                "1234",
                "gtrans",
                "hello",
            ])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    assert_eq!(gtrans_in.tout_page_load, None);

                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");
                    let wda_setts = GtransClient::get_wda_setts(&merged_in);

                    assert_eq!(mafa_in.tout_page_load, 1234);
                    assert!(wda_setts.contains(&WdaSett::PageLoadTimeout(1234)));
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn pageload_2() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec![
                "mafa",
                "--timeout-pageload",
                "1234",
                "gtrans",
                "--timeout-pageload",
                "6789",
                "hello",
            ])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    assert_eq!(gtrans_in.tout_page_load, Some(6789));

                    let merged_in = GtransInput::merge(gtrans_in, &mafa_in).expect("must ok");
                    let wda_setts = GtransClient::get_wda_setts(&merged_in);

                    assert_eq!(mafa_in.tout_page_load, 1234);
                    assert!(wda_setts.contains(&WdaSett::PageLoadTimeout(6789)));
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn lang_1() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "gtrans", "--sl", "xxx", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(_mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => match GtransInput::from_ca_matched(sub_m) {
                    Err(err_match) => match err_match {
                        MafaError::InvalidSourceLang => {}
                        _ => assert!(false),
                    },
                    _ => assert!(false),
                },
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn lang_2() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "gtrans", "--tl", "xxx", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(_mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => match GtransInput::from_ca_matched(sub_m) {
                    Err(err_match) => match err_match {
                        MafaError::InvalidTargetLang => {}
                        _ => assert!(false),
                    },
                    _ => assert!(false),
                },

                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }
}
