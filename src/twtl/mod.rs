// Copyright (C) 2023 Michael Lee <micl2e2@proton.me>
//
// Licensed under the GNU General Public License, Version 3.0 or any later
// version <LICENSE-GPL or https://www.gnu.org/licenses/gpl-3.0.txt>.
//
// This file may not be copied, modified, or distributed except in compliance
// with the license.
//

use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::mafadata::MafaData;
use crate::MafaClient;

use wda::BasicAutomation;
use wda::GeckoDriver;
use wda::WdaError as WdaErr;
use wda::WdaSett;
use wda::WdcError as WdcErr;
use wda::WebDrvAstn;

use crate::error::MafaError;
use crate::error::Result;

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

mod twov;
use twov::TweetOverview;

#[derive(Debug, Default, Copy, Clone)]
pub enum SaveFormat {
    #[default]
    Json,
    Xml,
}

impl SaveFormat {
    fn from_str(s: &str) -> Self {
        match s {
            "JSON" | "json" => Self::Json,
            "XML" | "xml" => Self::Xml,
            _ => Self::Json,
        }
    }
}

#[derive(Debug, Default)]
pub struct TwtlInput {
    imode: bool, // no from clap
    username: String,
    ntweets: u16,
    save_to: Option<PathBuf>,
    save_format: Option<SaveFormat>,
    try_login: bool,
}

impl TwtlInput {
    pub fn from_ca_matched(ca_matched: &ClapArgMatches) -> Result<Self> {
        let mut twtl_in = TwtlInput::default();

        // username
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::Username::id()) {
            if !is_valid_username(optval) {
                return Err(MafaError::InvalidTwitterUsername);
            }
            twtl_in.username = normalized_username(optval);
        }

        // ntweets
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::NumTweets::id()) {
            let intval =
                u16::from_str_radix(&optval, 10).map_err(|_| MafaError::InvalidNumTweets)?;
            twtl_in.ntweets = intval;
        }

        // save-format
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::SaveFormat::id()) {
            twtl_in.save_format = Some(SaveFormat::from_str(optval));
        }

        // save-to
        if let Ok(Some(optval)) = ca_matched.try_get_one::<String>(opts::SaveTo::id()) {
            twtl_in.save_to = Some(ensure_save_to(optval, twtl_in.save_format)?);
        }

        // login
        if ca_matched.get_flag(opts::TryLogin::id()) {
            twtl_in.try_login = true;
        }

        dbgg!(&twtl_in);

        Ok(twtl_in)
    }

    pub fn from_i_mode2(args: Vec<&str>) -> Result<TwtlInput> {
        let cmd_twtl = get_cmd();

        let m = cmd_twtl.try_get_matches_from(args);

        match m {
            Ok(ca_matched) => {
                let mut twtl_in = TwtlInput::from_ca_matched(&ca_matched)?;

                twtl_in.imode = true;

                Ok(twtl_in)
            }
            // this will print helper
            Err(err_match) => Err(MafaError::ClapMatchError(err_match.render())),
        }
    }
}

// opts //

pub mod opts {
    use core::ops::Range;

    pub struct Username;
    impl Username {
        #[inline]
        pub fn id() -> &'static str {
            "USERNAME"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn helper() -> &'static str {
            "A valid Twitter username"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = "A valid Twitter username

For example, the valid usernames for Twitter official account are: `@twitter`, `twitter`, `@Twitter` or `Twitter`(quotes excluded).";

            let mut af_buf = [0u8; 256];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct NumTweets;
    impl NumTweets {
        #[inline]
        pub fn id() -> &'static str {
            "NUMTWEETS"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "ntweets"
        }
        #[inline]
        pub fn shortopt() -> char {
            'n'
        }
        #[inline]
        pub fn def_val() -> &'static str {
            "10"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Number of tweets being fetched"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = "Number of tweets being fetched

ATTENTION: The maximum is 800. Due to restricted environment of Twitter website, any value larger than 800 will fallback to 800, and this value will likely change in the future.";

            let mut af_buf = [0u8; 256];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct SaveTo;
    impl SaveTo {
        #[inline]
        pub fn id() -> &'static str {
            "SAVE_TO"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "save-to"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "The file to save the data"
        }
        #[inline]
        pub fn long_helper() -> String {
            let bf = r#"The file to save the data

Any path is allowed as long as it is a valid path on disk, and file extension is not significant.

NOTE: in case of any non-existing directory found, this will fallback to "./twtl-saved.<SAVE_FORMAT>"."#;
            let mut af_buf = [0u8; 256];

            let rl = bwrap::Wrapper::new(bf, 70, &mut af_buf)
                .unwrap()
                .wrap()
                .unwrap();

            String::from_utf8_lossy(&af_buf[0..rl]).to_string()
        }
    }

    pub struct SaveFormat;
    impl SaveFormat {
        #[inline]
        pub fn id() -> &'static str {
            "SAVE_FORMAT"
        }
        #[inline]
        pub fn n_args() -> Range<usize> {
            1..2
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "save-format"
        }
        #[inline]
        pub fn def_val() -> &'static str {
            "json"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Format of saved data"
        }
        #[inline]
        pub fn long_helper() -> &'static str {
            r#"Format of saved data

Available values are: json, xml."#
        }
    }

    pub struct TryLogin;
    impl TryLogin {
        #[inline]
        pub fn id() -> &'static str {
            "TRYLOGIN"
        }
        #[inline]
        pub fn longopt() -> &'static str {
            "login"
        }
        #[inline]
        pub fn helper() -> &'static str {
            "Login or logout Twitter account"
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
            "Enable silent mode                                      "
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
    let opt_username = {
        type O = opts::Username;
        ClapArg::new(O::id())
            .required(true)
            .num_args(O::n_args())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_ntweets = {
        type O = opts::NumTweets;
        ClapArg::new(O::id())
            .long(O::longopt())
            .short(O::shortopt())
            .default_value(O::def_val())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_saveto = {
        type O = opts::SaveTo;
        ClapArg::new(O::id())
            .long(O::longopt())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_saveformat = {
        type O = opts::SaveFormat;
        ClapArg::new(O::id())
            .long(O::longopt())
            .default_value(O::def_val())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let opt_trylogin = {
        type O = opts::TryLogin;
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

    let cmd_twtl = ClapCommand::new("twtl")
        .about("Twitter users' timeline")
        .arg(opt_username)
        .arg(opt_ntweets)
        .arg(opt_saveto)
        .arg(opt_saveformat)
        .arg(opt_silient)
        .arg(opt_trylogin)
        .arg(opt_gui)
        .arg(opt_socks5)
        .arg(opt_tout_pageload)
        .arg(opt_tout_script);

    cmd_twtl
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct UlPath {
    upper_idx: Vec<u8>,
    lower_idx: Vec<u8>,
}

#[derive(Debug)]
struct UlpathCache(Vec<UlPath>);

impl UlpathCache {
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
            let deser = serde_json::from_slice::<Vec<Vec<u8>>>(ele).unwrap();
            if deser.len() != 2 {
                return Err(MafaError::CacheCorrupted);
            }
            let mut ulp = UlPath::default();
            ulp.upper_idx = deser[0].clone();
            ulp.lower_idx = deser[1].clone();
            ret.push(ulp);
        }

        Ok(Self(ret))
    }
}

///
/// it is ensured that the returned pbuf is completely valid,
/// if orig is not, give twtl-saved.json as default value.
fn ensure_save_to(s: &str, save_fmt: Option<SaveFormat>) -> Result<PathBuf> {
    let pbuf = PathBuf::from(s);

    let try_open = OpenOptions::new().create(true).write(true).open(&pbuf);

    let default_saved = if save_fmt.is_none() {
        "twtl-saved.json"
    } else {
        let save_fmt = save_fmt.unwrap();
        match save_fmt {
            SaveFormat::Json => "twtl-saved.json",
            SaveFormat::Xml => "twtl-saved.xml",
        }
    };

    match try_open {
        Ok(_) => {
            dbgmsg!("path good, whether file exist or not");
            Ok(pbuf)
        }
        Err(err_io) => match err_io.kind() {
            std::io::ErrorKind::NotFound => {
                dbgmsg!("path bad");
                Ok(PathBuf::from(default_saved))
            }
            _ => {
                dbgmsg!("io bad {:?}", err_io);
                Err(MafaError::BugFound(2348))
            }
        },
    }
}

fn is_valid_username(v: &str) -> bool {
    if v.len() > 0 {
        true
    } else {
        false
    }
}

fn normalized_username(v: &str) -> String {
    let vv = v.trim();

    let firstc = vv.bytes().nth(0).expect("buggy");
    let s = if firstc == b'@' {
        String::from(&vv[1..])
    } else {
        String::from(&vv[..])
    };

    s
}

impl<'a, 'b, 'c> MafaClient<'a, 'b, 'c, TwtlInput, UlPath> {
    fn notify(&self, ev: MafaEvent) -> Result<()> {
        self.ntf
            .lock()
            .map_err(|_| MafaError::BugFound(7890))?
            .notify(ev);

        Ok(())
    }

    fn refresh_ulpath(&mut self, rebuild_cache: bool) -> Result<()> {
        if !rebuild_cache {
            let caches_from_files = UlpathCache::from_pbuf(self.mafad.pathto_exist_cache("twtl")?)?;
            self.caches = caches_from_files.0;
            return Ok(());
        }

        let dest_url = "https://twitter.com/mafa_rs";

        match self.wda.go_url(dest_url) {
            Ok(_) => {}
            Err(err_navi) => {
                if let WdaErr::WdcFail(WdcErr::BadDrvCmd(err, msg)) = err_navi {
                    return Err(MafaError::WebDrvCmdRejected(err, msg));
                } else {
                    return Err(MafaError::UnexpectedWda(err_navi));
                }
            }
        }

        // sleep(Duration::from_secs(1000)); // test purpose

        // script/twtl_v1-ulpath.js
        let jsin = "console.log=function(){};window['ulpath']=1;function locate_elem(e,_,t){var l=[];var n=[];function o(t,i){let r=t.childNodes.length;for(let d=0;d<r;d++){let r=t.childNodes[d];if(r.innerText==e){if(l.length==0)l=[...i,d]}else if(r.innerText==_){if(n.length==0)n=[...i,d]}else{o(r,[...i,d])}}}o(t,[]);if(l.length!=n.length){return null}console.log(l,n);let i=-1;for(let e=0;e<l.length;e++){if(l[e]!=n[e]){i=e;break}}let r=[];let d=[];let u='document.body';let a='';if(i==-1)for(let e=0;e<l.length;e++){let _=l[e];r.push(_);u+='.childNodes['+_+']'}else for(let e=0;e<l.length;e++){let _=l[e];if(e<i){r.push(_);u+='.childNodes['+_+']'}else if(e>i){d.push(_)}}window['ulpath']={upper_idx:r,lower_idx:d,upper_path:u};return window['ulpath']}var send_back=arguments[arguments.length-1];clearInterval(window['twtl-get-ulpath']);window['twtl-get-ulpath']=setInterval((function(){if(document.body.innerText.includes('__________0__________')){var e=locate_elem('__________1__________','__________0__________',document.body);send_back(e);clearInterval(window['twtl-get-ulpath'])}}),1e3);";

        let jsout: String;

        match self.wda.eval_async(jsin, vec![]) {
            Ok(ret) => jsout = ret,
            Err(err_eval) => {
                if let WdaErr::WdcFail(WdcErr::BadDrvCmd(err, msg)) = err_eval {
                    if err.contains("timeout") {
                        return Err(MafaError::CacheNotBuildable);
                        // reaching here may bc twitter timeline has huge change, or,
                        // script timeout is too small
                    } else {
                        return Err(MafaError::WebDrvCmdRejected(err, msg));
                        // reaching here may bc twitter timeline has huge change
                    }
                } else {
                    return Err(MafaError::UnexpectedWda(err_eval));
                }
            }
        }

        dbgg!(("ulpath in str", &jsout));

        let ulpath = serde_json::from_slice::<UlPath>(jsout.as_bytes()).expect("deser");
        dbgg!(&ulpath);

        let u_part = serde_json::to_string(&ulpath.upper_idx).unwrap();
        let l_part = serde_json::to_string(&ulpath.lower_idx).unwrap();
        let comb = format!("[{},{}]\n", u_part, l_part);
        dbgg!(&comb);

        self.mafad
            .cache_append("twtl", &comb, &format!("{}-", &comb))?;

        self.caches.push(ulpath);

        Ok(())
    }

    fn cache_on_gh(&self, url: &str) -> Result<String> {
        match self.wda.go_url(url) {
            Ok(_) => {}
            Err(err_navi) => {
                if let WdaErr::WdcFail(WdcErr::BadDrvCmd(err, msg)) = err_navi {
                    return Err(MafaError::WebDrvCmdRejected(err, msg));
                } else {
                    return Err(MafaError::UnexpectedWda(err_navi));
                }
            }
        }

        let jsout: String;
        let jsin = "return document.getElementsByTagName('pre')[0].innerText;";

        match self.wda.eval(jsin, vec![]) {
            Ok(ret) => jsout = ret,
            Err(err_eval) => {
                if let WdaErr::WdcFail(WdcErr::BadDrvCmd(err, msg)) = err_eval {
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
            let remote_data = self
                .cache_on_gh("https://raw.githubusercontent.com/micl2e2/mafa-cache/master/twtl")?;

            self.mafad.init_cache("twtl", &remote_data)?;
        } else if let CacheMechanism::Local = self.input.cachm {
            self.mafad.try_init_cache(
                "twtl",
                "[[2,0,0,2,3,0,0,0,0,0,2,0,0,2,1,0,0,0],[0,0,0,0,0,1,1,1]]\n[[2,0,0,1,3,0,0,0,0,0,2,0,0,2,1,0,0,0],[0,0,0,0,0,1,1,1]]\n[[2,0,0,1,3,0,0,0,0,0,2,0,0,2,1,0],[0,0,0,0,0,1,1,1]]\n-",
            )?;
            // number of NL is the number of website changes
        } else if let CacheMechanism::No = self.input.cachm {
            rebuild_cache = true;
        }

        if rebuild_cache {
            self.notify(MafaEvent::BuildCache {
                cate: Category::Twtl,
                is_fin: false,
            })?;
            self.refresh_ulpath(true)?;
            self.notify(MafaEvent::BuildCache {
                cate: Category::Twtl,
                is_fin: true,
            })?;
        } else {
            self.refresh_ulpath(false)?;
        }

        Ok(())
    }

    fn handle_login(&self) -> Result<(EurKind, String)> {
        let url = "https://twitter.com/i/flow/login";
        if let Err(err_navi) = self.wda.go_url(url) {
            if let WdaErr::WdcFail(WdcErr::BadDrvCmd(err, msg)) = err_navi {
                return Err(MafaError::WebDrvCmdRejected(err, msg));
            } else {
                return Err(MafaError::UnexpectedWda(err_navi));
            }
        }

        let wait_in_secs = 60u64;

        self.notify(MafaEvent::WaitSecsMayInterrupt {
            cate: Category::Twtl,
            count: wait_in_secs,
            safe: !self.sub_input.imode,
        })?;

        std::thread::sleep(std::time::Duration::from_secs(wait_in_secs));

        Ok((EurKind::TwtlTryLogin, "_".to_string()))
    }

    ///
    /// Returned `String` is pretty-printed.
    pub fn handle(&mut self, pred_caches: Option<Vec<Vec<Vec<u8>>>>) -> Result<(EurKind, String)> {
        if self.sub_input.try_login {
            // this check should be inside handle, not inside new, bc needs a alive wda
            if !self.input.gui {
                return Err(MafaError::MustGui);
            }
            return self.handle_login();
        }

        if pred_caches.is_none() {
            self.try_rebuild_cache()?;
        } else {
            let pred_caches = pred_caches.expect("buggy");
            pred_caches.iter().for_each(|v| {
                self.caches.push(UlPath {
                    upper_idx: v[0].clone(),
                    lower_idx: v[1].clone(),
                })
            });
        }

        let n_tweets = if self.sub_input.ntweets > 8000 {
            8000
        } else {
            self.sub_input.ntweets
        };

        let tweets;
        self.notify(MafaEvent::FetchResult {
            cate: Category::Twtl,
            is_fin: false,
        })?;
        tweets = self.fetch(&self.sub_input.username, n_tweets)?;
        self.notify(MafaEvent::FetchResult {
            cate: Category::Twtl,
            is_fin: true,
        })?;

        let nocolor = self.input.nocolor;
        let asciiful = self.input.ascii;
        let wrap_width = self.input.wrap_width;
        let wrap_may_break = self.input.wrap_may_break;

        let mut twov_list = Vec::<TweetOverview>::new();
        let mut all_output = String::from("");
        let mut i_valid = 0;

        for i in 0..tweets.len() {
            let one_tweet = &tweets[i];
            let may_twov = TweetOverview::from_str(one_tweet);
            if let Ok(twov) = may_twov {
                all_output +=
                    &twov.pretty_print(i_valid + 1, nocolor, asciiful, wrap_width, wrap_may_break);
                twov_list.push(twov);
                i_valid += 1;
            }
        }

        self.try_save_tweets(twov_list)?;

        Ok((EurKind::TwtlResult, all_output))
    }

    fn try_save_tweets(&self, twov_list: Vec<TweetOverview>) -> Result<()> {
        if self.sub_input.save_to.is_none() {
            return Ok(());
        }

        assert!(self.sub_input.save_to.is_some());

        let save_to = self.sub_input.save_to.as_ref().expect("buggy");
        let save_format = self.sub_input.save_format;

        let try_open = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(save_to);

        match try_open {
            Ok(mut outf) => {
                if let None = save_format {
                    let wbuf = serde_json::to_vec(&twov_list).unwrap();
                    outf.write_all(&wbuf).unwrap();
                } else if let Some(SaveFormat::Json) = save_format {
                    let wbuf = serde_json::to_vec(&twov_list).unwrap();
                    outf.write_all(&wbuf).unwrap();
                } else if let Some(SaveFormat::Xml) = save_format {
                    #[derive(Debug, serde::Serialize)]
                    struct XmlResult<'a> {
                        item: &'a Vec<TweetOverview<'a>>,
                    }
                    let g = XmlResult { item: &twov_list };
                    let wbuf = quick_xml::se::to_string_with_root("list", &g).unwrap();
                    outf.write_all(&wbuf.as_bytes()).unwrap();
                }
            }
            Err(_ioerr) => match _ioerr.kind() {
                std::io::ErrorKind::NotFound => {
                    todo!();
                }
                _other => {
                    dbgg!(_other);
                }
            },
        }

        Ok(())
    }

    fn fetch(&self, tuid: &str, n_tweets: u16) -> Result<Vec<String>> {
        let url = format!("https://twitter.com/{}", tuid);

        let mut is_url_reached = false;
        let mut try_times = 10;
        while try_times > 0 {
            if let Err(err_navi) = self.wda.go_url(&url) {
                if let WdaErr::WdcFail(WdcErr::BadDrvCmd(err, msg)) = err_navi {
                    if err.contains("timeout") {
                        self.notify(MafaEvent::ConnectTimeoutRetry {
                            cate: Category::Twtl,
                            is_fin: false,
                        })?;
                        try_times -= 1;
                        dbgg!(try_times);
                        continue;
                    } else {
                        return Err(MafaError::WebDrvCmdRejected(err, msg));
                    }
                } else {
                    return Err(MafaError::UnexpectedWda(err_navi));
                }
            }

            // check needs login
            match self.wda.get_url() {
                Ok(url) => {
                    if url.contains("login") {
                        return Err(MafaError::RequireLogin);
                    }
                }
                Err(err_geturl) => {
                    if let WdaErr::WdcFail(WdcErr::BadDrvCmd(err, msg)) = err_geturl {
                        return Err(MafaError::WebDrvCmdRejected(err, msg));
                    } else {
                        return Err(MafaError::UnexpectedWda(err_geturl));
                    }
                }
            }

            is_url_reached = true;
            self.notify(MafaEvent::ConnectTimeoutRetry {
                cate: Category::Twtl,
                is_fin: true,
            })?;
            break;
        }

        if !is_url_reached {
            return Err(MafaError::DataFetchedNotReachable);
        }

        let mut nleft_tweets = n_tweets;
        let mut tweets_got_final = Vec::<String>::new();
        let mut tweets_got_uoset = std::collections::BTreeSet::<String>::new();

        let mut reach_tl_end_counter = 0u8;
        let mut prev_nreq = 0u16;

        let mut ulpaths_i = 0;
        let ulpaths_len = self.caches.len();

        if ulpaths_len == 0 {
            return Err(MafaError::BugFound(2347));
        }

        while nleft_tweets > 0 {
            let mut jsin = "window['ulpath']=arguments[0];".to_string();
            // script/twtl_v1-tweets.js
            jsin += "console.log=function(){};function get_loaded_nth(e){let n=e.childNodes.length;if(n==0)return 0;let t=0;for(let l=0;l<n;l++){let o=e.childNodes[l];if(o==undefined){console.log('nth child null',n,e.childNodes);return 0}if(o.innerText!=null&&o.innerText!=undefined)t+=1;else return t}return t}var send_back=arguments[arguments.length-1];clearInterval(window['get_tweets']);window['get_tweets']=setInterval((function(){var e=window['ulpath'];var n=document.body;for(let t=0;t<e.upper_idx.length;t++){let l=e.upper_idx[t];if(n.childNodes.length>l){n=n.childNodes[l]}}window.ulpath.parent_of_fork_nodes=n;let t=get_loaded_nth(n);console.log('loaded_n',t);if(t>0){var l=t;let e=[];console.log('nchild is',l);for(let t=0;t<l;t++){let l=n.childNodes[t];let d=l.innerHTML.match('/status/([0-9]+)/');var o='twtl_v1'+String.fromCharCode(10);if(d!=null&&d!=undefined&&d.length==2)o+=d[1]+String.fromCharCode(10);else o+='UNKNOWNID'+String.fromCharCode(10);o+=l.innerText;e.push(o)}console.log(e);send_back(e);clearInterval(window['get_tweets'])}else{}}),1e3);";

            let arg0 = serde_json::to_string(&self.caches[ulpaths_i]).unwrap();
            let jsout: String;
            match self.wda.eval_async(&jsin, vec![&arg0]) {
                Ok(ret) => jsout = ret,
                Err(err_eval) => {
                    if let WdaErr::WdcFail(WdcErr::BadDrvCmd(err, msg)) = err_eval {
                        if err.contains("timeout") {
                            ulpaths_i += 1;
                            if ulpaths_i < ulpaths_len {
                                self.notify(MafaEvent::TryNextCache {
                                    cate: Category::Twtl,
                                    is_fin: false,
                                })?;
                                continue;
                            } else {
                                if tweets_got_final.len() != 0 {
                                    dbgmsg!("keep all we've got, skip others");
                                    break;
                                } else {
                                    return Err(MafaError::AllCachesInvalid);
                                }
                                // reaching here may bc twitter timeline has huge change, or,
                                // script timeout is too small
                            }
                        } else {
                            if tweets_got_final.len() != 0 {
                                dbgmsg!("keep all we've got, skip others");
                                break;
                            } else {
                                return Err(MafaError::WebDrvCmdRejected(err, msg));
                            }
                        }
                    } else {
                        if tweets_got_final.len() != 0 {
                            dbgmsg!("keep all we've got, skip others");
                            break;
                        } else {
                            return Err(MafaError::UnexpectedWda(err_eval));
                        }
                    }
                }
            }

            self.notify(MafaEvent::TryNextCache {
                cate: Category::Twtl,
                is_fin: true,
            })?;

            let tweets = serde_json::from_slice::<Vec<String>>(&jsout.as_bytes()).expect("deser");
            let n_got = tweets.len();

            dbgg!(tweets.len());

            for t in tweets {
                if tweets_got_uoset.insert(t.to_string()) {
                    tweets_got_final.push(t.to_string());
                    nleft_tweets -= 1;
                }
                if nleft_tweets == 0 {
                    break;
                }
            }

            dbgg!(&nleft_tweets);

            self.notify(MafaEvent::SimpleProgress {
                cate: Category::Twtl,
                total: n_tweets as u32,
                curr: (n_tweets - nleft_tweets) as u32,
                is_fin: if nleft_tweets == 0 { true } else { false },
            })?;

            if nleft_tweets == prev_nreq {
                reach_tl_end_counter += 1;
                std::thread::sleep(std::time::Duration::from_secs(1));
            } else {
                prev_nreq = nleft_tweets;
                reach_tl_end_counter = 0;
            }

            if reach_tl_end_counter > 9 {
                dbgmsg!("maybe reach end, left: {}", nleft_tweets);
                break;
            }

            // script/twtl_v1-scrolld.js
            let jsin = "console.log=function(){};var ulpath=window['ulpath'];var parent_of_fork_nodes=ulpath.parent_of_fork_nodes;var n_most_recent_loaded=arguments[0];var nchild=parent_of_fork_nodes.childNodes.length;let nth_child=parent_of_fork_nodes.childNodes[n_most_recent_loaded-1];if(nth_child==undefined){console.log('scroll fail',parent_of_fork_nodes.childNodes.length,n_most_recent_loaded)}else{console.log('scroll good');nth_child.scrollIntoView()}";

            if nleft_tweets > 0 {
                match self.wda.eval(&jsin, vec![&n_got.to_string()]) {
                    Ok(_) => {}
                    Err(err_eval) => {
                        if tweets_got_final.len() != 0 {
                            dbgmsg!("keep all we've got, skip others");
                            break;
                        } else {
                            return Err(MafaError::UnexpectedWda(err_eval));
                        }
                    }
                }
            }
        }

        Ok(tweets_got_final)
    }
}

#[cfg(test)]
mod utst_merged {
    use super::*;

    #[test]
    fn username_1() {
        let matched = crate::get_cmd().get_matches_from(vec!["mafa", "twtl", "twitter"]);

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("twtl", sub_m)) => {
                    let twtl_in = TwtlInput::from_ca_matched(sub_m).expect("must ok");
                    assert_eq!(twtl_in.username, "twitter");
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn username_2() {
        let matched = crate::get_cmd().get_matches_from(vec!["mafa", "twtl", ""]);

        match MafaInput::from_ca_matched(&matched) {
            Ok(_mafa_in) => match matched.subcommand() {
                Some(("twtl", sub_m)) => {
                    let twtl_in = TwtlInput::from_ca_matched(sub_m);
                    if let Err(MafaError::InvalidTwitterUsername) = twtl_in {
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
    fn username_3() {
        let matched = crate::get_cmd().try_get_matches_from(vec!["mafa", "twtl"]);

        if let Err(e) = matched {
            assert_eq!(e.kind(), clap::error::ErrorKind::MissingRequiredArgument);
        } else {
            assert!(false);
        }
    }
}
