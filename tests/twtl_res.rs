#[cfg(feature = "twtl")]
mod twtl_res {
    use std::sync::Arc;
    use std::sync::Mutex;

    use mafa;
    use mafa::ev_ntf::EurKind;
    use mafa::ev_ntf::EventNotifier;
    use mafa::mafadata::MafaData;
    use mafa::twtl::TwtlClient;
    use mafa::twtl::TwtlInput;
    use mafa::MafaError;
    use mafa::MafaInput;

    #[test]
    fn _1() {
        // ~30s
        // tweet content

        let pxy = if let Ok(v) = std::env::var("SOCKS5_PROXY") {
            v
        } else {
            "".to_string()
        };

        let matched = mafa::get_cmd().try_get_matches_from(vec![
            "mafa", //
            // "--gui", //
            "--timeout-script",
            "10000",
            "--socks5",
            &pxy,
            "twtl",
            "mafa_rs",
        ]);
        let matched = matched.expect("must ok");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("twtl", sub_m)) => {
                    let mafad = MafaData::init();
                    let twtl_in = TwtlInput::from_ca_matched(sub_m).expect("must ok");
                    let ntf = Arc::new(Mutex::new(EventNotifier::new()));
                    let mut ag = TwtlClient::new(&mafad, ntf, &mafa_in, twtl_in).expect("must ok");
                    match ag.handle(None) {
                        Ok((ewrk, ret)) => {
                            dbg!(&ret);
                            assert_eq!(ewrk, EurKind::TwtlResult);
                            // NOTE: dont check pretty-printed result,
                            //       but raw content

                            // content
                            assert!(ret.contains("__________0__________"));
                            assert!(ret.contains("__________1__________"));
                            // timestamp
                            // assert!(ret.contains("???"));
                            // nickname
                            assert!(ret.contains("mafa"));
                            // username
                            assert!(ret.contains("mafa_rs"));
                        }
                        // Err(MafaError::RequireLogin) => {}
                        Err(e) => assert!(false, "unexpected error {:?}", e),
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
