#[cfg(feature = "gtrans")]
mod gtrans_err {

    use std::sync::{Arc, Mutex};

    use mafa;
    use mafa::error::MafaError;
    use mafa::ev_ntf::EventNotifier;
    use mafa::gtrans::GtransClient;
    use mafa::gtrans::GtransInput;
    use mafa::mafadata::MafaData;
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
            "gtrans",
            "hello",
        ]);

        let matched = matched.expect("must ok");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let mafad = MafaData::init();
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let ntf = Arc::new(Mutex::new(EventNotifier::new()));
                    let ag = GtransClient::new(&mafad, ntf, &mafa_in, gtrans_in);
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
    fn at_new_2() {
        let matched = mafa::get_cmd().try_get_matches_from(vec!["mafa", "gtrans"]);

        let matched = matched.expect("must ok");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let mafad = MafaData::init();
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let ntf = Arc::new(Mutex::new(EventNotifier::new()));
                    let ag = GtransClient::new(&mafad, ntf, &mafa_in, gtrans_in);
                    if let Err(e) = ag {
                        match e {
                            MafaError::InvalidWords => {}
                            _ => assert!(false, "unexpected error {:?}", e),
                        }
                    } else {
                        assert!(false);
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
            // "--gui",
            "--socks5",
            &pxy,
            "--timeout-pageload",
            "90000",
            "--timeout-script",
            "10000",
            "gtrans",
            "hello",
        ]);
        let matched = matched.expect("must ok");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let mafad = MafaData::init();
                    // no way to gurantee the cache data the is read from a file
                    // thus put it in memory
                    let likely_failed_cache = vec![vec![1, 1, 1], vec![2, 2, 2]];
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let ntf = Arc::new(Mutex::new(EventNotifier::new()));
                    let mut ag =
                        GtransClient::new(&mafad, ntf, &mafa_in, gtrans_in).expect("must ok");
                    match ag.handle(Some(likely_failed_cache)) {
                        Ok(_) => assert!(false),
                        Err(e) => match e {
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
        // ~50s
        // no matter what caches we had and how valid they are, script timeout
        // cannot be too short

        let pxy = if let Ok(v) = std::env::var("SOCKS5_PROXY") {
            v
        } else {
            "".to_string()
        };

        let matched = mafa::get_cmd().try_get_matches_from(vec![
            "mafa",
            // "--gui",
            "--socks5",
            &pxy,
            "--timeout-pageload",
            "90000",
            "--timeout-script",
            "10",
            "gtrans",
            "hello",
        ]);
        let matched = matched.expect("must ok");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let mafad = MafaData::init();
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let ntf = Arc::new(Mutex::new(EventNotifier::new()));
                    let mut ag =
                        GtransClient::new(&mafad, ntf, &mafa_in, gtrans_in).expect("must ok");
                    if let Err(e) = ag.handle(None) {
                        match e {
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
