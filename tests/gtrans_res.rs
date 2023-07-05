#[cfg(feature = "gtrans")]
mod gtrans_res {
    use std::sync::Arc;
    use std::sync::Mutex;

    use mafa;
    use mafa::ev_ntf::EurKind;
    use mafa::ev_ntf::EventNotifier;
    use mafa::gtrans::GtransClient;
    use mafa::gtrans::GtransInput;
    use mafa::mafadata::MafaData;
    use mafa::MafaInput;

    #[test]
    fn _1() {
        // translate result

        let pxy = if let Ok(v) = std::env::var("SOCKS5_PROXY") {
            v
        } else {
            "".to_string()
        };

        let matched = mafa::get_cmd().try_get_matches_from(vec![
            "mafa",
            "--socks5",
            &pxy,
            "gtrans",
            "--tl",
            "ja",
            "thank you",
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
                    match ag.handle(None) {
                        Ok((ewrk, ret)) => {
                            dbg!(&ret);
                            assert_eq!(ewrk, EurKind::GtransResult);
                            assert!(ret.contains("thank you"));
                            assert!(ret.contains("ありがとう"));
                        }
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

    #[test]
    fn _2() {
        // list lang

        let matched = mafa::get_cmd().try_get_matches_from(vec!["mafa", "gtrans", "--list-lang"]);
        let matched = matched.expect("must ok");

        match MafaInput::from_ca_matched(&matched) {
            Ok(mafa_in) => match matched.subcommand() {
                Some(("gtrans", sub_m)) => {
                    let mafad = MafaData::init();
                    let gtrans_in = GtransInput::from_ca_matched(sub_m).expect("must ok");
                    let ntf = Arc::new(Mutex::new(EventNotifier::new()));
                    let mut ag =
                        GtransClient::new(&mafad, ntf, &mafa_in, gtrans_in).expect("must ok");
                    match ag.handle(None) {
                        Ok((ewrk, ret)) => {
                            dbg!(&ret);
                            assert_eq!(ewrk, EurKind::GtransAllLang);
                            assert!(ret.contains("Irish: ga"));
                        }
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
