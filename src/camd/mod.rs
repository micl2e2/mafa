// Copyright (C) 2023 Michael Lee <micl2e2@proton.me>
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
use crate::MafaClient;

use crate::ev_ntf::Category;
use crate::ev_ntf::EurKind;
use crate::ev_ntf::EventNotifier;
use crate::ev_ntf::MafaEvent;

use crate::MafaInput;

use crate::comm;
use crate::comm::CacheMechanism;

mod camd_res;
use camd_res::CamdResult;

use clap::Arg as ClapArg;
use clap::ArgAction as ClapArgAction;
use clap::ArgMatches as ClapArgMatches;
use clap::Command as ClapCommand;

#[derive(Debug, Default)]
pub struct CamdInput {
    words: String,
}

impl CamdInput {
    pub fn from_ca_matched(ca_matched: &ClapArgMatches) -> Result<Self> {
        let mut camd_in = CamdInput::default();

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
            camd_in.words = words;
        }

        dbgg!(&camd_in);

        Ok(camd_in)
    }

    pub fn from_imode_args(args: Vec<&str>) -> Result<CamdInput> {
        let cmd_camd = get_cmd();

        let m = cmd_camd.try_get_matches_from(args);

        match m {
            Ok(ca_matched) => {
                let camd_in = CamdInput::from_ca_matched(&ca_matched)?;
                Ok(camd_in)
            }
            // this will print helper
            Err(err_match) => Err(MafaError::ClapMatchError(err_match.render())),
        }
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
            "WORD"
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

$ mafa Camd thanks

or 

$ mafa Camd thank you"#;
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
    let opt_words = {
        type O = opts::Words;
        ClapArg::new(O::id())
            // .required(true)
            .num_args(O::n_args())
            .help(O::helper())
            .long_help(O::long_helper())
    };

    let cmd_camd = ClapCommand::new("camd")
        .about("Word definition by Cambridge Dictionary")
        .arg(opt_words);

    cmd_camd
}

#[derive(Debug, Default)]
pub struct Upath(Vec<u8>);

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

fn is_valid_words(v: &str) -> bool {
    if v.len() > 0 {
        true
    } else {
        false
    }
}

impl<'a, 'b, 'c> MafaClient<'a, 'b, 'c, CamdInput, Upath> {
    fn upath_locate(&self, words: &str, expl: &str, wait_before_extract: u64) -> Result<Vec<u8>> {
        let url = format!(
            "https://dictionary.cambridge.org/us/dictionary/english/{}",
            words
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

        // script/camd-upath.js, CHECKED
        let js_in="console.log=function(){};function locate_elem(e){var o=[];function l(e,n,t){let c=e.childNodes.length;for(let d=0;d<c;d++){let c=e.childNodes[d];if(c.innerText&&c.innerText==n){console.log('yes',c);o=[...t,d]}else{l(c,n,[...t,d])}}}let n=e;l(document.body,n,[]);console.log(o);let t=o.map((()=>document.body));console.log(t);for(let e=0;e<o.length;e++){for(let l=0;l<o[e].length;l++){t[e]=t[e].childNodes[o[e][l]]}}return o}return locate_elem(arguments[0]);";

        let js_out;
        match self.wda.eval(&js_in, vec![expl]) {
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

    fn rebuild_internal(&mut self, is_rebuild: bool) -> Result<()> {
        if !is_rebuild {
            let caches_from_files = UpathCache::from_pbuf(self.mafad.pathto_exist_cache("camd")?)?;
            self.caches = caches_from_files.0;
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
                match self.upath_locate(
                    "hello",
                    "\"used when meeting or greeting someone:\"",
                    time_before,
                ) {
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
                match self.upath_locate(
                    "world",
                    "\"the earth and all the people, places, and things on it:\"",
                    time_before,
                ) {
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
                    cate: Category::Camd,
                    is_fin: true,
                })?;
                break;
            } else {
                self.notify(MafaEvent::CacheRetry {
                    cate: Category::Camd,
                    is_fin: false,
                })?;
                try_times -= 1;
                time_before += time_before;
                dbgmsg!("need retry {} {}", try_times, time_before);
            }
        }

        dbgg!(&try_times);

        // all must be present
        if upath1.is_none() || upath2.is_none() {
            return Err(MafaError::CacheRebuildFail(
                CacheRebuildFailKind::UpathNotFound,
            ));
        }

        let upath1 = upath1.expect("buggy");
        let upath2 = upath2.expect("buggy");

        dbgmsg!("upath1:{:?} upath2:{:?}", &upath1, &upath2);

        if upath1.len() == 0 || upath2.len() == 0 {
            return Err(MafaError::CacheRebuildFail(
                CacheRebuildFailKind::UpathLenZero,
            ));
        }

        let matched_len = {
            let mut res = 0;
            let len = usize::min(upath1.len(), upath2.len());
            for i in 0..len {
                if upath1[i] == upath2[i] {
                    res += 1;
                } else {
                    break;
                }
            }
            res
        };

        let u_part = serde_json::to_string(&upath1[0..matched_len]).unwrap();
        let comb = format!("{}\n", u_part);
        dbgg!(&comb);

        self.mafad
            .cache_append("camd", &comb, &format!("{}-", &comb))?;

        self.caches.push(Upath(Vec::from(&upath1[0..matched_len])));

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

    ///
    /// Try to rebuild cache regarding cache mechanism, built cache will be
    /// put in dedicated file on disk, typically inside $HOME/.mafa. Default to
    /// read from pre-filled cache file.
    fn try_rebuild_cache(&mut self) -> Result<()> {
        let mut is_rebuild = false;

        if let CacheMechanism::Remote = self.input.cachm {
            let remote_data = self
                .cache_on_gh("https://raw.githubusercontent.com/micl2e2/mafa-cache/master/camd")?;

            self.mafad.init_cache("camd", &remote_data)?;
        } else if let CacheMechanism::Local = self.input.cachm {
            self.mafad.try_init_cache("camd", "[11,1,1,3,3]\n-")?;
        } else if let CacheMechanism::No = self.input.cachm {
            is_rebuild = true;
        }

        if is_rebuild {
            self.notify(MafaEvent::BuildCache {
                cate: Category::Camd,
                is_fin: false,
            })?;
            self.rebuild_internal(true)?;
            self.notify(MafaEvent::BuildCache {
                cate: Category::Camd,
                is_fin: true,
            })?;
        } else {
            self.rebuild_internal(false)?;
        }

        Ok(())
    }

    ///
    /// Returned `String` is pretty-printed.
    pub fn handle(&mut self, pred_caches: Option<Vec<Vec<u8>>>) -> Result<(EurKind, String)> {
        // caches
        if pred_caches.is_none() {
            self.try_rebuild_cache()?;
        } else {
            let pred_caches = pred_caches.ok_or(MafaError::BugFound(4567))?;
            pred_caches
                .iter()
                .for_each(|v| self.caches.push(Upath(v.clone())));
        }
        if self.caches.len() == 0 {
            panic!("buggy");
        }

        self.notify(MafaEvent::FetchResult {
            cate: Category::Camd,
            is_fin: false,
        })?;
        let explained = self.fetch(&self.sub_input.words)?;
        self.notify(MafaEvent::FetchResult {
            cate: Category::Camd,
            is_fin: true,
        })?;

        dbgg!((&self.sub_input.words, &explained));
        let camd_res = CamdResult::from_str(&self.sub_input.words, &explained)?;
        dbgg!(&camd_res);

        Ok((
            EurKind::CamdResult,
            camd_res.pretty_print(
                self.input.nocolor,
                self.input.ascii,
                self.input.wrap_width.into(),
            )?,
        ))
    }

    fn fetch(&self, words: &str) -> Result<String> {
        let url = format!(
            "https://dictionary.cambridge.org/us/dictionary/english/{}",
            words
        );

        let mut res = "???".to_string();

        dbgg!(&self.caches);
        let mut upaths_i = 0; // iterate over all upaths
        let upaths_len = self.caches.len();

        // script/camd-getres.js, CHECKED
        let jsin_getres = "console.log=function(){};var send_back=arguments[arguments.length-1];var upath=arguments[0];clearInterval(window['camd-res']);window['camd-res']=setInterval(try_send_back,500);function try_send_back(){var e=document.body;for(let n=0;n<upath.length;n++){if(e==undefined){console.log('undef',n,e);return}else{console.log('following...')}e=e.childNodes[upath[n]]}if(e!=undefined){var n=e.childNodes.length;if(n==0){return}for(let d=0;d<n;d++){let n=e.childNodes[d];if(n.innerText!=undefined&&n.innerText.includes(String.fromCharCode(10)+'Add to word list '+String.fromCharCode(10))){e=e.childNodes[d];break}}console.log('upath & interested',upath,e);let d='';for(let t=0;t<n;t++){let n=e.childNodes[t];if(n!=undefined&&n.nodeType==1){d+='______'+n.innerText}}send_back(d);clearInterval(window['camd-res'])}}";

        let mut is_url_reached = false;

        // try_times = go_url + eval_js
        let mut try_times = 5; // sufficient to let try again succeed

        // let mut wait_before = 500;
        let mut wait_before = 10; // time to wait after navi but before eval

        let mut expl_res = "".to_string();

        while try_times > 0 {
            if let Err(err_navi) = self.wda.go_url(&url) {
                if let WdaError::WdcFail(WdcError::BadDrvCmd(err, msg)) = err_navi {
                    if err.contains("timeout") {
                        self.notify(MafaEvent::ConnectTimeoutRetry {
                            cate: Category::Camd,
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
                cate: Category::Camd,
                is_fin: true,
            })?;

            let upath_curr = &self.caches[upaths_i].0;
            let arg0 = serde_json::to_string(&upath_curr[..]).unwrap();

            sleep(Duration::from_millis(wait_before));

            match self.wda.eval_async(&jsin_getres, vec![&arg0]) {
                Ok(retstr) => {
                    expl_res = retstr;
                    break; // we done
                }

                Err(err_eval) => {
                    if let WdaError::WdcFail(WdcError::BadDrvCmd(err, msg)) = err_eval {
                        if err.contains("timeout") {
                            upaths_i += 1;
                            if upaths_i < upaths_len {
                                self.notify(MafaEvent::TryNextCache {
                                    cate: Category::Camd,
                                    is_fin: false,
                                })?;
                                // continue;
                            } else {
                                self.notify(MafaEvent::TryNextCache {
                                    cate: Category::Camd,
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
            cate: Category::Camd,
            is_fin: true,
        })?;

        if !is_url_reached {
            return Err(MafaError::DataFetchedNotReachable);
        }

        Ok(expl_res)
    }
}

#[cfg(test)]
mod utst_merged {
    use super::*;

    #[test]
    fn words_1() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "camd", "hello"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("camd", sub_m)) => {
                    let camd_in = CamdInput::from_ca_matched(sub_m).expect("must ok");
                    assert_eq!(camd_in.words, "hello");
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn words_2() {
        let matched = crate::get_cmd()
            .try_get_matches_from(vec!["mafa", "camd", "hello", "everyone"])
            .expect("buggy");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("camd", sub_m)) => {
                    let camd_in = CamdInput::from_ca_matched(sub_m).expect("must ok");
                    assert_eq!(camd_in.words, "hello everyone");
                }
                _ => assert!(false),
            },
            Err(_) => assert!(false),
        }
    }
}
