use regex::Regex;

use crate::error::Result;
use crate::MafaError;

#[derive(Debug, Default, PartialEq)]
pub struct DefaultExpl<'a> {
    is_interme: bool,
    is_busi: bool,
    pronun: &'a str,
    expls: Vec<Expl<'a>>,
}

// #[derive(Debug, Default)]
// pub struct IntermeExpl<'a> {
//     pronun: &'a str,
//     expls: Vec<Expl<'a>>,
// }

#[derive(Debug, Default, PartialEq)]
pub struct ExampExpl<'a> {
    pronun: &'a str,
    expls: Vec<Expl<'a>>,
}

#[derive(Debug, Default, PartialEq)]
struct Expl<'a> {
    meaning: &'a str,
    usages: Vec<&'a str>,
}

#[derive(Debug, PartialEq)]
pub enum LevelExpained<'a> {
    DefaultKind(DefaultExpl<'a>),
    ExamplesKind(ExampExpl<'a>),
}

impl<'a> LevelExpained<'a> {
    fn from_str(word: &str, s: &'a str) -> Result<Self> {
        let mut lines = Vec::<&str>::new();

        let bytes = s.as_bytes();

        let mut begi = 0usize;
        let mut endi = 0usize;

        for i in 0..bytes.len() {
            if i + 2 <= bytes.len() && &bytes[i..i + 2] == br"\n" {
                endi = i;
                lines.push(&s[begi..endi]);
                begi = endi + 2;
            }
        }

        if lines.len() == 0 {
            return Err(MafaError::CamdLevelNotRecoginized(1));
        }
        dbgg!(&lines);

        let mut li_awl = usize::MAX; // Add to word list
        let re_awl = Regex::new("Add to word list ").expect("bug");

        for i in 0..lines.len() {
            if re_awl.is_match(lines[i]) {
                li_awl = i;
                break;
            }
        }

        if li_awl < usize::MAX {
            // default, intermediate, business
            Self::from_str_internal_default(word, lines, li_awl)
        } else {
            // examples
            Self::from_str_internal_examples(word, lines)
        }
    }

    fn from_str_internal_examples(word: &str, lines: Vec<&str>) -> Result<Self> {
        Ok(LevelExpained::DefaultKind(Default::default()))
    }

    fn from_str_internal_default(word: &str, lines: Vec<&'a str>, li_awl: usize) -> Result<Self> {
        let mut ret = DefaultExpl::default();

        // which kind
        let re_interme = Regex::new("INTERMEDIATE ENGLISH").expect("bug");
        let re_busi = Regex::new("BUSINESS ENGLISH").expect("bug");
        // pronun
        let mut li_pronu = usize::MAX;
        let re_pronu = Regex::new("^US */.*/( *UK */.*/)?$").expect("bug");
        for i in 0..li_awl {
            if re_pronu.is_match(lines[i]) {
                li_pronu = i;
            }
            if re_interme.is_match(lines[i]) {
                ret.is_interme = true;
            }
            if re_busi.is_match(lines[i]) {
                ret.is_busi = true;
            }
        }
        if li_pronu < usize::MAX {
            ret.pronun = lines[li_pronu];
        }
        dbgg!(&ret);

        // usages
        let mut i = li_awl + 1;
        loop {
            if i == lines.len() {
                break;
            }
            let line = lines[i];
            let re_meaning = Regex::new(".*:$").expect("bug");
            let re_meaning_only = Regex::new(r#".*(\.|"|\?|!)$"#).expect("bug");
            let re_usage = Regex::new(&format!(r#"{}.*(\.|"|\?|!)$"#, word)).expect("bug");
            let re_fewer_examples = Regex::new("Fewer examples").expect("bug");
            if re_meaning.is_match(line) {
                // parse expl
                let mut one_expl = Expl::default();
                one_expl.meaning = line;
                if i + 1 < lines.len() {
                    for j in i + 1..lines.len() {
                        let nex_line_s = lines[j].to_ascii_lowercase();
                        if re_usage.is_match(&nex_line_s) {
                            one_expl.usages.push(lines[j]);
                            i += 1;
                        } else if re_fewer_examples.is_match(lines[j]) {
                            one_expl.usages.push(lines[j]);
                            // we currently blindly include fewer examples
                            // and all of them will be dropped into last
                            // elem of usages
                            i += 1;
                        } else {
                            dbgg!(lines[j]);
                            break;
                        }
                    }
                    dbgg!(&one_expl);
                }
                ret.expls.push(one_expl);
            } else if re_meaning_only.is_match(line) {
                let mut one_expl = Expl::default();
                one_expl.meaning = line;
                ret.expls.push(one_expl);
            }
            i += 1;
        }

        dbgg!(&ret);

        Ok(LevelExpained::DefaultKind(ret))
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct CamdResult<'a>(Vec<LevelExpained<'a>>);

impl<'a> CamdResult<'a> {
    pub fn from_str(word: &str, s: &'a str) -> Result<Self> {
        let bytes = s.as_bytes();
        let mut lv_expl_list = Vec::<&str>::new();

        let mut begi_lv = 0usize;
        let mut endi_lv = 0usize;
        for i in 0..bytes.len() {
            let cur_byte = bytes[i];
            if cur_byte == b'_' && i + 5 < bytes.len() {
                if bytes[i + 1] == b'_'
                    && bytes[i + 2] == b'_'
                    && bytes[i + 3] == b'_'
                    && bytes[i + 4] == b'_'
                    && bytes[i + 5] == b'_'
                {
                    endi_lv = i;
                    let lv_range = endi_lv - begi_lv;
                    if lv_range > 1 {
                        lv_expl_list.push(&s[begi_lv..endi_lv]);
                    }
                    begi_lv = endi_lv + 6;
                    endi_lv = begi_lv;
                }
            }
        }

        dbgg!(begi_lv, endi_lv, &lv_expl_list); // used for early debug

        let mut ret = Self::default();
        for lv_expl in lv_expl_list {
            if let Ok(obj) = LevelExpained::from_str(word, lv_expl) {
                ret.0.push(obj);
            }
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tst {
    use super::*;

    #[test]
    fn _1() {
        let explained = "\"______ \\nhello\\nexclamation, noun\\nUS  /heˈloʊ/ UK  /heˈləʊ/\\n(also mainly UK hallo); (hullo)\\nAdd to word list \\nA1\\nused when meeting or greeting someone:\\nHello, Paul. I haven't seen you for ages.\\nI know her vaguely - we've exchanged hellos a few times.\\nI just thought I'd call by and say hello.\\nAnd a big hello (= welcome) to all the parents who've come to see the show.\\n \\nA1\\nsomething that is said at the beginning of a phone conversation:\\n\\\"Hello, I'd like some information about flights to the U.S., please.\\\"\\n \\nsomething that is said to attract someone's attention:\\nThe front door was open so she walked inside and called out, \\\"Hello! Is there anybody in?\\\"\\n \\ninformal\\nsaid to someone who has just said or done something stupid, especially something that shows they are not noticing what is happening:\\nShe asked me if I'd just arrived and I was like \\\"Hello, I've been here for an hour.\\\"\\n \\nold-fashioned\\nan expression of surprise:\\nHello, this is very strange - I know that man.\\n Fewer examples\\nCathy poked her head round the door to say hello.\\nWhen he said hello, I felt my face turn bright red.\\nHello - could I speak to Ann, please?\\nAfter we'd said our hellos, it all went quiet and nobody knew what to do.\\nOh, hello - what are you doing in here?\\n SMART Vocabulary: related words and phrases\\nGrammar\\nGreetings and farewells: hello, goodbye, Happy New Year\\nWhen we see someone we know, we usually exchange greetings: …\\nSaying hello\\nWhen we see someone we know, we usually exchange greetings: …\\nSaying goodbye\\nWhen we leave people, we usually say something as we leave: …\\n(Definition of hello from the Cambridge Advanced Learner's Dictionary & Thesaurus © Cambridge University Press)______hello | INTERMEDIATE ENGLISH\\nhello\\nexclamation, noun [ C ]\\nUS  /heˈloʊ, hə-/\\nplural hellos\\nAdd to word list \\nused when meeting or greeting someone:\\n\\\"Hello, Paul,\\\" she said, \\\"I haven’t seen you for months.\\\"\\nI know her vaguely – we’ve exchanged hellos a few times.\\nCome and say hello to my friends (= meet them).\\n \\nHello is also said at the beginning of a telephone conversation.\\n \\nHello is also used to attract someone’s attention:\\nShe walked into the shop and called out, \\\"Hello! Is anybody here?\\\"\\n(Definition of hello from the Cambridge Academic Content Dictionary © Cambridge University Press)______EXAMPLES of hello\\nhello\\nShe said that the highlight of her day was when she went up to say hello to one of the families.\\nFrom Huffington Post\\nWe learned to hear sorrow in one \\\"hello,\\\" and how to sit with each other without words.\\nFrom Huffington Post\\nThey say they hear him saying words like \\\"hello,\\\" even if others are skeptical, and say he responds to their attention.\\nFrom ABC News\\nUsually, they get a response, and the second baseman will find his friends and say hello.\\nFrom ESPN\\nThough he played a criminal on television, they say he was one of the nicest men and always waved hello.\\nFrom CNN\\nAlmost everybody stops by to say hello and chat.\\nFrom Chicago Tribune\\nHello, this is your friendly government authority here.\\nFrom Gizmodo\\nHello, please allow me to introduce myself, sir.\\nFrom CNN\\nWalk up to him or her and do three things: smile, say hello, and listen.\\nFrom Huffington Post\\nWhat are the characteristics of the way you say, \\\"hello,\\\" (or anything else for that matter) that makes you recognizable over the phone?\\nFrom Phys.Org\\nHello didn't become \\\"hi\\\" until the telephone arrived.\\nFrom NPR\\nThese examples are from corpora and from sources on the web. Any opinions in the examples do not represent the opinion of the Cambridge Dictionary editors or of Cambridge University Press or its licensors.______What is the pronunciation of hello?______\u{a0}\"";

        let camd_res = CamdResult::from_str("hello", &explained).expect("bug");

        let mut expected_camd_res = CamdResult(vec![]);

        // primary one
        let mut default_expl = DefaultExpl::default();
        default_expl.is_interme = false;
        default_expl.is_busi = false;
        default_expl.pronun = "US  /heˈloʊ/ UK  /heˈləʊ/";
        default_expl.expls.push(Expl {
            meaning: "used when meeting or greeting someone:",
            usages: vec![
                "Hello, Paul. I haven't seen you for ages.",
                "I know her vaguely - we've exchanged hellos a few times.",
                "I just thought I'd call by and say hello.",
                "And a big hello (= welcome) to all the parents who've come to see the show.",
            ],
        });
        default_expl.expls.push(Expl {
            meaning: "something that is said at the beginning of a phone conversation:",
            usages: vec![
                "\\\"Hello, I'd like some information about flights to the U.S., please.\\\"",
            ],
        });
        default_expl.expls.push(Expl {
            meaning: "something that is said to attract someone's attention:",
            usages: vec![
                "The front door was open so she walked inside and called out, \\\"Hello! Is there anybody in?\\\"",
            ],
        });
        default_expl.expls.push(Expl {
            meaning: "said to someone who has just said or done something stupid, especially something that shows they are not noticing what is happening:",
            usages: vec![
                "She asked me if I'd just arrived and I was like \\\"Hello, I've been here for an hour.\\\"",
            ],
        });
        default_expl.expls.push(Expl {
            meaning: "an expression of surprise:",
            usages: vec![
                "Hello, this is very strange - I know that man.",
                " Fewer examples",
                "Cathy poked her head round the door to say hello.",
                "When he said hello, I felt my face turn bright red.",
                "Hello - could I speak to Ann, please?",
                "After we'd said our hellos, it all went quiet and nobody knew what to do.",
                "Oh, hello - what are you doing in here?",
            ],
        });

        expected_camd_res
            .0
            .push(LevelExpained::DefaultKind(default_expl));

        // intermediate one
        let mut default_expl = DefaultExpl::default();
        default_expl.is_interme = true;
        default_expl.is_busi = false;
        default_expl.pronun = "US  /heˈloʊ, hə-/";
        default_expl.expls.push(Expl {
            meaning: "used when meeting or greeting someone:",
            usages: vec![
                "\\\"Hello, Paul,\\\" she said, \\\"I haven’t seen you for months.\\\"",
                "I know her vaguely – we’ve exchanged hellos a few times.",
                "Come and say hello to my friends (= meet them).",
            ],
        });
        default_expl.expls.push(Expl {
            meaning: "Hello is also said at the beginning of a telephone conversation.",
            usages: vec![],
        });
        default_expl.expls.push(Expl {
            meaning: "Hello is also used to attract someone’s attention:",
            usages: vec![
                "She walked into the shop and called out, \\\"Hello! Is anybody here?\\\"",
            ],
        });

        expected_camd_res
            .0
            .push(LevelExpained::DefaultKind(default_expl));

        expected_camd_res
            .0
            .push(LevelExpained::DefaultKind(Default::default()));

        assert_eq!(camd_res, expected_camd_res);

        dbg!(&camd_res, &expected_camd_res);
    }
}
