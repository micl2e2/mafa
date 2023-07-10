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
pub struct RealExamp<'a>(Vec<Examp<'a>>);

#[derive(Debug, Default, PartialEq)]
struct Expl<'a> {
    nv_cate: Option<&'a str>,
    meaning: &'a str,
    usages: Vec<&'a str>,
}

#[derive(Debug, Default, PartialEq)]
struct Examp<'a> {
    usage: &'a str,
    from: &'a str,
}

#[derive(Debug, PartialEq)]
pub enum LevelExpained<'a> {
    DefaultKind(DefaultExpl<'a>),
    RealExampKind(RealExamp<'a>),
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

    fn from_str_internal_default(word: &str, lines: Vec<&'a str>, li_awl: usize) -> Result<Self> {
        let mut ret = DefaultExpl::default();

        // which kind
        let re_interme = Regex::new("INTERMEDIATE ENGLISH").expect("bug");
        let re_busi = Regex::new("BUSINESS ENGLISH").expect("bug");
        // pronun
        let mut li_pronu = usize::MAX;
        // let re_pronu = Regex::new("^U(S|K) */.*/( *U(K|S) */.*/)?$").expect("bug");
        let re_pronu = Regex::new("^US|K */.*/").expect("bug");
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
            ret.pronun = lines[li_pronu].trim();
        }
        dbgg!(&ret);

        let re_nv_cate1 = Regex::new(&format!(r"{} (noun|verb) \([/ A-Z]+\)", word)).expect("bug");
        let re_nv_cate2 =
            Regex::new(&format!(r"{} (noun|verb) (\[[CUSTI]\]) \([/ A-Z]+\)", word)).expect("bug");

        let is_nv_cate = |s| re_nv_cate1.is_match(s) || re_nv_cate2.is_match(s);
        let not_nv_cate = |s| !re_nv_cate1.is_match(s) && !re_nv_cate2.is_match(s);

        // let mut nex_nv_cate: Option<&str> =
        //     if re_nv_cate1.is_match(lines[li_awl - 1]) || re_nv_cate2.is_match(lines[li_awl - 1]) {
        //         Some(lines[li_awl - 1])
        //     } else {
        //         None
        //     };

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
                one_expl.nv_cate = if i - 3 > 0 && is_nv_cate(lines[i - 3]) {
                    Some(lines[i - 3])
                } else {
                    None
                };
                if i + 1 < lines.len() {
                    for j in i + 1..lines.len() {
                        if is_nv_cate(lines[j]) {
                            continue;
                        }
                        let nex_line_s = lines[j].to_ascii_lowercase();
                        if nex_line_s.contains(word) {
                            one_expl.usages.push(lines[j]);
                            i += 1;
                        } else if re_fewer_examples.is_match(lines[j]) {
                            one_expl.usages.push(lines[j]);
                            // we currently blindly include fewer examples
                            // and all of them will be thrown into last
                            // elem of usages
                            i += 1;
                        // }
                        // if re_usage.is_match(&nex_line_s) {
                        //     one_expl.usages.push(lines[j]);
                        //     i += 1;
                        // } else if re_fewer_examples.is_match(lines[j]) {
                        //     one_expl.usages.push(lines[j]);
                        //     // we currently blindly include fewer examples
                        //     // and all of them will be thrown into last
                        //     // elem of usages
                        //     i += 1;
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

    fn from_str_internal_examples(word: &str, lines: Vec<&'a str>) -> Result<Self> {
        let mut ret = RealExamp::default();

        if lines.len() < 3 {
            return Err(MafaError::CamdLevelNotRecoginized(1));
        }

        let re_0th_line = Regex::new(&format!("EXAMPLES of {}", word)).expect("bug");
        let re_1th_line = Regex::new(&format!("{}", word)).expect("bug");

        if !re_0th_line.is_match(lines[0]) || !re_1th_line.is_match(lines[1]) {
            return Err(MafaError::CamdLevelNotRecoginized(2));
        }

        let re_usage = Regex::new(&format!(r#"{}.*(\.|"|\?|!)$"#, word)).expect("bug");
        let re_from = Regex::new("^From .*").expect("bug");
        let mut i = 2;
        loop {
            if i + 1 >= lines.len() {
                break;
            }
            if re_usage.is_match(&lines[i].to_ascii_lowercase()) && re_from.is_match(lines[i + 1]) {
                let mut one_examp = Examp::default();
                one_examp.usage = lines[i];
                one_examp.from = lines[i + 1];
                ret.0.push(one_examp);
            }
            i += 2;
        }

        Ok(LevelExpained::RealExampKind(ret))
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
        let mut expl = DefaultExpl::default();
        expl.is_interme = false;
        expl.is_busi = false;
        expl.pronun = "US  /heˈloʊ/ UK  /heˈləʊ/";
        expl.expls.push(Expl {
            meaning: "used when meeting or greeting someone:",
            usages: vec![
                "Hello, Paul. I haven't seen you for ages.",
                "I know her vaguely - we've exchanged hellos a few times.",
                "I just thought I'd call by and say hello.",
                "And a big hello (= welcome) to all the parents who've come to see the show.",
            ],
            nv_cate: None,
        });
        expl.expls.push(Expl {
            meaning: "something that is said at the beginning of a phone conversation:",
            usages: vec![
                "\\\"Hello, I'd like some information about flights to the U.S., please.\\\"",
            ],
            nv_cate: None,
        });
        expl.expls.push(Expl {
            meaning: "something that is said to attract someone's attention:",
            usages: vec![
                "The front door was open so she walked inside and called out, \\\"Hello! Is there anybody in?\\\"",
            ],
            nv_cate: None,
        });
        expl.expls.push(Expl {
            meaning: "said to someone who has just said or done something stupid, especially something that shows they are not noticing what is happening:",
            usages: vec![
                "She asked me if I'd just arrived and I was like \\\"Hello, I've been here for an hour.\\\"",
            ],
            nv_cate: None,
        });
        expl.expls.push(Expl {
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
            nv_cate: None,
        });

        expected_camd_res.0.push(LevelExpained::DefaultKind(expl));

        // intermediate one
        let mut expl = DefaultExpl::default();
        expl.is_interme = true;
        expl.is_busi = false;
        expl.pronun = "US  /heˈloʊ, hə-/";
        expl.expls.push(Expl {
            meaning: "used when meeting or greeting someone:",
            usages: vec![
                "\\\"Hello, Paul,\\\" she said, \\\"I haven’t seen you for months.\\\"",
                "I know her vaguely – we’ve exchanged hellos a few times.",
                "Come and say hello to my friends (= meet them).",
            ],
            nv_cate: None,
        });
        expl.expls.push(Expl {
            meaning: "Hello is also said at the beginning of a telephone conversation.",
            usages: vec![],
            nv_cate: None,
        });
        expl.expls.push(Expl {
            meaning: "Hello is also used to attract someone’s attention:",
            usages: vec![
                "She walked into the shop and called out, \\\"Hello! Is anybody here?\\\"",
            ],
            nv_cate: None,
        });

        expected_camd_res.0.push(LevelExpained::DefaultKind(expl));

        // examples
        let mut expl = RealExamp::default();
        expl.0.push(Examp {
            usage: "She said that the highlight of her day was when she went up to say hello to one of the families.",
            from: "From Huffington Post",
        });
        expl.0.push(Examp {
            usage: "We learned to hear sorrow in one \\\"hello,\\\" and how to sit with each other without words.",
            from: "From Huffington Post",
        });
        expl.0.push(Examp {
            usage: "They say they hear him saying words like \\\"hello,\\\" even if others are skeptical, and say he responds to their attention.",
            from: "From ABC News",
        });
        expl.0.push(Examp {
            usage: "Usually, they get a response, and the second baseman will find his friends and say hello.",
            from: "From ESPN",
        });
        expl.0.push(Examp {
            usage: "Though he played a criminal on television, they say he was one of the nicest men and always waved hello.",
            from: "From CNN",
        });
        expl.0.push(Examp {
            usage: "Almost everybody stops by to say hello and chat.",
            from: "From Chicago Tribune",
        });
        expl.0.push(Examp {
            usage: "Hello, this is your friendly government authority here.",
            from: "From Gizmodo",
        });
        expl.0.push(Examp {
            usage: "Hello, please allow me to introduce myself, sir.",
            from: "From CNN",
        });
        expl.0.push(Examp {
            usage: "Walk up to him or her and do three things: smile, say hello, and listen.",
            from: "From Huffington Post",
        });
        expl.0.push(Examp {
            usage: "What are the characteristics of the way you say, \\\"hello,\\\" (or anything else for that matter) that makes you recognizable over the phone?",
            from: "From Phys.Org",
        });
        expl.0.push(Examp {
            usage: "Hello didn't become \\\"hi\\\" until the telephone arrived.",
            from: "From NPR",
        });

        expected_camd_res.0.push(LevelExpained::RealExampKind(expl));

        assert_eq!(camd_res, expected_camd_res);

        dbg!(&camd_res, &expected_camd_res);
    }

    #[test]
    fn _2() {
        let explained = "\"______ \\nworld\\nnoun\\nUS  /wɝːld/ UK  /wɜːld/\\nworld noun (THE EARTH)\\nAdd to word list \\nA1 [ S ]\\nthe earth and all the people, places, and things on it:\\nDifferent parts of the world have very different climatic conditions.\\nWhich bridge has the longest span in the world?\\nNews of the disaster shocked the (whole/entire) world.\\nWe live in a changing world and people must learn to adapt.\\nShe's a world authority on fetal development.\\na world record/championship\\n Fewer examples\\nPeople from different cultures have different conceptions of the world.\\nThe richer countries of the world should take concerted action to help the poorer countries.\\nI'm flirting with the idea of taking a year off and traveling round the world.\\nHe's one of the highest-earning professional golfers in the world.\\nThe museum's collection includes works of art from all around the world.\\n SMART Vocabulary: related words and phrases\\nworld noun (GROUP/AREA)\\n \\nB1 [ C usually singular ]\\na group of things such as countries or animals, or an area of human activity or understanding:\\nthe Muslim world\\nthe modern/industrialized world\\nthe animal world\\nstars from the rock music world\\nUnexpected things can happen in the world of subatomic particles.\\n More examples\\n SMART Vocabulary: related words and phrases\\nworld noun (PLANET)\\n \\n[ C ]\\na planet or other part of the universe, especially one where life might or does exist:\\nThere was a man on the news last night who believes we've been visited by beings from other worlds.\\n SMART Vocabulary: related words and phrases\\nIdioms\\nat one with the world\\nbe worlds apart\\ndo someone a world of good\\nfor all the world\\ngo/come down in the world\\ngo/come up in the world\\nhave the world at your feet\\nin a world of your own\\nmake a world of difference\\nmake the world go around/round\\n More idioms\\n(Definition of world from the Cambridge Advanced Learner's Dictionary & Thesaurus © Cambridge University Press)______world | INTERMEDIATE ENGLISH\\nworld\\nnoun\\nUS  /wɜrld/\\nworld noun (THE EARTH)\\nAdd to word list \\n[ U ]\\nthe planet on which human life has developed, esp. including all people and their ways of life:\\nPeople from all over the world will be attending the conference.\\nThe rapid growth of computers has changed the world.\\n \\n[ U ]\\nThe world can also mean the whole physical universe:\\nThe world contains many solar systems, not just ours.\\nworld noun (WHOLE AREA)\\n \\n[ C ]\\nall of a particular group or type of thing, such as countries or animals, or a whole area of human activity or understanding:\\nthe animal/plant world\\nthe business world\\nthe world of entertainment\\nIn the world of politics, the president’s voice is still the most powerful in the nation.\\nworld noun (LARGE DEGREE)\\n \\n[ U ]\\na large degree; a lot:\\nThere’s a world of difference between the two hotels.\\nIdioms\\nin a world of your own\\nin the world\\nman of the world\\n(Definition of world from the Cambridge Academic Content Dictionary © Cambridge University Press)______world | BUSINESS ENGLISH\\nworld\\nnoun [ C, usually singular ]\\nUK  /wɜːld/ US \\nAdd to word list \\na particular area of activity:\\nOur world of work is changing rapidly.\\nthe world of advertising/the internet\\nthe business/corporate world\\n(Definition of world from the Cambridge Business English Dictionary © Cambridge University Press)______EXAMPLES of world\\nworld\\nWhat happens in my life, in my world, doesn't have anything to do with you.\\nFrom NPR\\nMore than 300,000 podcasts exist in the world as of the close of 2015.\\nFrom The Atlantic\\nThis is the world we are headed toward.\\nFrom TIME\\nThe book goes out into the world, and who knows?\\nFrom The Atlantic\\nThis will make the world a better place.\\nFrom CNN\\nWe're all coming together towards making the world a better place.\\nFrom Voice of America\\nIt's what people in 3rd and 4th world countries do.\\nFrom CNN\\nAnd as the world for birds goes, our world can't be far behind.\\nFrom National Geographic\\nBudgets fool us into believing that they will not only tame us, but the world around us as well.\\nFrom New York Daily News\\nBut what in the world was that album all about?\\nFrom TIME\\nIt's a fun world to be a part of.\\nFrom VentureBeat\\nIt's bringing those worlds together that most interests me.\\nFrom NJ.com\\nIs the world a better place for having you and your work a part of it?\\nFrom Fast Company\\nStart paying attention to the physical world around you.\\nFrom Huffington Post\\nIt housed one of the world's important collections of arms and armor.\\nFrom CNBC\\nThese examples are from corpora and from sources on the web. Any opinions in the examples do not represent the opinion of the Cambridge Dictionary editors or of Cambridge University Press or its licensors.______COLLOCATIONS with world\\nworld\\n\\nThese are words often used in combination with world.\\n\\nClick on a collocation to see more examples of it.\\n\\nalien world\\nFrom everyday objects they built an alien world.\\nFrom the Cambridge English Corpus\\n\u{a0}\\nancient world\\nThe ancient world made a welcome reappearance in three theses.\\nFrom the Cambridge English Corpus\\n\u{a0}\\ncapitalist world\\nAcross the capitalist world the problem of cost containment has dominated health care since the mid-1970s.\\nFrom the Cambridge English Corpus\\n\u{a0}\\nThese examples are from corpora and from sources on the web. Any opinions in the examples do not represent the opinion of the Cambridge Dictionary editors or of Cambridge University Press or its licensors.\\nSee all collocations with world______What is the pronunciation of world?______\u{a0}\"";

        let camd_res = CamdResult::from_str("world", &explained).expect("bug");

        let mut expected_camd_res = CamdResult(vec![]);

        // primary one
        let mut expl = DefaultExpl::default();
        expl.is_interme = false;
        expl.is_busi = false;
        expl.pronun = "US  /wɝːld/ UK  /wɜːld/";
        expl.expls.push(Expl {
            meaning: "the earth and all the people, places, and things on it:",
            usages: vec![
                "Different parts of the world have very different climatic conditions.",
                "Which bridge has the longest span in the world?",
                "News of the disaster shocked the (whole/entire) world.",
                "We live in a changing world and people must learn to adapt.",
                "She's a world authority on fetal development.",
		"a world record/championship",
                " Fewer examples",
                "People from different cultures have different conceptions of the world.",
		"The richer countries of the world should take concerted action to help the poorer countries.",
		"I'm flirting with the idea of taking a year off and traveling round the world.",
		"He's one of the highest-earning professional golfers in the world.",
		"The museum's collection includes works of art from all around the world.",
            ],
            nv_cate: Some("world noun (THE EARTH)"),
        });
        expl.expls.push(Expl {
            meaning: "a group of things such as countries or animals, or an area of human activity or understanding:",
            usages: vec![
                "the Muslim world",
                "the modern/industrialized world",
                "the animal world",
                "stars from the rock music world",
                "Unexpected things can happen in the world of subatomic particles.",
            ],
            nv_cate: Some("world noun (GROUP/AREA)"),
        });
        expl.expls.push(Expl {
            meaning: "a planet or other part of the universe, especially one where life might or does exist:",
            usages: vec![
                "There was a man on the news last night who believes we've been visited by beings from other worlds.",
            ],
            nv_cate: Some("world noun (PLANET)"),
        });

        expected_camd_res.0.push(LevelExpained::DefaultKind(expl));

        // intermediate one
        let mut expl = DefaultExpl::default();
        expl.is_interme = true;
        expl.is_busi = false;
        expl.pronun = "US  /wɜrld/";
        expl.expls.push(Expl {
            meaning: "the planet on which human life has developed, esp. including all people and their ways of life:",
            usages: vec![
                "People from all over the world will be attending the conference.",
                "The rapid growth of computers has changed the world.",
            ],
            nv_cate: Some("world noun (THE EARTH)"),
        });
        expl.expls.push(Expl {
            meaning: "The world can also mean the whole physical universe:",
            usages: vec!["The world contains many solar systems, not just ours."],
            nv_cate: None,
        });
        expl.expls.push(Expl {
            meaning: "all of a particular group or type of thing, such as countries or animals, or a whole area of human activity or understanding:",
            usages: vec![
                "the animal/plant world",
                "the business world",
                "the world of entertainment",
                "In the world of politics, the president’s voice is still the most powerful in the nation.",
            ],
            nv_cate: Some("world noun (WHOLE AREA)"),
        });
        expl.expls.push(Expl {
            meaning: "a large degree; a lot:",
            usages: vec!["There’s a world of difference between the two hotels."],
            nv_cate: Some("world noun (LARGE DEGREE)"),
        });

        expected_camd_res.0.push(LevelExpained::DefaultKind(expl));

        // business one
        let mut expl = DefaultExpl::default();
        expl.is_interme = false;
        expl.is_busi = true;
        expl.pronun = "UK  /wɜːld/ US";
        expl.expls.push(Expl {
            meaning: "a particular area of activity:",
            usages: vec![
                "Our world of work is changing rapidly.",
                "the world of advertising/the internet",
                "the business/corporate world",
            ],
            nv_cate: None,
        });

        expected_camd_res.0.push(LevelExpained::DefaultKind(expl));

        // examples
        let mut expl = RealExamp::default();
        expl.0.push(Examp {
            usage: "What happens in my life, in my world, doesn't have anything to do with you.",
            from: "From NPR",
        });
        expl.0.push(Examp {
            usage: "More than 300,000 podcasts exist in the world as of the close of 2015.",
            from: "From The Atlantic",
        });
        expl.0.push(Examp {
            usage: "This is the world we are headed toward.",
            from: "From TIME",
        });
        expl.0.push(Examp {
            usage: "The book goes out into the world, and who knows?",
            from: "From The Atlantic",
        });
        expl.0.push(Examp {
            usage: "This will make the world a better place.",
            from: "From CNN",
        });
        expl.0.push(Examp {
            usage: "We're all coming together towards making the world a better place.",
            from: "From Voice of America",
        });
        expl.0.push(Examp {
            usage: "It's what people in 3rd and 4th world countries do.",
            from: "From CNN",
        });
        expl.0.push(Examp {
            usage: "And as the world for birds goes, our world can't be far behind.",
            from: "From National Geographic",
        });
        expl.0.push(Examp {
            usage: "Budgets fool us into believing that they will not only tame us, but the world around us as well.",
            from: "From New York Daily News",
        });
        expl.0.push(Examp {
            usage: "But what in the world was that album all about?",
            from: "From TIME",
        });
        expl.0.push(Examp {
            usage: "It's a fun world to be a part of.",
            from: "From VentureBeat",
        });
        expl.0.push(Examp {
            usage: "It's bringing those worlds together that most interests me.",
            from: "From NJ.com",
        });
        expl.0.push(Examp {
            usage: "Is the world a better place for having you and your work a part of it?",
            from: "From Fast Company",
        });
        expl.0.push(Examp {
            usage: "Start paying attention to the physical world around you.",
            from: "From Huffington Post",
        });
        expl.0.push(Examp {
            usage: "It housed one of the world's important collections of arms and armor.",
            from: "From CNBC",
        });

        expected_camd_res.0.push(LevelExpained::RealExampKind(expl));

        assert_eq!(camd_res, expected_camd_res);

        dbg!(&camd_res, &expected_camd_res);
    }
}
