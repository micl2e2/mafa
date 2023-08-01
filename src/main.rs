// Copyright (C) 2023 Michael Lee <micl2e2@proton.me>
//
// Licensed under the GNU General Public License, Version 3.0 or any later
// version <LICENSE-GPL or https://www.gnu.org/licenses/gpl-3.0.txt>.
//
// This file may not be copied, modified, or distributed except in compliance
// with the license.
//

use wda::GeckoDriver;
use wda::WdaError;
use wda::WebDrvAstn;

use mafa::MafaClient;

use std::sync::Arc;
use std::sync::Mutex;

#[macro_use]
mod private_macros;

use mafa::mafadata::MafaData;

#[cfg(any(feature = "gtrans", feature = "twtl", feature = "camd"))]
use mafa::{
    error::{MafaError, Result},
    ev_ntf::EurKind,
};

use mafa::ev_ntf::Category;
use mafa::ev_ntf::EventNotifier;
use mafa::ev_ntf::MafaEvent;

use mafa::MafaInput;

#[cfg(feature = "imode")]
use rustyline::{error::ReadlineError, DefaultEditor};

#[cfg(feature = "gtrans")]
use mafa::gtrans::GtransInput;

#[cfg(feature = "twtl")]
use mafa::twtl::TwtlInput;

#[cfg(feature = "camd")]
use mafa::camd::CamdInput;

fn main() {
    let mut exit_code = 0;

    let mafad = MafaData::init();
    let cmd_mafa = mafa::get_cmd();
    let ntf = EventNotifier::new();
    let ntf = Arc::new(Mutex::new(ntf));
    let m = cmd_mafa.try_get_matches();

    match m {
        Ok(matched) => match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => {
                let mut ignore_subcmd = false;

                if mafa_in.silent {
                    ntf.lock().expect("bug").set_silent();
                }

                if mafa_in.nocolor {
                    ntf.lock().expect("bug").set_nocolor();
                }

                dbgg!(&mafa_in);

                // init wda
                ntf.lock().expect("bug").notify(MafaEvent::Initialize {
                    cate: Category::Mafa,
                    is_fin: false,
                });
                let wda_inst = mafa::init_wda(&mafa_in);

                if let Err(e) = wda_inst {
                    match e {
                        MafaError::InvalidUseProfile
                        | MafaError::FirefoxNotFound
                        | MafaError::UnexpectedWda(_) => {
                            ntf.lock().expect("bug").notify(MafaEvent::FatalMafaError {
                                cate: Category::Mafa,
                                err: e,
                            });

                            exit_code = 7;
                        }
                        _ => {
                            ntf.lock().expect("bug").notify(MafaEvent::HandlerMissed {
                                cate: Category::Mafa,
                                err: e,
                            });

                            exit_code = 7;
                        }
                    }
                } else {
                    // finish things left
                    let wda_inst = wda_inst.expect("bug");
                    ntf.lock().expect("bug").notify(MafaEvent::Initialize {
                        cate: Category::Mafa,
                        is_fin: true,
                    });

                    // needs alive wda
                    if mafa_in.list_profile {
                        let list_got = format!(
                            "------ Available Profiles ------
{}
--------------------------------",
                            wda_inst
                                .existing_profiles()
                                .expect("bug")
                                .iter()
                                .map(|v| format!("<{v}>"))
                                .collect::<Vec<String>>()
                                .join("\n")
                        );

                        ntf.lock()
                            .expect("buggy")
                            .notify(MafaEvent::ExactUserRequest {
                                cate: Category::Mafa,
                                kind: EurKind::ListProfile,
                                output: list_got,
                            });

                        ignore_subcmd = true;
                        exit_code = 0;
                    }

                    // subcommand
                    if !ignore_subcmd {
                        match matched.subcommand() {
                            #[cfg(feature = "gtrans")]
                            Some(("gtrans", sub_m)) => {
                                let gtrans_in = GtransInput::from_ca_matched(sub_m);
                                exit_code = workflow_gtrans(
                                    &mafad,
                                    &mafa_in,
                                    gtrans_in,
                                    &wda_inst,
                                    Arc::clone(&ntf),
                                );
                            }

                            #[cfg(feature = "twtl")]
                            Some(("twtl", sub_m)) => {
                                let twtl_in = TwtlInput::from_ca_matched(sub_m);
                                exit_code = workflow_twtl(
                                    &mafad,
                                    &mafa_in,
                                    twtl_in,
                                    &wda_inst,
                                    Arc::clone(&ntf),
                                );
                            }

                            #[cfg(feature = "camd")]
                            Some(("camd", sub_m)) => {
                                let camd_in = CamdInput::from_ca_matched(sub_m);
                                exit_code = workflow_camd(
                                    &mafad,
                                    &mafa_in,
                                    camd_in,
                                    &wda_inst,
                                    Arc::clone(&ntf),
                                );
                            }

                            #[cfg(feature = "imode")]
                            Some(("i", _)) => {
                                exit_code =
                                    enter_i_mode(&mafad, &mafa_in, &wda_inst, Arc::clone(&ntf));
                            }

                            _ => {
                                ntf.lock()
                                    .expect("buggy")
                                    .notify(MafaEvent::ExactUserRequest {
                                    cate: Category::Mafa,
                                    kind: EurKind::NoSubCmd,
                                    output:
                                        "no components supplied. Please check supported components by -h or --help"
                                            .into(),
                                });
                            }
                        }
                    }
                }
            }
            Err(err_in) => {
                ntf.lock()
                    .expect("buggy") // FIXME: handle gracefully
                    .notify(MafaEvent::FatalMafaError {
                        cate: Category::Mafa,
                        err: err_in,
                    });
                exit_code = 5;
            }
        },
        Err(err_match) => {
            err_match.print().unwrap(); // this will print helper
        }
    }

    drop(mafad);
    drop(ntf);

    std::process::exit(exit_code as i32);
}

#[cfg(feature = "imode")]
fn enter_i_mode(
    mafad: &MafaData,
    mafa_in: &MafaInput,
    wda_inst: &WebDrvAstn<GeckoDriver>,
    ntf: Arc<Mutex<EventNotifier>>,
) -> u8 {
    let mut rl = DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline("[mafa]>> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                match line.as_str() {
                    #[cfg(feature = "gtrans")]
                    "gtrans" => {
                        if let Err(_err_imode) =
                            gtrans_i_mode(mafad, mafa_in, wda_inst, Arc::clone(&ntf))
                        {
                            return 4;
                        } else {
                            continue;
                        }
                    }

                    #[cfg(feature = "twtl")]
                    "twtl" => {
                        if let Err(_err_imode) =
                            twtl_i_mode(mafad, mafa_in, wda_inst, Arc::clone(&ntf))
                        {
                            return 4;
                        } else {
                            continue;
                        }
                    }

                    #[cfg(feature = "camd")]
                    "camd" => {
                        if let Err(_err_imode) =
                            camd_i_mode(mafad, mafa_in, wda_inst, Arc::clone(&ntf))
                        {
                            return 4;
                        } else {
                            continue;
                        }
                    }

                    "clear" => {
                        rl.clear_screen().expect("buggy");
                        continue;
                    }

                    _other => {
                        let mut helper = String::from("");
                        helper += "Available commands under interactive mode:\n";
                        helper += "\n";
                        helper += "  help (Print help)\n";
                        helper += "  clear (Clear Screen)\n";
                        #[cfg(feature = "twtl")]
                        {
                            helper += "  twtl (Twitter Timeline)\n";
                        }
                        #[cfg(feature = "gtrans")]
                        {
                            helper += "  gtrans (Google Translate)\n";
                        }
                        #[cfg(feature = "camd")]
                        {
                            helper += "  camd (Cambridge Dictionary)\n";
                        }
                        ntf.lock()
                            .expect("buggy")
                            .notify(MafaEvent::ExactUserRequest {
                                cate: Category::Mafa,
                                kind: EurKind::ImodeHelper,
                                output: helper,
                            });

                        continue;
                    }
                }
            }

            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(_rl_err) => {
                dbgg!(_rl_err);
                break;
            }
        }
    }

    return 0;
}

#[cfg(all(feature = "imode", feature = "gtrans"))]
fn gtrans_i_mode(
    mafad: &MafaData,
    mafa_in: &MafaInput,
    wda_inst: &WebDrvAstn<GeckoDriver>,
    ntf: Arc<Mutex<EventNotifier>>,
) -> Result<()> {
    let mut rl = DefaultEditor::new().unwrap();
    let mut client: Option<MafaClient<GtransInput, mafa::gtrans::Upath>> = None;
    loop {
        let readline = rl.readline("[mafa-gtrans] >> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());

                if line.as_str() == "clear" {
                    rl.clear_screen().expect("buggy");
                    continue;
                }

                let splits = line.split_whitespace();
                let mut args = Vec::<&str>::new();
                args.push("gtrans");
                for split in splits {
                    args.push(split);
                }

                let gtrans_in = GtransInput::from_imode_args(args);

                match gtrans_in {
                    Ok(_) => {}
                    Err(err_in) => match err_in {
                        MafaError::InvalidTimeoutPageLoad
                        | MafaError::InvalidTimeoutScript
                        | MafaError::InvalidSocks5Proxy
                        | MafaError::InvalidSourceLang
                        | MafaError::InvalidTargetLang
                        | MafaError::ClapMatchError(_) => {
                            lock_or_err!(ntf).notify(MafaEvent::FatalMafaError {
                                cate: Category::Gtrans,
                                err: err_in,
                            });

                            continue;
                        }

                        _ => {
                            lock_or_err!(ntf).notify(MafaEvent::HandlerMissed {
                                cate: Category::Gtrans,
                                err: err_in,
                            });

                            continue;
                        }
                    },
                }

                let gtrans_in = gtrans_in.expect("buggy");

                if client.is_none() {
                    client = Some(MafaClient::new(
                        mafad,
                        Arc::clone(&ntf),
                        mafa_in,
                        gtrans_in,
                        wda_inst,
                    ));
                } else {
                    client.as_mut().expect("bug").set_sub_input(gtrans_in);
                }

                match client.as_mut().expect("bug").handle(None) {
                    Ok((eurk, ret)) => {
                        lock_or_err!(ntf).notify(MafaEvent::ExactUserRequest {
                            cate: Category::Gtrans,
                            kind: eurk,
                            output: ret,
                        });

                        if mafa_in.elap {
                            lock_or_err!(ntf).elap(Category::Gtrans);
                        }

                        // return 0;
                        continue;
                    }

                    Err(err_hdl) => match err_hdl {
                        MafaError::AllCachesInvalid
                        | MafaError::DataFetchedNotReachable
                        | MafaError::WebDrvCmdRejected(_, _)
                        | MafaError::UnexpectedWda(_)
                        | MafaError::CacheRebuildFail(_) => {
                            lock_or_err!(ntf).notify(MafaEvent::FatalMafaError {
                                cate: Category::Gtrans,
                                err: err_hdl,
                            });

                            // return 3;
                            continue;
                        }

                        _ => {
                            lock_or_err!(ntf).notify(MafaEvent::HandlerMissed {
                                cate: Category::Gtrans,
                                err: err_hdl,
                            });

                            // return 3;
                            continue;
                        }
                    },
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(_rl_err) => {
                dbgg!(_rl_err);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(all(feature = "imode", feature = "twtl"))]
fn twtl_i_mode(
    mafad: &MafaData,
    mafa_in: &MafaInput,
    wda_inst: &WebDrvAstn<GeckoDriver>,
    ntf: Arc<Mutex<EventNotifier>>,
) -> Result<()> {
    let mut rl = DefaultEditor::new().unwrap();
    let mut client: Option<MafaClient<TwtlInput, mafa::twtl::UlPath>> = None;

    loop {
        let readline = rl.readline("[mafa-twtl] >> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());

                if line.as_str() == "clear" {
                    rl.clear_screen().expect("buggy");
                    continue;
                }

                let splits = line.split_whitespace();
                let mut args = Vec::<&str>::new();
                args.push("twtl");
                for split in splits {
                    args.push(split);
                }

                let twtl_in = TwtlInput::from_imode_args(args);

                match twtl_in {
                    Ok(_) => {}
                    Err(err_in) => match err_in {
                        MafaError::InvalidTimeoutPageLoad
                        | MafaError::InvalidTimeoutScript
                        | MafaError::InvalidSocks5Proxy
                        | MafaError::InvalidNumTweets
                        | MafaError::InvalidWrapWidth
                        | MafaError::ClapMatchError(_) => {
                            lock_or_err!(ntf).notify(MafaEvent::FatalMafaError {
                                cate: Category::Twtl,
                                err: err_in,
                            });
                            continue;
                        }
                        _ => {
                            lock_or_err!(ntf).notify(MafaEvent::HandlerMissed {
                                cate: Category::Twtl,
                                err: err_in,
                            });
                            continue;
                        }
                    },
                }

                let twtl_in = twtl_in.expect("buggy");

                if client.is_none() {
                    client = Some(MafaClient::new(
                        mafad,
                        Arc::clone(&ntf),
                        mafa_in,
                        twtl_in,
                        wda_inst,
                    ));
                } else {
                    client.as_mut().expect("bug").set_sub_input(twtl_in);
                }

                match client.as_mut().expect("bug").handle(None) {
                    Ok((ewrk, ret)) => {
                        lock_or_err!(ntf).notify(MafaEvent::ExactUserRequest {
                            cate: Category::Twtl,
                            kind: ewrk,
                            output: ret,
                        });

                        if mafa_in.elap {
                            lock_or_err!(ntf).elap(Category::Twtl);
                        }

                        // return Ok(());
                        continue;
                    }

                    Err(err_hdl) => match err_hdl {
                        MafaError::RequireLogin
                        | MafaError::MustGui
                        | MafaError::TweetNotRecoginized(_)
                        | MafaError::AllCachesInvalid
                        | MafaError::DataFetchedNotReachable
                        | MafaError::WebDrvCmdRejected(_, _)
                        | MafaError::UnexpectedWda(_)
                        | MafaError::CacheRebuildFail(_) => {
                            lock_or_err!(ntf).notify(MafaEvent::FatalMafaError {
                                cate: Category::Twtl,
                                err: err_hdl,
                            });
                            // return 3;
                            continue;
                        }

                        _ => {
                            lock_or_err!(ntf).notify(MafaEvent::HandlerMissed {
                                cate: Category::Twtl,
                                err: err_hdl,
                            });
                            // return 3;
                            continue;
                        }
                    },
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                // println!("CTRL-C");
                break;
            }
            Err(_rl_err) => {
                dbgg!(_rl_err);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(all(feature = "imode", feature = "camd"))]
fn camd_i_mode(
    mafad: &MafaData,
    mafa_in: &MafaInput,
    wda_inst: &WebDrvAstn<GeckoDriver>,
    ntf: Arc<Mutex<EventNotifier>>,
) -> Result<()> {
    let mut rl = DefaultEditor::new().unwrap();
    let mut client: Option<MafaClient<CamdInput, mafa::camd::Upath>> = None;

    loop {
        let readline = rl.readline("[mafa-camd] >> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());

                if line.as_str() == "clear" {
                    rl.clear_screen().expect("buggy");
                    continue;
                }

                let splits = line.split_whitespace();
                let mut args = Vec::<&str>::new();
                args.push("camd");
                for split in splits {
                    args.push(split);
                }

                let camd_in = CamdInput::from_imode_args(args);

                match camd_in {
                    Ok(_) => {}
                    Err(err_in) => match err_in {
                        MafaError::InvalidTimeoutPageLoad
                        | MafaError::InvalidTimeoutScript
                        | MafaError::InvalidSocks5Proxy
                        | MafaError::ClapMatchError(_) => {
                            lock_or_err!(ntf).notify(MafaEvent::FatalMafaError {
                                cate: Category::Camd,
                                err: err_in,
                            });

                            continue;
                        }

                        _ => {
                            lock_or_err!(ntf).notify(MafaEvent::HandlerMissed {
                                cate: Category::Camd,
                                err: err_in,
                            });

                            continue;
                        }
                    },
                }

                let camd_in = camd_in.expect("buggy");

                if client.is_none() {
                    client = Some(MafaClient::new(
                        mafad,
                        Arc::clone(&ntf),
                        mafa_in,
                        camd_in,
                        wda_inst,
                    ));
                } else {
                    client.as_mut().expect("bug").set_sub_input(camd_in);
                }

                match client.as_mut().expect("bug").handle(None) {
                    Ok((eurk, ret)) => {
                        lock_or_err!(ntf).notify(MafaEvent::ExactUserRequest {
                            cate: Category::Camd,
                            kind: eurk,
                            output: ret,
                        });

                        if mafa_in.elap {
                            lock_or_err!(ntf).elap(Category::Camd);
                        }

                        // return 0;
                        continue;
                    }

                    Err(err_hdl) => match err_hdl {
                        MafaError::AllCachesInvalid
                        | MafaError::DataFetchedNotReachable
                        | MafaError::WebDrvCmdRejected(_, _)
                        | MafaError::UnexpectedWda(_)
                        | MafaError::CacheRebuildFail(_) => {
                            lock_or_err!(ntf).notify(MafaEvent::FatalMafaError {
                                cate: Category::Camd,
                                err: err_hdl,
                            });

                            // return 3;
                            continue;
                        }

                        _ => {
                            lock_or_err!(ntf).notify(MafaEvent::HandlerMissed {
                                cate: Category::Camd,
                                err: err_hdl,
                            });

                            // return 3;
                            continue;
                        }
                    },
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(_rl_err) => {
                dbgg!(_rl_err);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(feature = "gtrans")]
fn workflow_gtrans(
    mafad: &MafaData,
    mafa_in: &MafaInput,
    gtrans_in: Result<GtransInput>,
    wda_inst: &WebDrvAstn<GeckoDriver>,
    ntf: Arc<Mutex<EventNotifier>>,
) -> u8 {
    if let Err(err_in) = gtrans_in {
        match err_in {
            MafaError::InvalidTimeoutPageLoad
            | MafaError::InvalidTimeoutScript
            | MafaError::InvalidSocks5Proxy
            | MafaError::InvalidSourceLang
            | MafaError::InvalidTargetLang => {
                lock_or_rtn!(ntf).notify(MafaEvent::FatalMafaError {
                    cate: Category::Gtrans,
                    err: err_in,
                });

                return 1;
            }

            _ => {
                lock_or_rtn!(ntf).notify(MafaEvent::HandlerMissed {
                    cate: Category::Gtrans,
                    err: err_in,
                });

                return 1;
            }
        }
    }

    let gtrans_in = gtrans_in.expect("buggy");

    let mut client = MafaClient::new(mafad, Arc::clone(&ntf), mafa_in, gtrans_in, wda_inst);

    match client.handle(None) {
        Ok((eurk, ret)) => {
            lock_or_rtn!(ntf).notify(MafaEvent::ExactUserRequest {
                cate: Category::Gtrans,
                kind: eurk,
                output: ret,
            });

            if mafa_in.elap {
                lock_or_rtn!(ntf).elap(Category::Gtrans);
            }

            return 0;
        }
        Err(err_hdl) => match err_hdl {
            MafaError::AllCachesInvalid
            | MafaError::DataFetchedNotReachable
            | MafaError::WebDrvCmdRejected(_, _)
            | MafaError::UnexpectedWda(_)
            | MafaError::CacheRebuildFail(_) => {
                lock_or_rtn!(ntf).notify(MafaEvent::FatalMafaError {
                    cate: Category::Gtrans,
                    err: err_hdl,
                });

                return 3;
            }

            _ => {
                lock_or_rtn!(ntf).notify(MafaEvent::HandlerMissed {
                    cate: Category::Gtrans,
                    err: err_hdl,
                });

                return 3;
            }
        },
    }

    return 0;
}

#[cfg(feature = "twtl")]
fn workflow_twtl(
    mafad: &MafaData,
    mafa_in: &MafaInput,
    twtl_in: Result<TwtlInput>,
    wda_inst: &WebDrvAstn<GeckoDriver>,
    ntf: Arc<Mutex<EventNotifier>>,
) -> u8 {
    if let Err(err_in) = twtl_in {
        match err_in {
            MafaError::InvalidTimeoutPageLoad
            | MafaError::InvalidTimeoutScript
            | MafaError::InvalidSocks5Proxy
            | MafaError::InvalidNumTweets
            | MafaError::InvalidWrapWidth => {
                lock_or_rtn!(ntf).notify(MafaEvent::FatalMafaError {
                    cate: Category::Twtl,
                    err: err_in,
                });
                return 1;
            }
            _ => {
                lock_or_rtn!(ntf).notify(MafaEvent::HandlerMissed {
                    cate: Category::Twtl,
                    err: err_in,
                });
                return 1;
            }
        }
    }

    let twtl_in = twtl_in.expect("buggy");

    let mut client = MafaClient::new(mafad, Arc::clone(&ntf), mafa_in, twtl_in, wda_inst);

    match client.handle(None) {
        Ok((ewrk, ret)) => {
            lock_or_rtn!(ntf).notify(MafaEvent::ExactUserRequest {
                cate: Category::Twtl,
                kind: ewrk,
                output: ret,
            });

            if mafa_in.elap {
                lock_or_rtn!(ntf).elap(Category::Twtl);
            }

            return 0;
        }
        Err(err_hdl) => match err_hdl {
            MafaError::RequireLogin
            | MafaError::MustGui
            | MafaError::TweetNotRecoginized(_)
            | MafaError::AllCachesInvalid
            | MafaError::DataFetchedNotReachable
            | MafaError::WebDrvCmdRejected(_, _)
            | MafaError::UnexpectedWda(_)
            | MafaError::CacheRebuildFail(_) => {
                lock_or_rtn!(ntf).notify(MafaEvent::FatalMafaError {
                    cate: Category::Twtl,
                    err: err_hdl,
                });
                return 3;
            }

            _ => {
                lock_or_rtn!(ntf).notify(MafaEvent::HandlerMissed {
                    cate: Category::Twtl,
                    err: err_hdl,
                });
                return 3;
            }
        },
    }

    // return 0;
}

#[cfg(feature = "camd")]
fn workflow_camd(
    mafad: &MafaData,
    mafa_in: &MafaInput,
    camd_in: Result<CamdInput>,
    wda_inst: &WebDrvAstn<GeckoDriver>,
    ntf: Arc<Mutex<EventNotifier>>,
) -> u8 {
    if let Err(err_in) = camd_in {
        match err_in {
            MafaError::InvalidTimeoutPageLoad
            | MafaError::InvalidTimeoutScript
            | MafaError::InvalidSocks5Proxy
            | MafaError::InvalidSourceLang
            | MafaError::InvalidTargetLang => {
                lock_or_rtn!(ntf).notify(MafaEvent::FatalMafaError {
                    cate: Category::Camd,
                    err: err_in,
                });

                return 1;
            }

            _ => {
                lock_or_rtn!(ntf).notify(MafaEvent::HandlerMissed {
                    cate: Category::Camd,
                    err: err_in,
                });

                return 1;
            }
        }
    }

    let camd_in = camd_in.expect("buggy");

    let mut client = MafaClient::new(mafad, Arc::clone(&ntf), mafa_in, camd_in, wda_inst);

    match client.handle(None) {
        Ok((eurk, ret)) => {
            lock_or_rtn!(ntf).notify(MafaEvent::ExactUserRequest {
                cate: Category::Camd,
                kind: eurk,
                output: ret,
            });

            if mafa_in.elap {
                lock_or_rtn!(ntf).elap(Category::Camd);
            }

            return 0;
        }
        Err(err_hdl) => match err_hdl {
            MafaError::AllCachesInvalid
            | MafaError::DataFetchedNotReachable
            | MafaError::WebDrvCmdRejected(_, _)
            | MafaError::UnexpectedWda(_)
            | MafaError::CacheRebuildFail(_) => {
                lock_or_rtn!(ntf).notify(MafaEvent::FatalMafaError {
                    cate: Category::Camd,
                    err: err_hdl,
                });

                return 3;
            }

            _ => {
                lock_or_rtn!(ntf).notify(MafaEvent::HandlerMissed {
                    cate: Category::Camd,
                    err: err_hdl,
                });

                return 3;
            }
        },
    }

    // return 0;
}

//
