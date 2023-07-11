use crate::error::Result;

#[derive(Debug, Default)]
pub struct DefaultExpl<'a> {
    pronun: Option<&'a str>,
    expls: Vec<Expl<'a>>,
}

#[derive(Debug, Default)]
pub struct IntermeExpl<'a> {
    pronun: Option<&'a str>,
    expls: Vec<Expl<'a>>,
}

#[derive(Debug, Default)]
pub struct ExampExpl<'a> {
    pronun: Option<&'a str>,
    expls: Vec<Expl<'a>>,
}

#[derive(Debug, Default)]
struct Expl<'a> {
    meaning: &'a str,
    usages: Vec<&'a str>,
}

#[derive(Debug)]
pub enum LevelExpained<'a> {
    DefaultKind(DefaultExpl<'a>),
    IntermediaKind(IntermeExpl<'a>),
    ExamplesKind(ExampExpl<'a>),
}

impl LevelExpained<'_> {
    fn from_str(s: &str) -> Result<Self> {
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

        dbgg!(&lines);

        Ok(LevelExpained::DefaultKind(Default::default()))
    }
}

#[derive(Debug, Default)]
pub struct CamdResult<'a>(Vec<LevelExpained<'a>>);

impl CamdResult<'_> {
    pub fn from_str(s: &str) -> Result<Self> {
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

        dbgg!(begi_lv, endi_lv, &lv_expl_list);

        for lv_expl in lv_expl_list {
            let obj = LevelExpained::from_str(lv_expl);
        }

        Ok(Default::default())
    }
}

#[cfg(test)]
mod tst {

    #[test]
    fn _1() {
        let s = "\"______ \\nhello\\nexclamation, noun\\nUS  /heˈloʊ/ UK  /heˈləʊ/\\n(also mainly UK hallo); (hullo)\\nAdd to word list \\nA1\\nused when meeting or greeting someone:\\nHello, Paul. I haven't seen you for ages.\\nI know her vaguely - we've exchanged hellos a few times.\\nI just thought I'd call by and say hello.\\nAnd a big hello (= welcome) to all the parents who've come to see the show.\\n \\nA1\\nsomething that is said at the beginning of a phone conversation:\\n\\\"Hello, I'd like some information about flights to the U.S., please.\\\"\\n \\nsomething that is said to attract someone's attention:\\nThe front door was open so she walked inside and called out, \\\"Hello! Is there anybody in?\\\"\\n \\ninformal\\nsaid to someone who has just said or done something stupid, especially something that shows they are not noticing what is happening:\\nShe asked me if I'd just arrived and I was like \\\"Hello, I've been here for an hour.\\\"\\n \\nold-fashioned\\nan expression of surprise:\\nHello, this is very strange - I know that man.\\n Fewer examples\\nCathy poked her head round the door to say hello.\\nWhen he said hello, I felt my face turn bright red.\\nHello - could I speak to Ann, please?\\nAfter we'd said our hellos, it all went quiet and nobody knew what to do.\\nOh, hello - what are you doing in here?\\n SMART Vocabulary: related words and phrases\\nGrammar\\nGreetings and farewells: hello, goodbye, Happy New Year\\nWhen we see someone we know, we usually exchange greetings: …\\nSaying hello\\nWhen we see someone we know, we usually exchange greetings: …\\nSaying goodbye\\nWhen we leave people, we usually say something as we leave: …\\n(Definition of hello from the Cambridge Advanced Learner's Dictionary & Thesaurus © Cambridge University Press)______hello | INTERMEDIATE ENGLISH\\nhello\\nexclamation, noun [ C ]\\nUS  /heˈloʊ, hə-/\\nplural hellos\\nAdd to word list \\nused when meeting or greeting someone:\\n\\\"Hello, Paul,\\\" she said, \\\"I haven’t seen you for months.\\\"\\nI know her vaguely – we’ve exchanged hellos a few times.\\nCome and say hello to my friends (= meet them).\\n \\nHello is also said at the beginning of a telephone conversation.\\n \\nHello is also used to attract someone’s attention:\\nShe walked into the shop and called out, \\\"Hello! Is anybody here?\\\"\\n(Definition of hello from the Cambridge Academic Content Dictionary © Cambridge University Press)______EXAMPLES of hello\\nhello\\nShe said that the highlight of her day was when she went up to say hello to one of the families.\\nFrom Huffington Post\\nWe learned to hear sorrow in one \\\"hello,\\\" and how to sit with each other without words.\\nFrom Huffington Post\\nThey say they hear him saying words like \\\"hello,\\\" even if others are skeptical, and say he responds to their attention.\\nFrom ABC News\\nUsually, they get a response, and the second baseman will find his friends and say hello.\\nFrom ESPN\\nThough he played a criminal on television, they say he was one of the nicest men and always waved hello.\\nFrom CNN\\nAlmost everybody stops by to say hello and chat.\\nFrom Chicago Tribune\\nHello, this is your friendly government authority here.\\nFrom Gizmodo\\nHello, please allow me to introduce myself, sir.\\nFrom CNN\\nWalk up to him or her and do three things: smile, say hello, and listen.\\nFrom Huffington Post\\nWhat are the characteristics of the way you say, \\\"hello,\\\" (or anything else for that matter) that makes you recognizable over the phone?\\nFrom Phys.Org\\nHello didn't become \\\"hi\\\" until the telephone arrived.\\nFrom NPR\\nThese examples are from corpora and from sources on the web. Any opinions in the examples do not represent the opinion of the Cambridge Dictionary editors or of Cambridge University Press or its licensors.______What is the pronunciation of hello?______\u{a0}\"";
    }
}
