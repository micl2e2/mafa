use regex::Regex;
use unicode_width::UnicodeWidthStr;

use crate::comm;
use crate::error::Result;
use crate::MafaError;

const LABELS: [&str; 67] = [
    "adjective",
    "[after noun]",
    "[after verb]",
    "[before noun]",
    "comparative",
    "superlative",
    "[not gradable]",
    "noun",
    "[C]",
    "[U]",
    "[S]",
    "plural",
    "noun [plural]",
    "[usually plural]",
    "[usually singular]",
    "[+ sing/pl verb]",
    "verb",
    "[T]",
    "[I]",
    "auxiliary verb",
    "modal verb",
    "past simple",
    "past participle",
    "present participle",
    "phrasal verb",
    "[L]",
    "[L only + adjective]",
    "[L only + noun]",
    "[+ adv/prep]",
    "[+ that clause]",
    "[+ question word]",
    "[+ speech]",
    "[+ to infinitive]",
    "[+ infinitive without to]",
    "[+ -ing] verb",
    "[+ not or so]",
    "[+ not or so]",
    "[+ two objects]",
    "[+ obj + adjective]",
    "[+ obj + noun]",
    "[+ obj + noun or adjective]",
    "[+ obj + as noun or adjective]",
    "[+ obj + to be noun or adjective]",
    "[+ obj + that clause]",
    "[+ obj + to infinitive]",
    "[+ obj + infinitive without to]",
    "[+ obj + past participle]",
    "[+ obj + ing verb]",
    "[+ obj + question word]",
    "[usually passive]",
    "[not continuous]",
    "adverb",
    "conjunction",
    "determiner",
    "number",
    "ordinal number",
    "preposition",
    "predeterminer",
    "pronoun",
    "prefix",
    "suffix",
    "exclamation",
    "[+ ing verb]",
    "[+ to infinitive]",
    "[+ that]",
    "[+ question word]",
    "[as form of address]",
];

#[derive(Debug, Default, PartialEq)]
pub struct DefaultExpl<'a> {
    is_interme: bool,
    is_busi: bool,
    pronun: &'a str,
    expls: Vec<Expl<'a>>,
}

impl DefaultExpl<'_> {
    fn pretty_print(&self, nocolor: bool, asciiful: bool, wrap_width: usize) -> Result<String> {
        let mut output = String::default();

        let mut part_hdl = String::default();
        if self.is_interme {
            part_hdl += if nocolor { "" } else { "\x1b[31;1m" };
            part_hdl += if asciiful { "---" } else { "───" };
            part_hdl += " I N T E R M E D I A T E ";
            part_hdl += if asciiful { "---" } else { "───" };
            part_hdl += if nocolor { "" } else { "\x1b[0m" };
            part_hdl += "\n";
        }
        if self.is_busi {
            part_hdl += if nocolor { "" } else { "\x1b[31;1m" };
            part_hdl += if asciiful { "---" } else { "───" };
            part_hdl += " B U S I N E S S ";
            part_hdl += if asciiful { "---" } else { "───" };
            part_hdl += if nocolor { "" } else { "\x1b[0m" };
            part_hdl += "\n";
        }
        part_hdl += "\n";

        // currently omit pronun
        output += &Self::pretty_pronun(self.pronun, nocolor);
        // output += (self.pronun);
        output += "\n";

        for expl in &self.expls {
            output += &expl.pretty_print(nocolor, asciiful, wrap_width)?;
            output += "\n";
        }

        if output.trim().len() == 0 {
            Ok(String::default())
        } else {
            output = part_hdl + &output;
            Ok(output)
        }
    }

    fn pretty_pronun(s: &str, nocolor: bool) -> String {
        let extracted = Self::extract_pronun(s);
        let mut ret = String::default();

        let prefix_us = if nocolor {
            "/US "
        } else {
            "/\x1b[36;1mUS\x1b[0m "
        };
        let prefix_uk = if nocolor {
            "/UK "
        } else {
            "/\x1b[35;1mUK\x1b[0m "
        };

        match extracted {
            (Some(us), Some(uk), None) => {
                ret += prefix_us;
                ret += &us[1..];
                ret += "  ";
                ret += prefix_uk;
                ret += &uk[1..];
            }
            (Some(us), None, None) => {
                ret += prefix_us;
                ret += &us[1..];
            }
            (None, Some(uk), None) => {
                ret += prefix_uk;
                ret += &uk[1..];
            }
            (None, None, Some(unknown)) => {
                ret += "Unknown Pronun  ";
                ret += unknown;
            }
            _ => ret += "Unsupported Pronun",
        }

        ret += "\n";

        ret
    }

    ///
    /// us,uk,unknown
    fn extract_pronun(s: &str) -> (Option<&str>, Option<&str>, Option<&str>) {
        let us_pronu = Regex::new("US *(/[^A-Z]*/) *$").expect("bug");
        let us2_pronu = Regex::new("US *(/[^A-Z]*/) *UK$").expect("bug");
        let uk_pronu = Regex::new("UK *(/[^A-Z]*/) *$").expect("bug");
        let uk2_pronu = Regex::new("UK *(/[^A-Z]*/) *US$").expect("bug");
        let us_uk_pronu = Regex::new("US *(/[^A-Z]*/) *UK *(/[^A-Z]*/)").expect("bug");

        let mut us_got: Option<&str> = None;
        let mut uk_got: Option<&str> = None;

        if let Some(v) = us_uk_pronu.captures(s) {
            us_got = Some(v.get(1).expect("bug").as_str());
            uk_got = Some(v.get(2).expect("bug").as_str());
        } else if let Some(v) = us_pronu.captures(s) {
            us_got = Some(v.get(1).expect("bug").as_str());
        } else if let Some(v) = us2_pronu.captures(s) {
            us_got = Some(v.get(1).expect("bug").as_str());
        } else if let Some(v) = uk_pronu.captures(s) {
            uk_got = Some(v.get(1).expect("bug").as_str());
        } else if let Some(v) = uk2_pronu.captures(s) {
            uk_got = Some(v.get(1).expect("bug").as_str());
        }

        if us_got.is_none() && uk_got.is_none() {
            (None, None, Some(s))
        } else {
            (us_got, uk_got, None)
        }
    }
}

// #[derive(Debug, Default)]
// pub struct IntermeExpl<'a> {
//     pronun: &'a str,
//     expls: Vec<Expl<'a>>,
// }

#[derive(Debug, Default, PartialEq)]
pub struct RealExamp<'a>(Vec<Examp<'a>>);
impl RealExamp<'_> {
    fn pretty_print(&self, nocolor: bool, asciiful: bool, wrap_width: usize) -> Result<String> {
        let mut output = String::default();

        output += if nocolor { "" } else { "\x1b[31;1m" };
        output += if asciiful { "---" } else { "───" };
        output += " E X A M P L E S ";
        output += if asciiful { "---" } else { "───" };
        output += if nocolor { "" } else { "\x1b[0m" };
        output += "\n";
        output += "\n";

        for ele in &self.0 {
            output += &ele.pretty_print(nocolor, asciiful, wrap_width)?;
            output += "\n";
        }

        Ok(output)
    }
}

#[derive(Debug, Default, PartialEq)]
struct Expl<'a> {
    nv_cate: Option<&'a str>,
    meaning: &'a str,
    usages: Vec<&'a str>,
}
impl Expl<'_> {
    fn pretty_print(&self, nocolor: bool, asciiful: bool, wrap_width: usize) -> Result<String> {
        let mut output = String::default();

        // meaning (need wrap) (need readable)
        let mut part_meaning = "".to_string();
        part_meaning += if asciiful { "* " } else { "✪ " };
        let w_leading = UnicodeWidthStr::width(part_meaning.as_str());
        part_meaning += if nocolor { "" } else { "\x1b[1m" };
        part_meaning += &comm::make_readable(self.meaning);
        part_meaning += if nocolor { "" } else { "\x1b[0m" };
        let mut wrapper = bwrap::EasyWrapper::new(&part_meaning, wrap_width - w_leading).unwrap();
        let txt_leading = comm::replicate(" ", w_leading);
        let wrapped_part_meaning = wrapper
            .wrap_use_style(bwrap::WrapStyle::NoBrk(
                Some(&txt_leading),
                bwrap::ExistNlPref::KeepTrailSpc,
            ))
            .unwrap();
        output += &wrapped_part_meaning;
        output += "\n";

        // label (need readable)
        if let Some(v) = self.nv_cate {
            output += "- HINT: ";
            output += &comm::make_readable(v);
            output += "\n";
        }

        // usages (need wrap) (need readable)
        for a_usage in &self.usages {
            let mut part_a_usage = String::default();
            part_a_usage += "- ";
            let w_leading = UnicodeWidthStr::width(part_a_usage.as_str());
            part_a_usage += &comm::make_readable(a_usage);
            let mut wrapper =
                bwrap::EasyWrapper::new(&part_a_usage, wrap_width - w_leading).unwrap();
            let txt_leading = comm::replicate(" ", w_leading);
            let wrapped_part_a_usage = wrapper
                .wrap_use_style(bwrap::WrapStyle::NoBrk(
                    Some(&txt_leading),
                    bwrap::ExistNlPref::KeepTrailSpc,
                ))
                .unwrap();
            output += &wrapped_part_a_usage;
            output += "\n";
        }

        Ok(output)
    }
}

#[derive(Debug, Default, PartialEq)]
struct Examp<'a> {
    usage: &'a str,
    from: &'a str,
}
impl Examp<'_> {
    fn pretty_print(&self, nocolor: bool, asciiful: bool, wrap_width: usize) -> Result<String> {
        let mut output = String::default();

        let mut part_a_usage = String::default();
        part_a_usage += "- ";
        let w_leading = UnicodeWidthStr::width(part_a_usage.as_str());
        part_a_usage += if nocolor { "" } else { "\x1b[1m" };
        part_a_usage += self.from;
        part_a_usage += if nocolor { "" } else { "\x1b[0m" };
        part_a_usage += ": ";
        part_a_usage += &comm::make_readable(self.usage);

        let mut wrapper = bwrap::EasyWrapper::new(&part_a_usage, wrap_width - w_leading).unwrap();
        let txt_leading = comm::replicate(" ", w_leading);
        let wrapped_part_a_usage = wrapper
            .wrap_use_style(bwrap::WrapStyle::NoBrk(
                Some(&txt_leading),
                bwrap::ExistNlPref::KeepTrailSpc,
            ))
            .unwrap();

        output += &wrapped_part_a_usage;

        Ok(output)
    }
}

#[derive(Debug, PartialEq)]
pub enum LevelExpained<'a> {
    DefaultKind(DefaultExpl<'a>, &'a str),
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

        if lines.len() < 2 {
            return Err(MafaError::CamdLevelNotRecoginized(2));
        }
        if lines[1].len() == 0 {
            return Err(MafaError::CamdLevelNotRecoginized(3));
        }
        let corrected_word = lines[1]; // e.g. fewer -> few

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
            Self::from_str_internal_default(corrected_word, s, lines, li_awl)
        } else {
            // examples
            Self::from_str_internal_examples(corrected_word, lines)
        }
    }

    fn from_str_internal_default(
        word: &str,
        s: &'a str,
        lines: Vec<&'a str>,
        li_awl: usize,
    ) -> Result<Self> {
        let mut ret = DefaultExpl::default();

        // which kind
        let re_interme = Regex::new("INTERMEDIATE ENGLISH").expect("bug");
        let re_busi = Regex::new("BUSINESS ENGLISH").expect("bug");
        // pronun
        let mut li_pronu = usize::MAX;
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
        let is_label = |s: &str| {
            if s.contains(word) {
                let mut ret = false;
                if s.contains(".")
                    || s.contains(";")
                    || s.contains("?")
                    || s.contains("\"")
                    || s.contains("'")
                    || s.contains("!")
                {
                    ret = false;
                } else {
                    for l in LABELS {
                        if s.contains(l) {
                            ret = true;
                            break;
                        }
                    }
                }
                ret
            } else {
                false
            }
        };

        // usages
        let mut i = li_awl + 1;
        loop {
            if i == lines.len() {
                break;
            }
            let line = lines[i];
            let re_meaning = Regex::new(".*:$").expect("bug");
            // let re_meaning_only = Regex::new(r#".*(\.|"|\?|!)$"#).expect("bug");
            let re_usage = Regex::new(&format!(r#"{}.*(\.|"|\?|!)$"#, word)).expect("bug");
            let re_fewer_examples = Regex::new("Fewer examples").expect("bug");

            let re_unusable = Regex::new("\u{a0}").expect("bug");

            if re_meaning.is_match(line) {
                dbgg!(&line);
                // parse expl
                let mut one_expl = Expl::default();
                one_expl.meaning = line;
                one_expl.nv_cate = if i - 2 > 0 && is_label(lines[i - 2]) {
                    Some(lines[i - 2]) // some are 2L before
                } else if i - 3 > 0 && is_label(lines[i - 3]) {
                    Some(lines[i - 3]) // some are 3L before
                } else {
                    None
                };
                if i + 1 < lines.len() {
                    for j in i + 1..lines.len() {
                        if is_nv_cate(lines[j]) {
                            continue;
                        }
                        let nex_line_s = lines[j].to_ascii_lowercase();
                        if nex_line_s.contains(word)
                            && !is_label(&nex_line_s)
                            && !re_unusable.is_match(lines[j])
                        {
                            one_expl.usages.push(lines[j]);
                            i += 1;
                        // }
                        // else if re_fewer_examples.is_match(lines[j]) {
                        // one_expl.usages.push(lines[j]);
                        // we currently blindly include fewer examples
                        // and all of them will be thrown into last
                        // elem of usages
                        // i += 1;
                        } else {
                            dbgg!(lines[j]);
                            break;
                        }
                    }
                }
                dbgg!(&one_expl);
                ret.expls.push(one_expl);
            }
            // else if re_meaning_only.is_match(line) {
            //     let mut one_expl = Expl::default();
            //     one_expl.meaning = line;
            //     ret.expls.push(one_expl);
            // }
            i += 1;
        }

        dbgg!(&ret);

        Ok(LevelExpained::DefaultKind(ret, s))
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

    ///
    /// Note that all `pretty_print` should not handle their own LF, but leave
    /// the job to their parents.
    pub fn pretty_print(&self, nocolor: bool, asciiful: bool, wrap_width: usize) -> Result<String> {
        let mut output = String::default();

        match self {
            LevelExpained::DefaultKind(expl, s) => {
                let pres = expl.pretty_print(nocolor, asciiful, wrap_width)?;
                if pres.trim().len() == 0 {
                    output += if nocolor { "" } else { "\x1b[31;1m" };
                    output += "\n<UNSUPPORTED>\n";
                    output += if nocolor { "" } else { "\x1b[0m" };

                    output += if nocolor { "" } else { "\x1b[33m" };
                    output += &comm::make_printable(s);
                    output += if nocolor { "" } else { "\x1b[0m" };

                    output += if nocolor { "" } else { "\x1b[31;1m" };
                    output += "\n<UNSUPPORTED>\n";
                    output += if nocolor { "" } else { "\x1b[0m" };
                } else {
                    output += &pres;
                }
            }
            LevelExpained::RealExampKind(expl) => {
                output += &expl.pretty_print(nocolor, asciiful, wrap_width)?;
            }
        }

        Ok(output)
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct CamdResult<'w, 's>(&'w str, Vec<LevelExpained<'s>>);

impl<'w, 's> CamdResult<'w, 's> {
    pub fn from_str(word: &'w str, s: &'s str) -> Result<Self> {
        let bytes = s.as_bytes();
        let mut lv_expl_list = Vec::<&str>::new();

        dbgg!((word, s));

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
                ret.1.push(obj);
            }
        }
        ret.0 = word;

        Ok(ret)
    }

    pub fn pretty_print(&self, nocolor: bool, asciiful: bool, wrap_width: usize) -> Result<String> {
        let wrap_width: usize = if wrap_width > 17 {
            wrap_width.into()
        } else {
            80
        };

        let header_part = if asciiful {
            format!(" RESULT |")
        } else {
            format!(" RESULT │")
        };

        let header_part_colorful = if asciiful {
            format!(" \x1b[36;1mRESULT\x1b[0m |")
        } else {
            format!(" \x1b[36;1mRESULT\x1b[0m │")
        };

        // dbgg!(&header_part);

        let cols_header_part = UnicodeWidthStr::width(header_part.as_str());

        let mut output = String::from("");

        // 0 for top, 1 for bottom
        let line_comp = if asciiful { "-" } else { "─" };
        let line_tail_comp = if asciiful { ("-", "-") } else { ("╮", "┴") };

        let cols_line_comp = UnicodeWidthStr::width(line_comp);
        let rtimes_line_comp = (cols_header_part / cols_line_comp) - 1;

        // top line
        let top_line = comm::replicate(line_comp, rtimes_line_comp);
        output += &top_line;
        output += line_tail_comp.0;
        output += "\n";

        output += if nocolor {
            &header_part
        } else {
            &header_part_colorful
        };
        output += "\n";

        // bottom line
        let bottom_line = comm::replicate(line_comp, rtimes_line_comp);
        output += &bottom_line;
        output += line_tail_comp.1;
        output += &comm::replicate(
            line_comp,
            (wrap_width
                - cols_line_comp * rtimes_line_comp
                - UnicodeWidthStr::width(line_tail_comp.1))
                / cols_line_comp,
        ); // bottom needs extra line_comp to reach 80
        output += "\n";

        // the word user quest
        output += "? ";
        output += self.0;
        output += " ?";
        output += "\n";

        for lv_expl in &self.1 {
            let pres = &lv_expl.pretty_print(nocolor, asciiful, wrap_width)?;
            if pres.len() == 0 {
                output += "<UNSUPPORTED>\n";
            } else {
                output += pres;
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tst {
    use super::*;

    #[test]
    fn _1() {
        let explained = "\"______ \\nhello\\nexclamation, noun\\nUS  /heˈloʊ/ UK  /heˈləʊ/\\n(also mainly UK hallo); (hullo)\\nAdd to word list \\nA1\\nused when meeting or greeting someone:\\nHello, Paul. I haven't seen you for ages.\\nI know her vaguely - we've exchanged hellos a few times.\\nI just thought I'd call by and say hello.\\nAnd a big hello (= welcome) to all the parents who've come to see the show.\\n \\nA1\\nsomething that is said at the beginning of a phone conversation:\\n\\\"Hello, I'd like some information about flights to the U.S., please.\\\"\\n \\nsomething that is said to attract someone's attention:\\nThe front door was open so she walked inside and called out, \\\"Hello! Is there anybody in?\\\"\\n \\ninformal\\nsaid to someone who has just said or done something stupid, especially something that shows they are not noticing what is happening:\\nShe asked me if I'd just arrived and I was like \\\"Hello, I've been here for an hour.\\\"\\n \\nold-fashioned\\nan expression of surprise:\\nHello, this is very strange - I know that man.\\n Fewer examples\\nCathy poked her head round the door to say hello.\\nWhen he said hello, I felt my face turn bright red.\\nHello - could I speak to Ann, please?\\nAfter we'd said our hellos, it all went quiet and nobody knew what to do.\\nOh, hello - what are you doing in here?\\n SMART Vocabulary: related words and phrases\\nGrammar\\nGreetings and farewells: hello, goodbye, Happy New Year\\nWhen we see someone we know, we usually exchange greetings: …\\nSaying hello\\nWhen we see someone we know, we usually exchange greetings: …\\nSaying goodbye\\nWhen we leave people, we usually say something as we leave: …\\n(Definition of hello from the Cambridge Advanced Learner's Dictionary & Thesaurus © Cambridge University Press)______hello | INTERMEDIATE ENGLISH\\nhello\\nexclamation, noun [ C ]\\nUS  /heˈloʊ, hə-/\\nplural hellos\\nAdd to word list \\nused when meeting or greeting someone:\\n\\\"Hello, Paul,\\\" she said, \\\"I haven’t seen you for months.\\\"\\nI know her vaguely – we’ve exchanged hellos a few times.\\nCome and say hello to my friends (= meet them).\\n \\nHello is also said at the beginning of a telephone conversation.\\n \\nHello is also used to attract someone’s attention:\\nShe walked into the shop and called out, \\\"Hello! Is anybody here?\\\"\\n(Definition of hello from the Cambridge Academic Content Dictionary © Cambridge University Press)______EXAMPLES of hello\\nhello\\nShe said that the highlight of her day was when she went up to say hello to one of the families.\\nFrom Huffington Post\\nWe learned to hear sorrow in one \\\"hello,\\\" and how to sit with each other without words.\\nFrom Huffington Post\\nThey say they hear him saying words like \\\"hello,\\\" even if others are skeptical, and say he responds to their attention.\\nFrom ABC News\\nUsually, they get a response, and the second baseman will find his friends and say hello.\\nFrom ESPN\\nThough he played a criminal on television, they say he was one of the nicest men and always waved hello.\\nFrom CNN\\nAlmost everybody stops by to say hello and chat.\\nFrom Chicago Tribune\\nHello, this is your friendly government authority here.\\nFrom Gizmodo\\nHello, please allow me to introduce myself, sir.\\nFrom CNN\\nWalk up to him or her and do three things: smile, say hello, and listen.\\nFrom Huffington Post\\nWhat are the characteristics of the way you say, \\\"hello,\\\" (or anything else for that matter) that makes you recognizable over the phone?\\nFrom Phys.Org\\nHello didn't become \\\"hi\\\" until the telephone arrived.\\nFrom NPR\\nThese examples are from corpora and from sources on the web. Any opinions in the examples do not represent the opinion of the Cambridge Dictionary editors or of Cambridge University Press or its licensors.______What is the pronunciation of hello?______\u{a0}\"";

        let camd_res = CamdResult::from_str("hello", &explained).expect("bug");

        let mut expected_camd_res = CamdResult("hello", vec![]);

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
                // " Fewer examples",
                // "Cathy poked her head round the door to say hello.",
                // "When he said hello, I felt my face turn bright red.",
                // "Hello - could I speak to Ann, please?",
                // "After we'd said our hellos, it all went quiet and nobody knew what to do.",
                // "Oh, hello - what are you doing in here?",
            ],
            nv_cate: None,
        });

        expected_camd_res
            .1
            .push(LevelExpained::DefaultKind(expl, " \\nhello\\nexclamation, noun\\nUS  /heˈloʊ/ UK  /heˈləʊ/\\n(also mainly UK hallo); (hullo)\\nAdd to word list \\nA1\\nused when meeting or greeting someone:\\nHello, Paul. I haven't seen you for ages.\\nI know her vaguely - we've exchanged hellos a few times.\\nI just thought I'd call by and say hello.\\nAnd a big hello (= welcome) to all the parents who've come to see the show.\\n \\nA1\\nsomething that is said at the beginning of a phone conversation:\\n\\\"Hello, I'd like some information about flights to the U.S., please.\\\"\\n \\nsomething that is said to attract someone's attention:\\nThe front door was open so she walked inside and called out, \\\"Hello! Is there anybody in?\\\"\\n \\ninformal\\nsaid to someone who has just said or done something stupid, especially something that shows they are not noticing what is happening:\\nShe asked me if I'd just arrived and I was like \\\"Hello, I've been here for an hour.\\\"\\n \\nold-fashioned\\nan expression of surprise:\\nHello, this is very strange - I know that man.\\n Fewer examples\\nCathy poked her head round the door to say hello.\\nWhen he said hello, I felt my face turn bright red.\\nHello - could I speak to Ann, please?\\nAfter we'd said our hellos, it all went quiet and nobody knew what to do.\\nOh, hello - what are you doing in here?\\n SMART Vocabulary: related words and phrases\\nGrammar\\nGreetings and farewells: hello, goodbye, Happy New Year\\nWhen we see someone we know, we usually exchange greetings: …\\nSaying hello\\nWhen we see someone we know, we usually exchange greetings: …\\nSaying goodbye\\nWhen we leave people, we usually say something as we leave: …\\n(Definition of hello from the Cambridge Advanced Learner's Dictionary & Thesaurus © Cambridge University Press)"));

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
            nv_cate: Some("plural hellos"),
        });
        // parsing meaning-only is not supported currently
        // expl.expls.push(Expl {
        //     meaning: "Hello is also said at the beginning of a telephone conversation.",
        //     usages: vec![],
        //     nv_cate: None,
        // });
        expl.expls.push(Expl {
            meaning: "Hello is also used to attract someone’s attention:",
            usages: vec![
                "She walked into the shop and called out, \\\"Hello! Is anybody here?\\\"",
            ],
            nv_cate: None,
        });

        expected_camd_res
            .1
            .push(LevelExpained::DefaultKind(expl, "hello | INTERMEDIATE ENGLISH\\nhello\\nexclamation, noun [ C ]\\nUS  /heˈloʊ, hə-/\\nplural hellos\\nAdd to word list \\nused when meeting or greeting someone:\\n\\\"Hello, Paul,\\\" she said, \\\"I haven’t seen you for months.\\\"\\nI know her vaguely – we’ve exchanged hellos a few times.\\nCome and say hello to my friends (= meet them).\\n \\nHello is also said at the beginning of a telephone conversation.\\n \\nHello is also used to attract someone’s attention:\\nShe walked into the shop and called out, \\\"Hello! Is anybody here?\\\"\\n(Definition of hello from the Cambridge Academic Content Dictionary © Cambridge University Press)"));

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

        expected_camd_res.0 = "hello";
        expected_camd_res.1.push(LevelExpained::RealExampKind(expl));

        assert_eq!(camd_res, expected_camd_res);

        dbg!(&camd_res, &expected_camd_res);
    }

    #[test]
    fn _2() {
        let explained = "\"______ \\nworld\\nnoun\\nUS  /wɝːld/ UK  /wɜːld/\\nworld noun (THE EARTH)\\nAdd to word list \\nA1 [ S ]\\nthe earth and all the people, places, and things on it:\\nDifferent parts of the world have very different climatic conditions.\\nWhich bridge has the longest span in the world?\\nNews of the disaster shocked the (whole/entire) world.\\nWe live in a changing world and people must learn to adapt.\\nShe's a world authority on fetal development.\\na world record/championship\\n Fewer examples\\nPeople from different cultures have different conceptions of the world.\\nThe richer countries of the world should take concerted action to help the poorer countries.\\nI'm flirting with the idea of taking a year off and traveling round the world.\\nHe's one of the highest-earning professional golfers in the world.\\nThe museum's collection includes works of art from all around the world.\\n SMART Vocabulary: related words and phrases\\nworld noun (GROUP/AREA)\\n \\nB1 [ C usually singular ]\\na group of things such as countries or animals, or an area of human activity or understanding:\\nthe Muslim world\\nthe modern/industrialized world\\nthe animal world\\nstars from the rock music world\\nUnexpected things can happen in the world of subatomic particles.\\n More examples\\n SMART Vocabulary: related words and phrases\\nworld noun (PLANET)\\n \\n[ C ]\\na planet or other part of the universe, especially one where life might or does exist:\\nThere was a man on the news last night who believes we've been visited by beings from other worlds.\\n SMART Vocabulary: related words and phrases\\nIdioms\\nat one with the world\\nbe worlds apart\\ndo someone a world of good\\nfor all the world\\ngo/come down in the world\\ngo/come up in the world\\nhave the world at your feet\\nin a world of your own\\nmake a world of difference\\nmake the world go around/round\\n More idioms\\n(Definition of world from the Cambridge Advanced Learner's Dictionary & Thesaurus © Cambridge University Press)______world | INTERMEDIATE ENGLISH\\nworld\\nnoun\\nUS  /wɜrld/\\nworld noun (THE EARTH)\\nAdd to word list \\n[ U ]\\nthe planet on which human life has developed, esp. including all people and their ways of life:\\nPeople from all over the world will be attending the conference.\\nThe rapid growth of computers has changed the world.\\n \\n[ U ]\\nThe world can also mean the whole physical universe:\\nThe world contains many solar systems, not just ours.\\nworld noun (WHOLE AREA)\\n \\n[ C ]\\nall of a particular group or type of thing, such as countries or animals, or a whole area of human activity or understanding:\\nthe animal/plant world\\nthe business world\\nthe world of entertainment\\nIn the world of politics, the president’s voice is still the most powerful in the nation.\\nworld noun (LARGE DEGREE)\\n \\n[ U ]\\na large degree; a lot:\\nThere’s a world of difference between the two hotels.\\nIdioms\\nin a world of your own\\nin the world\\nman of the world\\n(Definition of world from the Cambridge Academic Content Dictionary © Cambridge University Press)______world | BUSINESS ENGLISH\\nworld\\nnoun [ C, usually singular ]\\nUK  /wɜːld/ US \\nAdd to word list \\na particular area of activity:\\nOur world of work is changing rapidly.\\nthe world of advertising/the internet\\nthe business/corporate world\\n(Definition of world from the Cambridge Business English Dictionary © Cambridge University Press)______EXAMPLES of world\\nworld\\nWhat happens in my life, in my world, doesn't have anything to do with you.\\nFrom NPR\\nMore than 300,000 podcasts exist in the world as of the close of 2015.\\nFrom The Atlantic\\nThis is the world we are headed toward.\\nFrom TIME\\nThe book goes out into the world, and who knows?\\nFrom The Atlantic\\nThis will make the world a better place.\\nFrom CNN\\nWe're all coming together towards making the world a better place.\\nFrom Voice of America\\nIt's what people in 3rd and 4th world countries do.\\nFrom CNN\\nAnd as the world for birds goes, our world can't be far behind.\\nFrom National Geographic\\nBudgets fool us into believing that they will not only tame us, but the world around us as well.\\nFrom New York Daily News\\nBut what in the world was that album all about?\\nFrom TIME\\nIt's a fun world to be a part of.\\nFrom VentureBeat\\nIt's bringing those worlds together that most interests me.\\nFrom NJ.com\\nIs the world a better place for having you and your work a part of it?\\nFrom Fast Company\\nStart paying attention to the physical world around you.\\nFrom Huffington Post\\nIt housed one of the world's important collections of arms and armor.\\nFrom CNBC\\nThese examples are from corpora and from sources on the web. Any opinions in the examples do not represent the opinion of the Cambridge Dictionary editors or of Cambridge University Press or its licensors.______COLLOCATIONS with world\\nworld\\n\\nThese are words often used in combination with world.\\n\\nClick on a collocation to see more examples of it.\\n\\nalien world\\nFrom everyday objects they built an alien world.\\nFrom the Cambridge English Corpus\\n\u{a0}\\nancient world\\nThe ancient world made a welcome reappearance in three theses.\\nFrom the Cambridge English Corpus\\n\u{a0}\\ncapitalist world\\nAcross the capitalist world the problem of cost containment has dominated health care since the mid-1970s.\\nFrom the Cambridge English Corpus\\n\u{a0}\\nThese examples are from corpora and from sources on the web. Any opinions in the examples do not represent the opinion of the Cambridge Dictionary editors or of Cambridge University Press or its licensors.\\nSee all collocations with world______What is the pronunciation of world?______\u{a0}\"";

        let camd_res = CamdResult::from_str("world", &explained).expect("bug");

        let mut expected_camd_res = CamdResult("world", vec![]);

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
                // " Fewer examples",
                // "People from different cultures have different conceptions of the world.",
                // "The richer countries of the world should take concerted action to help the poorer countries.",
                // "I'm flirting with the idea of taking a year off and traveling round the world.",
                // "He's one of the highest-earning professional golfers in the world.",
                // "The museum's collection includes works of art from all around the world.",
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

        expected_camd_res
            .1
            .push(LevelExpained::DefaultKind(expl, " \\nworld\\nnoun\\nUS  /wɝːld/ UK  /wɜːld/\\nworld noun (THE EARTH)\\nAdd to word list \\nA1 [ S ]\\nthe earth and all the people, places, and things on it:\\nDifferent parts of the world have very different climatic conditions.\\nWhich bridge has the longest span in the world?\\nNews of the disaster shocked the (whole/entire) world.\\nWe live in a changing world and people must learn to adapt.\\nShe's a world authority on fetal development.\\na world record/championship\\n Fewer examples\\nPeople from different cultures have different conceptions of the world.\\nThe richer countries of the world should take concerted action to help the poorer countries.\\nI'm flirting with the idea of taking a year off and traveling round the world.\\nHe's one of the highest-earning professional golfers in the world.\\nThe museum's collection includes works of art from all around the world.\\n SMART Vocabulary: related words and phrases\\nworld noun (GROUP/AREA)\\n \\nB1 [ C usually singular ]\\na group of things such as countries or animals, or an area of human activity or understanding:\\nthe Muslim world\\nthe modern/industrialized world\\nthe animal world\\nstars from the rock music world\\nUnexpected things can happen in the world of subatomic particles.\\n More examples\\n SMART Vocabulary: related words and phrases\\nworld noun (PLANET)\\n \\n[ C ]\\na planet or other part of the universe, especially one where life might or does exist:\\nThere was a man on the news last night who believes we've been visited by beings from other worlds.\\n SMART Vocabulary: related words and phrases\\nIdioms\\nat one with the world\\nbe worlds apart\\ndo someone a world of good\\nfor all the world\\ngo/come down in the world\\ngo/come up in the world\\nhave the world at your feet\\nin a world of your own\\nmake a world of difference\\nmake the world go around/round\\n More idioms\\n(Definition of world from the Cambridge Advanced Learner's Dictionary & Thesaurus © Cambridge University Press)"));

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

        expected_camd_res
            .1
            .push(LevelExpained::DefaultKind(expl, "world | INTERMEDIATE ENGLISH\\nworld\\nnoun\\nUS  /wɜrld/\\nworld noun (THE EARTH)\\nAdd to word list \\n[ U ]\\nthe planet on which human life has developed, esp. including all people and their ways of life:\\nPeople from all over the world will be attending the conference.\\nThe rapid growth of computers has changed the world.\\n \\n[ U ]\\nThe world can also mean the whole physical universe:\\nThe world contains many solar systems, not just ours.\\nworld noun (WHOLE AREA)\\n \\n[ C ]\\nall of a particular group or type of thing, such as countries or animals, or a whole area of human activity or understanding:\\nthe animal/plant world\\nthe business world\\nthe world of entertainment\\nIn the world of politics, the president’s voice is still the most powerful in the nation.\\nworld noun (LARGE DEGREE)\\n \\n[ U ]\\na large degree; a lot:\\nThere’s a world of difference between the two hotels.\\nIdioms\\nin a world of your own\\nin the world\\nman of the world\\n(Definition of world from the Cambridge Academic Content Dictionary © Cambridge University Press)"));

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

        expected_camd_res
            .1
            .push(LevelExpained::DefaultKind(expl, "world | BUSINESS ENGLISH\\nworld\\nnoun [ C, usually singular ]\\nUK  /wɜːld/ US \\nAdd to word list \\na particular area of activity:\\nOur world of work is changing rapidly.\\nthe world of advertising/the internet\\nthe business/corporate world\\n(Definition of world from the Cambridge Business English Dictionary © Cambridge University Press)"));

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

        expected_camd_res.0 = "world";
        expected_camd_res.1.push(LevelExpained::RealExampKind(expl));

        assert_eq!(camd_res, expected_camd_res);

        dbg!(&camd_res, &expected_camd_res);
    }

    #[test]
    fn _3() {
        let explained = "\"______ \\ndetail\\nnoun\\nUS  /dɪˈteɪl/ US  /ˈdiː.teɪl/ UK  /ˈdiː.teɪl/\\ndetail noun (INFORMATION)\\nAdd to word list \\nB1 [ C ]\\na single piece of information or fact about something:\\nShe insisted on telling me every single detail of what they did to her in the hospital.\\nWe don't know the full/precise details of the story yet.\\nShe refused to disclose/divulge any details about/of the plan.\\n\u{a0}details [ plural ]\\n \\nA2\\ninformation about someone or something:\\nA police officer took down the details of what happened.\\nSee more\\n \\n[ U ]\\nthe small features of something that you only notice when you look carefully:\\nI was just admiring the detail in the dollhouse - even the cans of food have labels on them.\\nIt's his eye for (= ability to notice) detail that distinguishes him as a painter.\\n\u{a0}in detail\\n \\nB1\\nincluding or considering all the information about something or every part of something:\\nWe haven't discussed the matter in detail yet.\\nSee more\\n\u{a0}go into detail\\n \\nB2\\nto tell or include all the facts about something:\\nI won't go into detail over the phone, but I've been having a few health problems recently.\\nSee more\\n \\n[ C ]\\na part of something that does not seem important:\\nTony says, he's going to get the car, and finding the money to pay for it is just a minor detail.\\n Fewer examples\\nThe model of the village is accurate down to the last detail.\\nHe forgot to tell you one important detail - he's married.\\nIt's only a detail, but could you just add the office phone number at the top of the page?\\nHer paintings are almost photographic in their detail and accuracy.\\nThere is one small detail you've gotten wrong in your report.\\n SMART Vocabulary: related words and phrases\\ndetail noun (GROUP)\\n \\n[ C, + sing/pl verb ]\\na group of people who have been given a particular task\\n SMART Vocabulary: related words and phrases\\n \\ndetail\\nverb\\nUS  /dɪˈteɪl/ US  /ˈdiː.teɪl/ UK  /ˈdiː.teɪl/\\ndetail verb (GIVE INFORMATION)\\n \\n[ T ] US  /dɪˈteɪl/ US  /ˈdiː.teɪl/ UK  /ˈdiː.teɪl/\\nto describe something completely, giving all the facts:\\n[ + question word ] Can you produce a report detailing what we've spent on the project so far?\\n SMART Vocabulary: related words and phrases\\ndetail verb (ORDER)\\n \\n[ T + to infinitive, often passive ] US  /dɪˈteɪl/ US  /ˈdiː.teɪl/ UK  /ˈdiː.teɪl/\\nto order someone, often a small group of soldiers or workers, to perform a particular task:\\nFour soldiers were detailed to check the road for troops.\\n SMART Vocabulary: related words and phrases\\ndetail verb (CLEAN CAR)\\n \\n[ T ] US US/ˈdiː.teɪl/ UK  /ˈdiː.teɪl/\\nto clean the inside and outside of a vehicle very carefully:\\nYou can skip the car wash; Rogers has all the equipment to wash and detail your car in your own driveway.\\na car detailing company\\n SMART Vocabulary: related words and phrases\\n(Definition of detail from the Cambridge Advanced Learner's Dictionary & Thesaurus © Cambridge University Press)______detail | INTERMEDIATE ENGLISH\\ndetail\\nnoun\\nUS  /dɪˈteɪl, ˈdi·teɪl/\\ndetail noun (INFORMATION)\\nAdd to word list \\n[ C/U ]\\na particular fact or item of information, often noticed only after giving something your close attention, or such facts or items considered as a group:\\n[ C ] We have a report of a serious accident on Route 23, but so far no details.\\n[ U ] She showed a businesslike attention to detail.\\n[ U ] I can’t go into much detail, but I’ve been having some health problems recently.\\nWe know roughly what he wants to do, but we haven’t discussed his plans in detail (= considering all the particular facts).\\ndetail noun (GROUP)\\n \\n[ C ]\\na small group, esp. of soldiers or police, ordered to perform a particular duty:\\nA detail of five police officers accompanied the diplomat to his hotel.\\ndetailed\\nadjective US  /dɪˈteɪld, ˈdi·teɪld/\\na detailed account/description\\n \\ndetail\\nverb [ T ]\\nUS  /dɪˈteɪl, ˈdi·teɪl/\\ndetail verb [T] (GIVE INFORMATION)\\n \\nto give exact and complete information about something:\\nThe committee members issued a brief statement detailing their plans.\\n(Definition of detail from the Cambridge Academic Content Dictionary © Cambridge University Press)______EXAMPLES of detail\\ndetail\\nHe sent a letter detailing the problems to the manufacturer.\\nFrom Voice of America\\nThey have not released any details about a motive.\\nFrom ABC News\\nThis will detail how to take actions like blocking users, for example.\\nFrom TechCrunch\\nShe had signs of trauma on her body; but, police are not releasing details.\\nFrom CBS Local\\nOne reason may be the terribly unsexy details of the employee-ownership structure.\\nFrom The Atlantic\\nWe are seeking more details from the district and will post them here when available.\\nFrom cleveland.com\\nSome details of the episode, though, remain murky.\\nFrom Washington Post\\nWe'll note where the two disagree on the details.\\nFrom VentureBeat\\nTimes staffers will be there to bring you the details.\\nFrom Los Angeles Times\\nI was very impressed with the level of detail he had maintained during the restoration.\\nFrom USA TODAY\\nA spokesman said there were \\\"no immediate details\\\" on the nature of the threat, saying the call came in around 12:15 p.m.\\nFrom Washington Post\\nHow do other people and entire communities come to care about species about whose biological details they might not know?\\nFrom Phys.Org\\nShe is gifted with language and is able to layer difficult details in such a way that the result is smooth as water.\\nFrom NPR\\nTheir older parents want to speak about the logistics of death in detail.\\nFrom Huffington Post\\nThese examples are from corpora and from sources on the web. Any opinions in the examples do not represent the opinion of the Cambridge Dictionary editors or of Cambridge University Press or its licensors.______COLLOCATIONS with detail\\ndetail\\n\\nThese are words often used in combination with detail.\\n\\nClick on a collocation to see more examples of it.\\n\\naccurate detail\\nThough the story is fictional, recent scholarship has uncovered a greater measure of historically accurate detail in its setting than had previously been realized.\\nFrom the Cambridge English Corpus\\n\u{a0}\\nadditional detail\\nExtended response that contains additional detail that is irrelevant, repetitive or bizarre.\\nFrom the Cambridge English Corpus\\n\u{a0}\\nadministrative detail\\nSuch policies cannot just be legislated; they must be worked out in administrative detail.\\nFrom the Cambridge English Corpus\\n\u{a0}\\nThese examples are from corpora and from sources on the web. Any opinions in the examples do not represent the opinion of the Cambridge Dictionary editors or of Cambridge University Press or its licensors.\\nSee all collocations with detail______What is the pronunciation of detail?______\u{a0}\"";

        let camd_res = CamdResult::from_str("detail", &explained).expect("bug");

        let mut expected_camd_res = CamdResult("detail", vec![]);

        // primary one
        let mut expl = DefaultExpl::default();
        expl.is_interme = false;
        expl.is_busi = false;
        expl.pronun = "US  /dɪˈteɪl/ US  /ˈdiː.teɪl/ UK  /ˈdiː.teɪl/";
        expl.expls.push(Expl {
            meaning: "a single piece of information or fact about something:",
            usages: vec![
                "She insisted on telling me every single detail of what they did to her in the hospital.",
                "We don't know the full/precise details of the story yet.",
                "She refused to disclose/divulge any details about/of the plan.",
            ],
            nv_cate: Some("detail noun (INFORMATION)"),
        });
        expl.expls.push(Expl {
            meaning: "information about someone or something:",
            usages: vec!["A police officer took down the details of what happened."],
            nv_cate: Some("\u{a0}details [ plural ]"),
        });
        expl.expls.push(Expl {
            meaning: "the small features of something that you only notice when you look carefully:",
            usages: vec![
                "I was just admiring the detail in the dollhouse - even the cans of food have labels on them.",
		"It's his eye for (= ability to notice) detail that distinguishes him as a painter.",
		// "\u{a0}in detail"
            ],
            nv_cate: None
        });
        expl.expls.push(Expl {
            meaning: "including or considering all the information about something or every part of something:",
            usages: vec![
                "We haven't discussed the matter in detail yet.",
            ],
            nv_cate: None
        });
        expl.expls.push(Expl {
            meaning: "to tell or include all the facts about something:",
            usages: vec!["I won't go into detail over the phone, but I've been having a few health problems recently."],
            nv_cate: None
        });
        expl.expls.push(Expl {
            meaning: "a part of something that does not seem important:",
            usages: vec![
		"Tony says, he's going to get the car, and finding the money to pay for it is just a minor detail.",
		// " Fewer examples",
		// "The model of the village is accurate down to the last detail.",
		// "He forgot to tell you one important detail - he's married.",
		// "It's only a detail, but could you just add the office phone number at the top of the page?",
		// "Her paintings are almost photographic in their detail and accuracy.",
		// "There is one small detail you've gotten wrong in your report."
	    ],
            nv_cate: None,
        });
        // no trailing punctuation
        // expl.expls.push(Expl {
        //     meaning: "a group of people who have been given a particular task",
        //     usages: vec![],
        //     nv_cate: Some("detail noun (GROUP)"),
        // });
        expl.expls.push(Expl {
            meaning: "to describe something completely, giving all the facts:",
            usages: vec!["[ + question word ] Can you produce a report detailing what we've spent on the project so far?"],
            nv_cate: Some("detail verb (GIVE INFORMATION)"),
        });
        expl.expls.push(Expl {
            meaning: "to order someone, often a small group of soldiers or workers, to perform a particular task:",
            usages: vec!["Four soldiers were detailed to check the road for troops."],
            nv_cate: Some("detail verb (ORDER)"),
        });
        expl.expls.push(Expl {
            meaning: "to clean the inside and outside of a vehicle very carefully:",
            usages: vec!["You can skip the car wash; Rogers has all the equipment to wash and detail your car in your own driveway.","a car detailing company"],
            nv_cate: Some("detail verb (CLEAN CAR)"),
        });

        expected_camd_res
            .1
            .push(LevelExpained::DefaultKind(expl, " \\ndetail\\nnoun\\nUS  /dɪˈteɪl/ US  /ˈdiː.teɪl/ UK  /ˈdiː.teɪl/\\ndetail noun (INFORMATION)\\nAdd to word list \\nB1 [ C ]\\na single piece of information or fact about something:\\nShe insisted on telling me every single detail of what they did to her in the hospital.\\nWe don't know the full/precise details of the story yet.\\nShe refused to disclose/divulge any details about/of the plan.\\n\u{a0}details [ plural ]\\n \\nA2\\ninformation about someone or something:\\nA police officer took down the details of what happened.\\nSee more\\n \\n[ U ]\\nthe small features of something that you only notice when you look carefully:\\nI was just admiring the detail in the dollhouse - even the cans of food have labels on them.\\nIt's his eye for (= ability to notice) detail that distinguishes him as a painter.\\n\u{a0}in detail\\n \\nB1\\nincluding or considering all the information about something or every part of something:\\nWe haven't discussed the matter in detail yet.\\nSee more\\n\u{a0}go into detail\\n \\nB2\\nto tell or include all the facts about something:\\nI won't go into detail over the phone, but I've been having a few health problems recently.\\nSee more\\n \\n[ C ]\\na part of something that does not seem important:\\nTony says, he's going to get the car, and finding the money to pay for it is just a minor detail.\\n Fewer examples\\nThe model of the village is accurate down to the last detail.\\nHe forgot to tell you one important detail - he's married.\\nIt's only a detail, but could you just add the office phone number at the top of the page?\\nHer paintings are almost photographic in their detail and accuracy.\\nThere is one small detail you've gotten wrong in your report.\\n SMART Vocabulary: related words and phrases\\ndetail noun (GROUP)\\n \\n[ C, + sing/pl verb ]\\na group of people who have been given a particular task\\n SMART Vocabulary: related words and phrases\\n \\ndetail\\nverb\\nUS  /dɪˈteɪl/ US  /ˈdiː.teɪl/ UK  /ˈdiː.teɪl/\\ndetail verb (GIVE INFORMATION)\\n \\n[ T ] US  /dɪˈteɪl/ US  /ˈdiː.teɪl/ UK  /ˈdiː.teɪl/\\nto describe something completely, giving all the facts:\\n[ + question word ] Can you produce a report detailing what we've spent on the project so far?\\n SMART Vocabulary: related words and phrases\\ndetail verb (ORDER)\\n \\n[ T + to infinitive, often passive ] US  /dɪˈteɪl/ US  /ˈdiː.teɪl/ UK  /ˈdiː.teɪl/\\nto order someone, often a small group of soldiers or workers, to perform a particular task:\\nFour soldiers were detailed to check the road for troops.\\n SMART Vocabulary: related words and phrases\\ndetail verb (CLEAN CAR)\\n \\n[ T ] US US/ˈdiː.teɪl/ UK  /ˈdiː.teɪl/\\nto clean the inside and outside of a vehicle very carefully:\\nYou can skip the car wash; Rogers has all the equipment to wash and detail your car in your own driveway.\\na car detailing company\\n SMART Vocabulary: related words and phrases\\n(Definition of detail from the Cambridge Advanced Learner's Dictionary & Thesaurus © Cambridge University Press)"));

        // intermediate one
        let mut expl = DefaultExpl::default();
        expl.is_interme = true;
        expl.is_busi = false;
        expl.pronun = "US  /dɪˈteɪl, ˈdi·teɪl/";
        expl.expls.push(Expl {
            meaning: "a particular fact or item of information, often noticed only after giving something your close attention, or such facts or items considered as a group:",
            usages: vec![
                "[ C ] We have a report of a serious accident on Route 23, but so far no details.",
                "[ U ] She showed a businesslike attention to detail.","[ U ] I can’t go into much detail, but I’ve been having some health problems recently.",
		"We know roughly what he wants to do, but we haven’t discussed his plans in detail (= considering all the particular facts)."
            ],
            nv_cate: Some("detail noun (INFORMATION)"),
        });
        expl.expls.push(Expl {
            meaning:
                "a small group, esp. of soldiers or police, ordered to perform a particular duty:",
            usages: vec![
                "A detail of five police officers accompanied the diplomat to his hotel.",
                "detailed",
            ],
            nv_cate: Some("detail noun (GROUP)"),
        });
        expl.expls.push(Expl {
            meaning: "to give exact and complete information about something:",
            usages: vec!["The committee members issued a brief statement detailing their plans."],
            nv_cate: Some("detail verb [T] (GIVE INFORMATION)"),
        });

        expected_camd_res
            .1
            .push(LevelExpained::DefaultKind(expl, "detail | INTERMEDIATE ENGLISH\\ndetail\\nnoun\\nUS  /dɪˈteɪl, ˈdi·teɪl/\\ndetail noun (INFORMATION)\\nAdd to word list \\n[ C/U ]\\na particular fact or item of information, often noticed only after giving something your close attention, or such facts or items considered as a group:\\n[ C ] We have a report of a serious accident on Route 23, but so far no details.\\n[ U ] She showed a businesslike attention to detail.\\n[ U ] I can’t go into much detail, but I’ve been having some health problems recently.\\nWe know roughly what he wants to do, but we haven’t discussed his plans in detail (= considering all the particular facts).\\ndetail noun (GROUP)\\n \\n[ C ]\\na small group, esp. of soldiers or police, ordered to perform a particular duty:\\nA detail of five police officers accompanied the diplomat to his hotel.\\ndetailed\\nadjective US  /dɪˈteɪld, ˈdi·teɪld/\\na detailed account/description\\n \\ndetail\\nverb [ T ]\\nUS  /dɪˈteɪl, ˈdi·teɪl/\\ndetail verb [T] (GIVE INFORMATION)\\n \\nto give exact and complete information about something:\\nThe committee members issued a brief statement detailing their plans.\\n(Definition of detail from the Cambridge Academic Content Dictionary © Cambridge University Press)"));

        // examples
        let mut expl = RealExamp::default();
        expl.0.push(Examp {
            usage: "He sent a letter detailing the problems to the manufacturer.",
            from: "From Voice of America",
        });
        expl.0.push(Examp {
            usage: "They have not released any details about a motive.",
            from: "From ABC News",
        });
        expl.0.push(Examp {
            usage: "This will detail how to take actions like blocking users, for example.",
            from: "From TechCrunch",
        });
        expl.0.push(Examp {
            usage: "She had signs of trauma on her body; but, police are not releasing details.",
            from: "From CBS Local",
        });
        expl.0.push(Examp {
            usage: "One reason may be the terribly unsexy details of the employee-ownership structure.",
            from: "From The Atlantic",
        });
        expl.0.push(Examp {
            usage: "We are seeking more details from the district and will post them here when available.",
            from: "From cleveland.com",
        });
        expl.0.push(Examp {
            usage: "Some details of the episode, though, remain murky.",
            from: "From Washington Post",
        });
        expl.0.push(Examp {
            usage: "We'll note where the two disagree on the details.",
            from: "From VentureBeat",
        });
        expl.0.push(Examp {
            usage: "Times staffers will be there to bring you the details.",
            from: "From Los Angeles Times",
        });
        expl.0.push(Examp {
            usage: "I was very impressed with the level of detail he had maintained during the restoration.",
            from: "From USA TODAY",
        });
        expl.0.push(Examp {
            usage: "A spokesman said there were \\\"no immediate details\\\" on the nature of the threat, saying the call came in around 12:15 p.m.",
            from: "From Washington Post",
        });
        expl.0.push(Examp {
            usage: "How do other people and entire communities come to care about species about whose biological details they might not know?",
            from: "From Phys.Org",
        });
        expl.0.push(Examp {
            usage: "She is gifted with language and is able to layer difficult details in such a way that the result is smooth as water.",
            from: "From NPR",
        });
        expl.0.push(Examp {
            usage: "Their older parents want to speak about the logistics of death in detail.",
            from: "From Huffington Post",
        });

        expected_camd_res.0 = "detail";
        expected_camd_res.1.push(LevelExpained::RealExampKind(expl));

        assert_eq!(camd_res, expected_camd_res);

        dbg!(&camd_res, &expected_camd_res);
    }

    #[test]
    fn extract_pronun_1() {
        let s = "US  /wɝːld/ UK  /wɜːld/";
        assert_eq!(
            DefaultExpl::extract_pronun(s),
            (Some("/wɝːld/"), Some("/wɜːld/"), None)
        );
    }

    #[test]
    fn extract_pronun_2() {
        let s = "US  /wɝːld/";
        assert_eq!(
            DefaultExpl::extract_pronun(s),
            (Some("/wɝːld/"), None, None)
        );
    }

    #[test]
    fn extract_pronun_3() {
        let s = "UK  /wɝːld/";
        assert_eq!(
            DefaultExpl::extract_pronun(s),
            (None, Some("/wɝːld/"), None)
        );
    }

    #[test]
    fn extract_pronun_4() {
        let s = "US  /wɝːld/ UK";
        assert_eq!(
            DefaultExpl::extract_pronun(s),
            (Some("/wɝːld/"), None, None)
        );
    }

    #[test]
    fn extract_pronun_5() {
        let s = "UK  /wɝːld/ US";
        assert_eq!(
            DefaultExpl::extract_pronun(s),
            (None, Some("/wɝːld/"), None)
        );
    }
}
