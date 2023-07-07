#[cfg(feature = "twtl")]
mod twtl_err {

    use std::sync::{Arc, Mutex};

    use mafa;
    use mafa::error::MafaError;
    use mafa::ev_ntf::EventNotifier;
    use mafa::mafadata::MafaData;
    use mafa::twtl::TwtlClient;
    use mafa::twtl::TwtlInput;
    use mafa::MafaInput;
    use wda::WdaError::FetchWebDriver;

    #[test]
    fn at_new_1() {
        let matched = mafa::get_cmd().try_get_matches_from(vec![
            "mafa",
            "--socks5",
            "127.0.0.1:108x",
            "--timeout-script",
            "10",
            "twtl",
            "twitter",
        ]);

        let matched = matched.expect("must ok");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("twtl", sub_m)) => {
                    let mafad = MafaData::init();
                    let twtl_in = TwtlInput::from_ca_matched(sub_m).expect("must ok");
                    let ntf = Arc::new(Mutex::new(EventNotifier::new()));
                    let ag = TwtlClient::new(&mafad, ntf, &mafa_in, twtl_in);
                    if let Err(e) = ag {
                        match e {
                            MafaError::InvalidSocks5Proxy
                            | MafaError::UnexpectedWda(FetchWebDriver(_)) => {}
                            _ => assert!(false, "unexpected error {:?}", e),
                        }
                    }
                }
                _ => {
                    assert!(false);
                }
            },
            Err(_) => {
                assert!(false);
            }
        }
    }

    #[test]
    fn at_handle_1() {
        // MafaError::AllCachesInvalid
        // ~40s
        // normal case for all inuseable caches

        let pxy = if let Ok(v) = std::env::var("SOCKS5_PROXY") {
            v
        } else {
            "".to_string()
        };

        let matched = mafa::get_cmd().try_get_matches_from(vec![
            "mafa",
            "--socks5",
            &pxy,
            "--timeout-pageload",
            "10000",
            "--timeout-script",
            "10000",
            "twtl",
            "twitter",
        ]);
        let matched = matched.expect("must ok");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("twtl", sub_m)) => {
                    let mafad = MafaData::init();
                    // no way to gurantee the cache data the is read from a file
                    // thus put it in memory
                    let likely_failed_cache = vec![
                        vec![vec![100, 100], vec![100, 100]],
                        vec![vec![101, 100], vec![101, 100]],
                    ];
                    let twtl_in = TwtlInput::from_ca_matched(sub_m).expect("must ok");
                    let ntf = Arc::new(Mutex::new(EventNotifier::new()));
                    let mut ag = TwtlClient::new(&mafad, ntf, &mafa_in, twtl_in).expect("must ok");
                    match ag.handle(Some(likely_failed_cache)) {
                        Ok(res) => assert!(false, "ok: {:?}", res),
                        Err(e) => match e {
			    #[cfg(not(feature = "tst_twtl_logined"))]
                            MafaError::RequireLogin =>{}
                            MafaError::AllCachesInvalid => {}
                            _ => assert!(false, "unexpected error {:?}", e),
                        },
                    }
                }
                _ => {
                    assert!(false);
                }
            },
            Err(_) => {
                assert!(false);
            }
        }
    }

    #[test]
    fn at_handle_2() {
        // MafaError::AllCachesInvalid
        // script timeout is too small

        let pxy = if let Ok(v) = std::env::var("SOCKS5_PROXY") {
            v
        } else {
            "".to_string()
        };

        let matched = mafa::get_cmd().try_get_matches_from(vec![
            "mafa",
            "--socks5",
            &pxy,
            "--timeout-script",
            "10",
            "twtl",
            "twitter",
        ]);
        let matched = matched.expect("must ok");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("twtl", sub_m)) => {
                    let mafad = MafaData::init();
                    let twtl_in = TwtlInput::from_ca_matched(sub_m).expect("must ok");
                    let ntf = Arc::new(Mutex::new(EventNotifier::new()));
                    let mut ag = TwtlClient::new(&mafad, ntf, &mafa_in, twtl_in).expect("must ok");
                    if let Err(e) = ag.handle(None) {
                        match e {
			    #[cfg(not(feature = "tst_twtl_logined"))]
                            MafaError::RequireLogin =>{}
                            MafaError::AllCachesInvalid => {}
                            _ => assert!(false, "unexpected error {:?}", e),
                        }
                    }
                }
                _ => {
                    assert!(false);
                }
            },
            Err(_) => {
                assert!(false);
            }
        }
    }
}
