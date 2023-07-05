// Copyright (C) 2023 Michael Lee <imichael2e2@proton.me/...@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 or any later
// version <LICENSE-GPL or https://www.gnu.org/licenses/gpl-3.0.txt>.
//
// This file may not be copied, modified, or distributed except in compliance
// with the license.
//

use core::time::Duration;
use std::io;
use std::io::Write;
use std::time::Instant;

use crate::error::MafaError;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Category {
    Gtrans,
    Twtl,
    Mafa,
}

impl Category {
    fn as_str(&self) -> &'static str {
        match self {
            Category::Mafa => "Mafa",
            Category::Gtrans => "Google Translate",
            Category::Twtl => "Twitter Timeline",
        }
    }
}

#[derive(Debug)]
pub enum MafaEvent {
    Initialize {
        cate: Category,
        is_fin: bool,
    },
    BuildCache {
        cate: Category,
        is_fin: bool,
    },
    FetchResult {
        cate: Category,
        is_fin: bool,
    },
    //
    TryNextCache {
        cate: Category,
        is_fin: bool,
    },
    CacheRetry {
        cate: Category,
        is_fin: bool,
    },
    SrvTempUnavRetry {
        cate: Category,
        is_fin: bool,
    },
    ConnectTimeoutRetry {
        cate: Category,
        is_fin: bool,
    },
    SimpleProgress {
        cate: Category,
        total: u32,
        curr: u32,
        is_fin: bool,
    },
    ///
    /// the errors should exit the process
    FatalMafaError {
        cate: Category,
        err: MafaError,
    },
    HandlerMissed {
        cate: Category,
        err: MafaError,
    },
    ExactWhatRequest {
        cate: Category,
        kind: EurKind,
    },
    ExactUserRequest {
        cate: Category,
        kind: EurKind,
        output: String,
    },
    WaitSecsMayInterrupt {
        cate: Category,
        count: u64,
        safe: bool,
    },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EurKind {
    ImodeHelper,
    GtransResult,  /* google translate translation result */
    GtransAllLang, /* list all supported lang */
    TwtlResult,    /* twitter timeline result */
    TwtlTryLogin,  /* login/logout twitter account */
}

#[derive(Debug)]
pub struct EventNotifier {
    smode: bool,
    pub(crate) color: bool,
    queue: Vec<EventDetail>,
    wall_clock: Instant,
}

#[derive(Debug)]
struct EventDetail(MafaEvent, Duration);

macro_rules! print_not {
    ($cond:expr, $print_what:expr) => {
        if !$cond {
            print!("{}",$print_what);
        }
    };
    ($cond:expr, $print_what:literal, $($val:tt)*) => {
        if !$cond {
            print!($print_what, $($val)*);
        }
    };
}

macro_rules! println_not {
    ($cond:expr, $print_what:literal) => {
        if !$cond {
            println!($print_what);
        }
    };
    ($cond:expr, $print_what:literal, $($val:tt)*) => {
        if !$cond {
            println!($print_what, $($val)*);
        }
    };
}

macro_rules! eprint_not {
    ($cond:expr, $print_what:expr) => {
        if !$cond {
            eprint!("{}",$print_what);
        }
    };
    ($cond:expr, $print_what:literal, $($val:tt)*) => {
        if !$cond {
            eprint!($print_what, $($val)*);
        }
    };
}

macro_rules! eprintln_not {
    ($cond:expr, $print_what:literal) => {
        if !$cond {
            eprintln!($print_what);
        }
    };
    ($cond:expr, $print_what:literal, $($val:tt)*) => {
        if !$cond {
            eprintln!($print_what, $($val)*);
        }
    };
}

impl EventNotifier {
    pub fn new() -> Self {
        EventNotifier {
            smode: false,
            color: true,
            queue: vec![],
            wall_clock: Instant::now(),
        }
    }

    pub fn set_silent(&mut self) {
        self.smode = true;
    }

    pub fn set_nsilent(&mut self) {
        self.smode = false;
    }

    pub fn set_color(&mut self) {
        self.color = true;
    }

    pub fn set_nocolor(&mut self) {
        self.color = false;
    }

    pub fn notify(&mut self, ev: MafaEvent) {
        let mut is_skip_push = false;

        match ev {
            MafaEvent::ExactUserRequest { ref output, .. } => {
                if !self.is_prev_final() {
                    println!();
                }
                // if start by _, we dont print anything
                if output.as_bytes()[0] != b'_' {
                    println!("{}", output);
                }
            }

            MafaEvent::WaitSecsMayInterrupt { cate, count, safe } => {
                if !self.is_prev_final() {
                    println!();
                }

                print!("[{}] Please finish in {} seconds, ", cate.as_str(), count,);
                if safe {
                    print!("press \u{1b}[40;1mCtrl-C\u{1b}[0m here if finished.",);
                } else {
                    print!("do \u{1b}[40;1mNOT\u{1b}[0m press other keys.",);
                }
                println!();
            }

            //
            MafaEvent::Initialize { cate, is_fin } => {
                if is_fin {
                    if self.queue.len() > 0 {
                        let last_ev = &self.queue.last().unwrap().0;
                        if let MafaEvent::Initialize { .. } = last_ev {
                            println_not!(self.smode, "ok");
                        } else {
                            if !self.is_prev_final() {
                                println_not!(self.smode, "");
                            }

                            println_not!(self.smode, "[{}] Initializing...ok", cate.as_str());
                        }
                    }
                } else {
                    print_not!(self.smode, "[{}] Initializing...", cate.as_str());
                }
            }

            MafaEvent::BuildCache { cate, is_fin } => {
                if is_fin {
                    if self.queue.len() > 0 {
                        let last_ev = &self.queue.last().unwrap().0;
                        if let MafaEvent::BuildCache { .. } = last_ev {
                            println_not!(self.smode, "ok");
                        } else {
                            if !self.is_prev_final() {
                                println_not!(self.smode, "");
                            }

                            println_not!(self.smode, "[{}] Building cache...ok", cate.as_str());
                        }
                    }
                } else {
                    print_not!(self.smode, "[{}] Building cache...", cate.as_str());
                }
            }

            MafaEvent::FetchResult { cate, is_fin } => {
                if is_fin {
                    if self.queue.len() > 0 {
                        let last_ev = &self.queue.last().unwrap().0;

                        if let MafaEvent::FetchResult { .. } = last_ev {
                            println_not!(self.smode, "ok");
                        } else {
                            if !self.is_prev_final() {
                                println_not!(self.smode, "");
                            }
                            println_not!(self.smode, "[{}] Fetching result...ok", cate.as_str());
                        }
                    }
                } else {
                    print_not!(self.smode, "[{}] Fetching result...", cate.as_str());
                }
            }

            MafaEvent::CacheRetry { cate, is_fin } => {
                if self.queue.len() > 0 {
                    let last_ev = &self.queue.last().unwrap().0;
                    if is_fin {
                        // only has retried previously
                        if let MafaEvent::CacheRetry { .. } = last_ev {
                            println_not!(self.smode, "");
                        } else {
                            is_skip_push = true;
                        }
                    } else {
                        if let MafaEvent::CacheRetry {
                            cate: o_cate,
                            is_fin: o_is_fin,
                        } = last_ev
                        {
                            if o_cate == &cate && o_is_fin == &is_fin {
                                print_not!(self.smode, ".");
                            }
                        } else {
                            if !self.is_prev_final() {
                                println_not!(self.smode, "");
                            }
                            print_not!(
                                self.smode,
                                "[{}] Build cache failed, retrying",
                                cate.as_str()
                            );
                        }
                    }
                } else {
                    unreachable!(); // FIXME: should be an mafaerror
                }
            }

            MafaEvent::SrvTempUnavRetry { cate, is_fin } => {
                if self.queue.len() > 0 {
                    let last_ev = &self.queue.last().unwrap().0;
                    if is_fin {
                        // only has retried previously
                        if let MafaEvent::SrvTempUnavRetry { .. } = last_ev {
                            println_not!(self.smode, "");
                        } else {
                            is_skip_push = true;
                        }
                    } else {
                        if let MafaEvent::SrvTempUnavRetry {
                            cate: o_cate,
                            is_fin: o_is_fin,
                        } = last_ev
                        {
                            if o_cate == &cate && o_is_fin == &is_fin {
                                print_not!(self.smode, ".");
                            }
                        } else {
                            if !self.is_prev_final() {
                                println_not!(self.smode, "");
                            }
                            print_not!(
                                self.smode,
                                "[{}] Service temporarily unavaiable, retrying",
                                cate.as_str()
                            );
                        }
                    }
                } else {
                    unreachable!(); // FIXME: should be an mafaerror
                }
            }

            MafaEvent::ConnectTimeoutRetry { cate, is_fin } => {
                if self.queue.len() > 0 {
                    let last_ev = &self.queue.last().unwrap().0;
                    if is_fin {
                        // only has retried previously
                        if let MafaEvent::ConnectTimeoutRetry { .. } = last_ev {
                            println_not!(self.smode, "");
                        } else {
                            is_skip_push = true;
                        }
                    } else {
                        if let MafaEvent::ConnectTimeoutRetry {
                            cate: o_cate,
                            is_fin: o_is_fin,
                        } = last_ev
                        {
                            if o_cate == &cate && o_is_fin == &is_fin {
                                print_not!(self.smode, ".");
                            }
                        } else {
                            if !self.is_prev_final() {
                                println_not!(self.smode, "");
                            }
                            print_not!(
                                self.smode,
                                "[{}] Connection timeout, retrying",
                                cate.as_str()
                            );
                        }
                    }
                } else {
                    unreachable!(); // FIXME: should be an mafaerror
                }
            }

            MafaEvent::TryNextCache { cate, is_fin } => {
                if self.queue.len() > 0 {
                    let last_ev = &self.queue.last().unwrap().0;
                    if is_fin {
                        // only has retried previously
                        if let MafaEvent::TryNextCache { .. } = last_ev {
                            println_not!(self.smode, "");
                        } else {
                            is_skip_push = true;
                        }
                    } else {
                        if let MafaEvent::TryNextCache {
                            cate: o_cate,
                            is_fin: o_is_fin,
                        } = last_ev
                        {
                            if o_cate == &cate && o_is_fin == &is_fin {
                                print_not!(self.smode, ".");
                            }
                        } else {
                            if !self.is_prev_final() {
                                println_not!(self.smode, "");
                            }
                            print_not!(self.smode, "[{}] Trying other caches", cate.as_str());
                        }
                    }
                } else {
                    unreachable!(); // FIXME: should be an mafaerror
                }
            }

            //
            MafaEvent::SimpleProgress {
                cate,
                total,
                curr,
                is_fin,
            } => {
                if self.queue.len() > 0 {
                    let last_ev = &self.queue.last().unwrap().0;
                    if let MafaEvent::SimpleProgress {
                        cate: o_cate,
                        total: o_total,
                        curr: _,
                        is_fin: _,
                    } = last_ev
                    {
                        if o_cate == &cate && o_total == &total {
                            print_not!(self.smode, "\r");
                        }
                    } else {
                        if !self.is_prev_final() {
                            println_not!(self.smode, "");
                        }
                    }
                    print_not!(
                        self.smode,
                        "[{}] {}/{} ({}%)",
                        cate.as_str(),
                        curr,
                        total,
                        ((curr as f64 / total as f64) * 100.0) as u32
                    );
                    if is_fin {
                        println_not!(self.smode, "");
                    }
                } else {
                    unreachable!(); // FIXME: should be an mafaerror
                }
            }

            MafaEvent::FatalMafaError { cate, ref err } => match err {
                MafaError::InvalidSocks5Proxy => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(
                        self.smode,
                        "socks5 proxy is not a valid value({})",
                        cate.as_str()
                    );
                }

                MafaError::InvalidTimeoutPageLoad => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(
                        self.smode,
                        "page load timeout is not a valid value({})",
                        cate.as_str()
                    );
                }

                MafaError::InvalidTimeoutScript => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(
                        self.smode,
                        "script timeout is not a valid value({})",
                        cate.as_str()
                    );
                }

                MafaError::InvalidSourceLang => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(self.smode, "invalid source language({})", cate.as_str());
                }

                MafaError::InvalidTargetLang => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(self.smode, "invalid target language({})", cate.as_str());
                }

                MafaError::AllCachesInvalid => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(self.smode, "all caches invalid({})", cate.as_str());
                }

                MafaError::CacheNotBuildable => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(self.smode, "cache not buildable({})", cate.as_str());
                }

                MafaError::RequireLogin => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(self.smode, "login required ({})", cate.as_str());
                }

                MafaError::MustGui => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(
                        self.smode,
                        "GUI mode required, try again with --gui option ({})",
                        cate.as_str()
                    );
                }

                MafaError::DataFetchedNotReachable => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(self.smode, "website is not reachable ({})", cate.as_str());
                }

                MafaError::ClapMatchError(ca_err) => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }
                    eprint_not!(self.smode, "{}", ca_err.ansi());
                }

                MafaError::WebDrvCmdRejected(err, msg) => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }
                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );

                    if msg.contains("neterror")
                        && (msg.contains("dnsNotFound") || msg.contains("nssFailure"))
                    {
                        eprintln_not!(self.smode, "internet connection failed");
                    } else if msg.contains("neterror") && msg.contains("proxyConnectFailure") {
                        eprintln_not!(self.smode, "proxy connection failed");
                    } else if err == "script timeout" {
                        eprintln_not!(self.smode, "script evaluation timeout ({})", cate.as_str());
                    } else {
                        eprintln_not!(
                            self.smode,
                            "webdriver cmd rejected({},{},{})",
                            cate.as_str(),
                            err,
                            msg
                        );
                    }
                }

                MafaError::UnexpectedWda(err_wda) => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(
                        self.smode,
                        "unexpected wda error({}): {:?}",
                        cate.as_str(),
                        err_wda
                    );
                }

                MafaError::CacheRebuildFail(fk) => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(
                        self.smode,
                        "rebuild cache failed({}): {:?}",
                        cate.as_str(),
                        fk
                    );
                }

                err_other => {
                    if !self.is_prev_final() {
                        eprintln_not!(self.smode, "");
                    }

                    eprint_not!(
                        self.smode,
                        if self.color {
                            "\u{1b}[31;1merror: \u{1b}[0m"
                        } else {
                            "error: "
                        }
                    );
                    eprintln_not!(self.smode, "unexpected: {:?}", err_other);
                }
            },

            // DONT FORGET: when this error occurs, check process exit code
            MafaEvent::HandlerMissed { cate, ref err } => {
                if !self.is_prev_final() {
                    eprintln_not!(self.smode, "");
                }

                eprint_not!(
                    self.smode,
                    if self.color {
                        "\u{1b}[31;1merror: \u{1b}[0m"
                    } else {
                        "error: "
                    }
                );
                eprintln_not!(
                    self.smode,
                    "handler for {:?} not found ({})",
                    err,
                    cate.as_str()
                );
            }

            _ => {}
        }

        io::stdout().flush().unwrap();
        if !is_skip_push {
            self.queue.push(EventDetail(ev, self.wall_clock.elapsed()));
        }
        // dbgg!(&self.queue);
    }

    fn is_prev_final(&self) -> bool {
        if self.queue.len() == 0 {
            return true;
        }

        let last_ev = &self.queue.last().unwrap().0;

        match last_ev {
            MafaEvent::Initialize { is_fin, .. } => {
                return *is_fin;
            }

            MafaEvent::BuildCache { is_fin, .. } => {
                return *is_fin;
            }

            MafaEvent::FetchResult { is_fin, .. } => {
                return *is_fin;
            }

            MafaEvent::CacheRetry { is_fin, .. } => {
                return *is_fin;
            }

            MafaEvent::ConnectTimeoutRetry { is_fin, .. } => {
                return *is_fin;
            }

            MafaEvent::SrvTempUnavRetry { is_fin, .. } => {
                return *is_fin;
            }

            _ => {}
        }

        return true;
    }

    // cannot be silent
    pub fn elap(&self, cate: Category) {
        match cate {
            Category::Gtrans => {
                self.elap_gtrans();
            }
            Category::Twtl => {
                self.elap_twtl();
            }
            _ => {}
        }
    }

    fn elap_gtrans(&self) {
        let mut p_prepare = (Duration::ZERO, Duration::ZERO);
        let mut p_cache = (Duration::ZERO, Duration::ZERO);
        let mut p_fetch = (Duration::ZERO, Duration::ZERO);
        let mut p_whole = (Duration::ZERO, Duration::ZERO); // should be just sum of above?

        let is_filled = |arg: (Duration, Duration)| -> bool {
            arg.0 != Duration::ZERO && arg.1 != Duration::ZERO
        };

        let lasti = self.queue.len() - 1;
        for _i in 0..self.queue.len() {
            if is_filled(p_prepare)
                && is_filled(p_cache)
                && is_filled(p_fetch)
                && is_filled(p_whole)
            {
                break;
            }
            let i = lasti - _i;
            let ev = &self.queue[i];
            match ev.0 {
                MafaEvent::Initialize { cate, is_fin } => {
                    if is_fin {
                        if cate == Category::Gtrans && p_prepare.1 == Duration::ZERO {
                            p_prepare.1 = ev.1;
                        }
                    } else {
                        if cate == Category::Gtrans && p_prepare.0 == Duration::ZERO {
                            p_prepare.0 = ev.1;
                            p_whole.0 = ev.1;
                        }
                    }
                }

                MafaEvent::BuildCache { cate, is_fin } => {
                    if is_fin {
                        if cate == Category::Gtrans && p_cache.1 == Duration::ZERO {
                            p_cache.1 = ev.1;
                        }
                    } else {
                        if cate == Category::Gtrans && p_cache.0 == Duration::ZERO {
                            p_cache.0 = ev.1;
                        }
                    }
                }

                MafaEvent::FetchResult { cate, is_fin } => {
                    if is_fin {
                        if cate == Category::Gtrans && p_fetch.1 == Duration::ZERO {
                            p_fetch.1 = ev.1;
                        }
                    } else {
                        if cate == Category::Gtrans && p_fetch.0 == Duration::ZERO {
                            p_fetch.0 = ev.1;
                        }
                    }
                }

                MafaEvent::ExactWhatRequest { cate, .. } => {
                    if cate == Category::Gtrans && p_whole.1 == Duration::ZERO {
                        p_whole.1 = ev.1;
                    }
                }

                _ => {}
            }
        }

        if p_prepare.1 < p_prepare.0 {
            p_prepare.1 = p_prepare.0;
        }
        if p_cache.1 < p_cache.0 {
            p_cache.1 = p_cache.0;
        }
        if p_fetch.1 < p_fetch.0 {
            p_fetch.1 = p_fetch.0;
        }
        if p_whole.1 < p_whole.0 {
            dbgg!(&p_whole);
            p_whole.1 = p_whole.0;
        }

        print!("(");
        print!("Google Translate");
        print!(" | ");
        print!("PREPARE:{}ms", (p_prepare.1 - p_prepare.0).as_millis());
        print!(" | ");
        print!("CACHE:{}ms", (p_cache.1 - p_cache.0).as_millis());
        print!(" | ");
        print!("FETCH:{}ms", (p_fetch.1 - p_fetch.0).as_millis());
        print!(" | ");
        print!("ALL:{}ms", (p_whole.1 - p_whole.0).as_millis());
        print!(")");

        println!();
        io::stdout().flush().unwrap();

        dbgg!(&self.queue);
    }

    fn elap_twtl(&self) {
        let mut p_prepare = (Duration::ZERO, Duration::ZERO);
        let mut p_cache = (Duration::ZERO, Duration::ZERO);
        let mut p_fetch = (Duration::ZERO, Duration::ZERO);
        let mut p_whole = (Duration::ZERO, Duration::ZERO); // should be just sum of above?

        let is_filled = |arg: (Duration, Duration)| -> bool {
            arg.0 != Duration::ZERO && arg.1 != Duration::ZERO
        };

        let lasti = self.queue.len() - 1;
        for _i in 0..self.queue.len() {
            if is_filled(p_prepare)
                && is_filled(p_cache)
                && is_filled(p_fetch)
                && is_filled(p_whole)
            {
                break;
            }
            let i = lasti - _i;
            let ev = &self.queue[i];
            match ev.0 {
                MafaEvent::Initialize { cate, is_fin } => {
                    if is_fin {
                        if cate == Category::Twtl && p_prepare.1 == Duration::ZERO {
                            p_prepare.1 = ev.1;
                        }
                    } else {
                        if cate == Category::Twtl && p_prepare.0 == Duration::ZERO {
                            p_prepare.0 = ev.1;
                            p_whole.0 = ev.1;
                        }
                    }
                }

                MafaEvent::BuildCache { cate, is_fin } => {
                    if is_fin {
                        if cate == Category::Twtl && p_cache.1 == Duration::ZERO {
                            p_cache.1 = ev.1;
                        }
                    } else {
                        if cate == Category::Twtl && p_cache.0 == Duration::ZERO {
                            p_cache.0 = ev.1;
                        }
                    }
                }

                MafaEvent::FetchResult { cate, is_fin } => {
                    if is_fin {
                        if cate == Category::Twtl && p_fetch.1 == Duration::ZERO {
                            p_fetch.1 = ev.1;
                        }
                    } else {
                        if cate == Category::Twtl && p_fetch.0 == Duration::ZERO {
                            p_fetch.0 = ev.1;
                        }
                    }
                }

                MafaEvent::ExactWhatRequest { cate, .. } => {
                    if cate == Category::Twtl && p_whole.1 == Duration::ZERO {
                        p_whole.1 = ev.1;
                    }
                }

                _ => {}
            }
        }

        if p_prepare.1 < p_prepare.0 {
            p_prepare.1 = p_prepare.0;
        }
        if p_cache.1 < p_cache.0 {
            p_cache.1 = p_cache.0;
        }
        if p_fetch.1 < p_fetch.0 {
            p_fetch.1 = p_fetch.0;
        }
        if p_whole.1 < p_whole.0 {
            p_whole.1 = p_whole.0;
        }

        print!("(");
        print!("Twtl");
        print!(" | ");
        print!("PREPARE:{}ms", (p_prepare.1 - p_prepare.0).as_millis());
        print!(" | ");
        print!("CACHE:{}ms", (p_cache.1 - p_cache.0).as_millis());
        print!(" | ");
        print!("FETCH:{}ms", (p_fetch.1 - p_fetch.0).as_millis());
        print!(" | ");
        print!("ALL:{}ms", (p_whole.1 - p_whole.0).as_millis());
        print!(")");

        println!();
        io::stdout().flush().unwrap();

        dbgg!(&self.queue);
    }
}
