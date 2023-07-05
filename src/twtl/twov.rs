// Copyright (C) 2023 Michael Lee <imichael2e2@proton.me/...@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 or any later
// version <LICENSE-GPL or https://www.gnu.org/licenses/gpl-3.0.txt>.
//
// This file may not be copied, modified, or distributed except in compliance
// with the license.
//

use std::borrow::Cow;

use unicode_width::UnicodeWidthStr;

use crate::comm;
use crate::error::{MafaError, Result};

#[derive(Debug, Default, serde::Serialize)]
struct QuoteTweet<'a> {
    dispname: Cow<'a, str>,
    username: Cow<'a, str>,
    tstamp: Cow<'a, str>,
    ctn: Cow<'a, str>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct TweetOverview<'a> {
    twid: Cow<'a, str>,
    dispname: Cow<'a, str>,
    username: Cow<'a, str>,
    post_tstamp: Cow<'a, str>,
    ctn: Cow<'a, str>,
    reply_to: Option<Cow<'a, str>>,
    retweeter: Option<Cow<'a, str>>,
    quote: Option<QuoteTweet<'a>>,
}

impl<'a> TweetOverview<'a> {
    // lately
    pub fn from_str(orig: &str) -> Result<TweetOverview> {
        Self::from_str_v1(orig)
    }

    fn from_str_v1(orig: &str) -> Result<TweetOverview> {
        // reject TODO: Readers added context they thought
        //              people might want to know
        // reject twitter spaces(may support in the future)
        if orig.contains("Host") && orig.contains("Play recording") {
            return Err(MafaError::TweetNotRecoginized(7));
        }

        let bytes_orig = orig.as_bytes();
        let mut tw_as_lines = Vec::<(&str, usize, usize)>::new();
        // (str, str_begi, str_endi)

        let mut prev_i = 0;
        for curr_i in 0..bytes_orig.len() {
            let curr_byte = bytes_orig[curr_i];
            if curr_byte == 0xa {
                tw_as_lines.push((&orig[prev_i..curr_i], prev_i, curr_i));
                prev_i = curr_i;
            }
        }
        if prev_i < orig.len() {
            tw_as_lines.push((&orig[prev_i..], prev_i, orig.len()));
        }

        dbgg!(&tw_as_lines);

        if tw_as_lines.len() == 0 || tw_as_lines[0].0 != "twtl_v1" {
            return Err(MafaError::TweetNotRecoginized(1));
        }

        if tw_as_lines.len() < 2 {
            dbgg!((2, orig));
            return Err(MafaError::TweetNotRecoginized(2));
        }

        let re_twid = regex::Regex::new("^\n[0-9]{19}$").map_err(|_| MafaError::BugFound(1122))?;

        if !re_twid.is_match(tw_as_lines[1].0) {
            dbgg!((6, orig));
            return Err(MafaError::TweetNotRecoginized(6));
        }
        let li_twid = 1;

        // Find tstamp pos

        let re_tstamp = regex::Regex::new(
        "^\n([0-9]{1,2}[smh])|((Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) [0-9]{1,2}(, [0-9]{4})*)$",
    )
    .expect("buggy");

        let mut li_tstamp = 0;

        for li in 0..tw_as_lines.len() {
            if re_tstamp.is_match(tw_as_lines[li].0) {
                li_tstamp = li;
                break;
            }
        }

        if li_tstamp == 0 || li_tstamp == tw_as_lines.len() - 1 {
            return Err(MafaError::TweetNotRecoginized(3));
        }

        // reply to
        let mut li_rpl = 0;
        dbgg!(tw_as_lines[li_tstamp + 1], tw_as_lines[li_tstamp + 2]);
        if li_tstamp + 1 < tw_as_lines.len()
            && tw_as_lines[li_tstamp + 1].0 == "\nReplying to "
            && li_tstamp + 2 < tw_as_lines.len()
            && tw_as_lines[li_tstamp + 2].0.as_bytes()[0] == b'\n'
        {
            li_rpl = li_tstamp + 2;
        }

        // Find others before tstamp //

        let mut li_usrname = 0;
        let re_usrname =
            regex::Regex::new("^\n@[A-Za-z0-9_]+$").map_err(|_| MafaError::BugFound(1122))?;
        let mut li_disname = 0;
        let re_rt = regex::Regex::new("^\n.* Retweeted$").map_err(|_| MafaError::BugFound(1122))?;
        let mut li_rt = 0;

        for i in 0..li_tstamp {
            let line = tw_as_lines[i].0;
            if re_usrname.is_match(line) {
                li_usrname = i;
                li_disname = i - 1;
            }
            if re_rt.is_match(line) {
                li_rt = i;
            }
        }

        if li_usrname == 0 || li_disname == 0 {
            return Err(MafaError::TweetNotRecoginized(4));
        }

        dbgg!(tw_as_lines[li_tstamp]);
        dbgg!(tw_as_lines[li_usrname]);
        dbgg!(tw_as_lines[li_disname]);

        // Find tweet end //

        let mut li_ctn_lastl = tw_as_lines.len() - 1;

        let re_noisecnt = regex::Regex::new(
        r"^\n(([0-9]{3})|([0-9]{2})|([0-9]{1})|([1-9]{1},[0-9]{3})|(([1-9]{1}|[0-9]{2}|[0-9]{3})(.[0-9]){0,1}[KMB]))$",
    )
    .map_err(|_| MafaError::BugFound(1122))?;

        let re_imagealt =
            regex::Regex::new(r"^\n(ALT|GIF)$").map_err(|_| MafaError::BugFound(1122))?;

        // li_ctn_lastl
        loop {
            let line = tw_as_lines[li_ctn_lastl].0;
            if re_noisecnt.is_match(line) || re_imagealt.is_match(line) {
                // dbgg!(("noise count", line));
                li_ctn_lastl -= 1;
            } else {
                break;
            }
        }
        if li_ctn_lastl <= li_tstamp {
            return Err(MafaError::TweetNotRecoginized(5));
        }

        let re_quote_header =
            regex::Regex::new(r"^\nQuote Tweet$").map_err(|_| MafaError::BugFound(1122))?;

        // selftw_lastl
        let mut li_selftw_lastl = 0; // lastl is inclusive
        let mut li_quotetw_firstl = 0;
        let mut li_quotetw_disname = 0;
        let mut li_quotetw_usrname = 0;
        let mut li_quotetw_tstamp = 0;
        for i in li_tstamp + 1..li_ctn_lastl + 1 {
            let line = tw_as_lines[i].0;
            if re_quote_header.is_match(line) {
                if li_selftw_lastl + 5 <= li_ctn_lastl {
                    let _line1 = tw_as_lines[i + 1].0; // Dis Name
                    let line2 = tw_as_lines[i + 2].0; // Usr Name
                    let line4 = tw_as_lines[i + 4].0; // Tstamp
                    dbgg!((1244, _line1, line2, line4));
                    if line2.contains('@') && re_tstamp.is_match(line4) {
                        li_selftw_lastl = i - 1;
                        li_quotetw_disname = i + 1;
                        li_quotetw_usrname = i + 2;
                        li_quotetw_tstamp = i + 4;
                        li_quotetw_firstl = i + 5;
                    }
                }
            }
        }

        if li_selftw_lastl == 0 {
            li_selftw_lastl = li_ctn_lastl;
        }

        // compose all //

        let twid = {
            let mut begi = tw_as_lines[li_twid].1;
            let endi = tw_as_lines[li_twid].2;
            if orig.as_bytes()[begi] == 0xa {
                begi += 1;
            }
            Cow::from(&orig[begi..endi])
        };

        let username = {
            let mut begi = tw_as_lines[li_usrname].1;
            let endi = tw_as_lines[li_usrname].2;
            if orig.as_bytes()[begi] == 0xa {
                begi += 1;
            }
            Cow::from(&orig[begi..endi])
        };

        let dispname = {
            let mut begi = tw_as_lines[li_disname].1;
            let endi = tw_as_lines[li_disname].2;
            if orig.as_bytes()[begi] == 0xa {
                begi += 1;
            }
            Cow::from(&orig[begi..endi])
        };

        let post_tstamp = {
            let mut begi = tw_as_lines[li_tstamp].1;
            let endi = tw_as_lines[li_tstamp].2;
            if orig.as_bytes()[begi] == 0xa {
                begi += 1;
            }
            Cow::from(&orig[begi..endi])
        };

        let retweeter = if li_rt == 0 {
            None
        } else {
            let line = tw_as_lines[li_rt].0;
            // reaching here line must have matched previous RE
            if line.len() > 11 {
                Some(Cow::from(&line[1..line.len() - 10]))
            } else {
                None
            }
        };

        let reply_to = if li_rpl == 0 {
            None
        } else {
            let line = tw_as_lines[li_rpl].0;
            // reaching here line must have matched previous pattern
            if line.len() > 1 {
                Some(Cow::from(&line[1..]))
            } else {
                None
            }
        };

        let ctn = {
            let li_tw_bef_start = if reply_to.is_some() {
                li_tstamp + 2
            } else {
                li_tstamp
            };
            let mut begi = tw_as_lines[li_tw_bef_start].2;
            let endi = tw_as_lines[li_selftw_lastl].2;
            if orig.as_bytes()[begi] == 0xa {
                begi += 1;
            }
            if begi > endi {
                begi = endi;
            }
            Cow::from(&orig[begi..endi])
        };

        let quote = {
            if li_quotetw_firstl != 0
                && li_quotetw_tstamp != 0
                && li_quotetw_disname != 0
                && li_quotetw_usrname != 0
            {
                let begi_ctn = tw_as_lines[li_quotetw_firstl].1 + 1; // drop leading LF
                let endi_ctn = tw_as_lines[li_ctn_lastl].2;
                let begi_disname = tw_as_lines[li_quotetw_disname].1 + 1;
                let endi_disname = tw_as_lines[li_quotetw_disname].2;
                let begi_usrname = tw_as_lines[li_quotetw_usrname].1 + 1;
                let endi_usrname = tw_as_lines[li_quotetw_usrname].2;
                let begi_tstamp = tw_as_lines[li_quotetw_tstamp].1 + 1;
                let endi_tstamp = tw_as_lines[li_quotetw_tstamp].2;
                Some(QuoteTweet {
                    dispname: Cow::Borrowed(&orig[begi_disname..endi_disname]),
                    username: Cow::Borrowed(&orig[begi_usrname..endi_usrname]),
                    tstamp: Cow::Borrowed(&orig[begi_tstamp..endi_tstamp]),
                    ctn: Cow::Borrowed(&orig[begi_ctn..endi_ctn]),
                })
            } else {
                None
            }
        };

        dbgg!(
            &twid,
            &dispname,
            &username,
            &post_tstamp,
            &ctn,
            &retweeter,
            &reply_to,
            &quote
        );

        Ok(TweetOverview {
            twid,
            dispname,
            username,
            post_tstamp,
            ctn,
            retweeter,
            reply_to,
            quote,
            ..Default::default()
        })
    }

    // early
    #[allow(unused)]
    pub fn _from_str(orig: &str) -> Result<TweetOverview> {
        println!("-------\n{}\n------\n{:?}", orig, orig);

        Ok(TweetOverview {
            twid: Cow::from("111111111".to_string()),
            dispname: Cow::from("111111111".to_string()),
            username: Cow::from("111111111".to_string()),
            post_tstamp: Cow::from("111111111".to_string()),
            ctn: Cow::from("111111111".to_string()),
            ..Default::default()
        })
    }

    // methods //

    fn get_comb_name(&self, max_width_names: usize) -> String {
        let dis_width = UnicodeWidthStr::width(self.dispname.to_string().as_str());
        let usr_width = UnicodeWidthStr::width(self.username.to_string().as_str());
        let total_width = dis_width + usr_width + 2;

        let mut comb = String::from("");
        let mut ret = String::from("");
        let mut ret_width = 0;

        comb += &self.dispname;
        comb += "(";
        comb += &self.username;
        comb += ")";

        if total_width <= max_width_names {
            return comb;
        }

        let max_width_names = max_width_names - 3; // width(...)==3
        let mut previ = 0;
        for curri in 0..comb.len() {
            dbgg!(&ret);
            if comb.is_char_boundary(curri) {
                let sseg = &comb[previ..curri];
                ret_width += UnicodeWidthStr::width(sseg);
                if ret_width == max_width_names as usize {
                    ret += sseg;
                    break;
                } else if ret_width > max_width_names as usize {
                    break;
                } else {
                    ret += sseg;
                    previ = curri;
                }
            }
        }

        ret += "...";

        ret
    }

    pub fn pretty_print(
        &self,
        given_id: usize,
        nocolor: bool,
        asciiful: bool,
        wrap_width: u16,
        wrap_may_break: bool,
    ) -> String {
        let max_width_names = 55;

        let comb_name = self.get_comb_name(max_width_names);

        let header_part = if asciiful {
            format!(" {} | {} | {} |", given_id, comb_name, self.post_tstamp)
        } else {
            format!(" {} │ {} │ {} │", given_id, comb_name, self.post_tstamp)
        };
        dbgg!(&header_part);

        let colorful_header_part = if asciiful {
            format!(
                " {} | \x1b[36;1m{}\x1b[0m | \x1b[32m{}\x1b[0m |",
                given_id, comb_name, self.post_tstamp
            )
        } else {
            format!(
                " {} │ \x1b[36;1m{}\x1b[0m │ \x1b[32m{}\x1b[0m │",
                given_id, comb_name, self.post_tstamp
            )
        };

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
            &colorful_header_part
        };
        output += "\n";

        // bottom line
        let bottom_line = comm::replicate(line_comp, rtimes_line_comp);
        output += &bottom_line;
        output += line_tail_comp.1;
        output += &comm::replicate(
            line_comp,
            (80 - cols_line_comp * rtimes_line_comp - UnicodeWidthStr::width(line_tail_comp.1))
                / cols_line_comp,
        ); // bottom needs extra line_comp to reach 80
        output += "\n";

        // rt ply just before content
        let before_ctn = if let Some(v) = &self.retweeter {
            String::from(if nocolor { "" } else { "\x1b[33m" })
                + (if asciiful { "RT " } else { "⥁ " })
                + v
                + (if nocolor { "" } else { "\x1b[0m" })
                + "\n\n"
        } else if let Some(v) = &self.reply_to {
            String::from(if nocolor { "" } else { "\x1b[33m" })
                + (if asciiful { "RPL " } else { "⤇ " })
                + v
                + (if nocolor { "" } else { "\x1b[0m" })
                + "\n\n"
        } else {
            String::from("")
        };

        // self content
        let displayed_content = before_ctn + &self.ctn;
        dbgg!(&displayed_content);
        let refined_content = refine_twtxt(displayed_content.as_bytes());
        dbgg!(&refined_content);
        let mut wrapper = bwrap::EasyWrapper::new(&refined_content, wrap_width as usize).unwrap();
        let wrapped = if wrap_may_break {
            wrapper.wrap_use_style(bwrap::WrapStyle::MayBreak).unwrap()
        } else {
            wrapper.wrap().unwrap()
        };
        output += &wrapped;
        output += "\n";

        // quote content
        if let Some(quote) = &self.quote {
            let mut quote_all = String::from("");
            quote_all += "\n";
            quote_all += if asciiful {
                if nocolor {
                    "\"\"\""
                } else {
                    "\x1b[34m\"\"\"\x1b[0m"
                }
            } else {
                if nocolor {
                    "\u{201c}\u{201c}\u{201c}"
                } else {
                    "\x1b[34m\u{201c}\u{201c}\u{201c}\x1b[0m"
                }
            };
            quote_all += "\n";
            quote_all += &quote.ctn;
            quote_all += if asciiful {
                if nocolor {
                    "\n\"\"\""
                } else {
                    "\n\x1b[34m\"\"\"\x1b[0m"
                }
            } else {
                if nocolor {
                    "\n\u{201d}\u{201d}\u{201d}"
                } else {
                    "\n\x1b[34m\u{201d}\u{201d}\u{201d}\x1b[0m"
                }
            };
            quote_all += " - ";
            quote_all += &quote.dispname;
            quote_all += "(";
            quote_all += &quote.username;
            quote_all += ")";
            quote_all += ", ";
            quote_all += &quote.tstamp;

            quote_all += "\n"; // done
            let mut wrapper = bwrap::EasyWrapper::new(&quote_all, wrap_width as usize).unwrap();
            let wrapped = if wrap_may_break {
                wrapper.wrap_use_style(bwrap::WrapStyle::MayBreak).unwrap()
            } else {
                wrapper.wrap().unwrap()
            };

            output += &wrapped;
        }

        output += "\n";

        // link part
        let linkpart = if nocolor {
            format!(
                "[ http://twitter.com/{}/status/{} ]",
                self.username, self.twid
            )
        } else {
            format!(
                "\x1b[90m[ http://twitter.com/{}/status/{} ]\x1b[0m",
                self.username, self.twid
            )
        };
        output += &linkpart;
        output += "\n";
        output += "\n";

        output
    }
}

fn refine_twtxt(obytes: &[u8]) -> String {
    let mut nbytes = Vec::<u8>::new();
    let len = obytes.len();
    let mut is_in_at = false;
    for i in 0..len {
        let curr_byte = obytes[i];
        if curr_byte == b'\n' {
            if is_in_at {
                // nbytes.push(b' '); // commented means just skip this byte
                is_in_at = false;
            } else if i + 1 < len && obytes[i + 1] == b'@' {
                // nbytes.push(b' '); // commented means just skip this byte
                is_in_at = true;
            } else {
                nbytes.push(curr_byte);
            }
        } else {
            nbytes.push(curr_byte);
        }
    }

    String::from_utf8_lossy(&nbytes).to_string()
}

#[cfg(test)]
mod utst_parse_v1 {
    use super::*;

    #[test]
    fn normal() {
        let strres = "twtl_v1\n1656697812266909696\nTwitter\n@Twitter\n·\nMay 12\nSay goodbye to prying eyes and hello to secure conversations. We're giving early access to Encrypted Direct Messages v1 to our verified users.\n\nWe're excited to get feedback, improve the experience, and roll it out to even more users. Learn more:\nhelp.twitter.com\nAbout Encrypted Direct Messages – DMs | Twitter Help\nTwitter seeks to be the most trusted platform on the internet, and encrypted Direct Messages are an important part of that.\n2,190\n3,551\n18K\n8.2M";

        let twov = TweetOverview::from_str(strres).expect("buggy");
        assert_eq!(twov.twid, "1656697812266909696");
        assert_eq!(twov.ctn, "Say goodbye to prying eyes and hello to secure conversations. We're giving early access to Encrypted Direct Messages v1 to our verified users.\n\nWe're excited to get feedback, improve the experience, and roll it out to even more users. Learn more:\nhelp.twitter.com\nAbout Encrypted Direct Messages – DMs | Twitter Help\nTwitter seeks to be the most trusted platform on the internet, and encrypted Direct Messages are an important part of that.");
        assert_eq!(twov.dispname, "Twitter");
        assert_eq!(twov.username, "@Twitter");
        assert_eq!(twov.post_tstamp, "May 12");
        assert_eq!(twov.retweeter, None);
        assert_eq!(twov.reply_to, None);
        assert!(twov.quote.is_none());
    }

    #[test]
    fn normal_tstamp() {
        let strres = "twtl_v1\n1656697812266909696\nTwitter\n@Twitter\n·\n35m\nSay goodbye to prying eyes and hello to secure conversations. We're giving early access to Encrypted Direct Messages v1 to our verified users.\n\nWe're excited to get feedback, improve the experience, and roll it out to even more users. Learn more:\nhelp.twitter.com\nAbout Encrypted Direct Messages – DMs | Twitter Help\nTwitter seeks to be the most trusted platform on the internet, and encrypted Direct Messages are an important part of that.\n2,190\n3,551\n18K\n8.2M";

        let twov = TweetOverview::from_str(strres).expect("buggy");
        assert_eq!(twov.twid, "1656697812266909696");
        assert_eq!(twov.ctn, "Say goodbye to prying eyes and hello to secure conversations. We're giving early access to Encrypted Direct Messages v1 to our verified users.\n\nWe're excited to get feedback, improve the experience, and roll it out to even more users. Learn more:\nhelp.twitter.com\nAbout Encrypted Direct Messages – DMs | Twitter Help\nTwitter seeks to be the most trusted platform on the internet, and encrypted Direct Messages are an important part of that.");
        assert_eq!(twov.dispname, "Twitter");
        assert_eq!(twov.username, "@Twitter");
        assert_eq!(twov.post_tstamp, "35m");
        assert_eq!(twov.retweeter, None);
        assert_eq!(twov.reply_to, None);
        assert!(twov.quote.is_none());
    }

    #[test]
    fn retweeter() {
        let strres = "twtl_v1\n1657073688632762368\nTwitter Retweeted\nSubscriptions\n@Subscriptions\n·\nMay 13\nYou asked (loudly), we listened.\n\nWe’ve reduced the signup flow for creators from 27 steps to just 4.\n\nIt’s never been easier to earn a living on Twitter. Tap on “Monetization” in settings to apply today.\n2,400\n2,004\n12.6K\n9.5M";

        let twov = TweetOverview::from_str(strres).expect("buggy");
        assert_eq!(twov.twid, "1657073688632762368");
        assert_eq!(twov.ctn, "You asked (loudly), we listened.\n\nWe’ve reduced the signup flow for creators from 27 steps to just 4.\n\nIt’s never been easier to earn a living on Twitter. Tap on “Monetization” in settings to apply today.");
        assert_eq!(twov.dispname, "Subscriptions");
        assert_eq!(twov.username, "@Subscriptions");
        assert_eq!(twov.post_tstamp, "May 13");
        assert_eq!(twov.retweeter, Some("Twitter".into()));
        assert_eq!(twov.reply_to, None);
        assert!(twov.quote.is_none());
    }

    #[test]
    fn reply_to() {
        let strres = "twtl_v1\n1601692772678762496\nTwitter\n@Twitter\n·\nDec 11, 2022\nReplying to \n@Twittee\nwe’ll begin replacing that “official” label with a gold checkmark for businesses, and later in the week a grey checkmark for government and multilateral accounts\n246\n1,211\n4,315";

        let twov = TweetOverview::from_str(strres).expect("buggy");
        assert_eq!(twov.twid, "1601692772678762496");
        assert_eq!(twov.ctn, "we’ll begin replacing that “official” label with a gold checkmark for businesses, and later in the week a grey checkmark for government and multilateral accounts");
        assert_eq!(twov.dispname, "Twitter");
        assert_eq!(twov.username, "@Twitter");
        assert_eq!(twov.post_tstamp, "Dec 11, 2022");
        assert_eq!(twov.retweeter, None);
        assert_eq!(twov.reply_to, Some("@Twittee".into()));
        assert!(twov.quote.is_none());
    }

    #[test]
    fn image_alt() {
        let strres = "twtl_v1\n1577730467436138524\nTwitter\n@Twitter\n·\nOct 6, 2022\nwhoa, it works\n\nnow everyone can mix GIFs, videos, and images in one Tweet, available on iOS and Android\nGIF\nALT\nALT\n2,911\n4,449\n21.7K";

        let twov = TweetOverview::from_str(strres).expect("buggy");
        assert_eq!(twov.twid, "1577730467436138524");
        assert_eq!(twov.ctn, "whoa, it works\n\nnow everyone can mix GIFs, videos, and images in one Tweet, available on iOS and Android");
        assert_eq!(twov.dispname, "Twitter");
        assert_eq!(twov.username, "@Twitter");
        assert_eq!(twov.post_tstamp, "Oct 6, 2022");
        assert_eq!(twov.retweeter, None);
        assert_eq!(twov.reply_to, None);
        assert!(twov.quote.is_none());
    }

    #[test]
    fn spaces() {
        let strres = "twtl_v1\n1645992677727666176\nTwitter\n@Twitter\n·\nApr 12\nTwitter\nHost\nBBC Interview with Elon\n2.6M tuned in\n·\nApr 12\n·\n1:39:55\nPlay recording\n18.7K\n4,447\n10.8K\n4.9M";

        if let Err(MafaError::TweetNotRecoginized(c)) = TweetOverview::from_str(strres) {
            assert_eq!(c, 7);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn quote_1() {
        let strres = "twtl_v1\n1668829554448777216\nElon Musk\n@elonmusk\n·\nJun 14\nWhy ESG is the devil …\nQuote Tweet\nAaron Sibarium\n@aaronsibarium\n·\nJun 14\nNEW: From S&P Global to the London Stock Exchange, tobacco companies are crushing Tesla in the ESG ratings. How could cigarettes, which kill over 8 million a year, be deemed a more ethical investment than electric cars? \n\nOne answer: Tobacco’s gone woke.https://freebeacon.com/latest-news/how-tobacco-companies-are-crushing-esg-ratings/…\n8,898\n26K\n144.7K\n36.2M";

        let twov = TweetOverview::from_str(strres).expect("buggy");
        assert_eq!(twov.twid, "1668829554448777216");
        assert_eq!(twov.ctn, "Why ESG is the devil …");
        assert_eq!(twov.dispname, "Elon Musk");
        assert_eq!(twov.username, "@elonmusk");
        assert_eq!(twov.post_tstamp, "Jun 14");
        assert_eq!(twov.retweeter, None);
        assert_eq!(twov.reply_to, None);
        assert!(twov.quote.is_some());
        assert_eq!(twov.quote.as_ref().unwrap().dispname, "Aaron Sibarium");
        assert_eq!(twov.quote.as_ref().unwrap().username, "@aaronsibarium");
        assert_eq!(twov.quote.as_ref().unwrap().tstamp, "Jun 14");
        assert_eq!(twov.quote.as_ref().unwrap().ctn, "NEW: From S&P Global to the London Stock Exchange, tobacco companies are crushing Tesla in the ESG ratings. How could cigarettes, which kill over 8 million a year, be deemed a more ethical investment than electric cars? \n\nOne answer: Tobacco’s gone woke.https://freebeacon.com/latest-news/how-tobacco-companies-are-crushing-esg-ratings/…");
    }

    #[test]
    fn quote_2() {
        let strres = "twtl_v1\n1668675404939272196\nElon Musk Retweeted\nTesla\n@Tesla\n·\nJun 14\nWithin 15 mins, you can recover up to 200 miles/275 km\nQuote Tweet\nArash Malek\n@MinimalDuck\n·\nJun 9\nWhat a 15min charge looks like. @Tesla @elonmusk @TeslaCharging\n0:01 / 0:31\n2,130\n3,129\n20K\n9.1M";

        let twov = TweetOverview::from_str(strres).expect("buggy");
        assert_eq!(twov.twid, "1668675404939272196");
        assert_eq!(
            twov.ctn,
            "Within 15 mins, you can recover up to 200 miles/275 km"
        );
        assert_eq!(twov.dispname, "Tesla");
        assert_eq!(twov.username, "@Tesla");
        assert_eq!(twov.post_tstamp, "Jun 14");
        assert_eq!(twov.retweeter, Some("Elon Musk".into()));
        assert_eq!(twov.reply_to, None);
        assert!(twov.quote.is_some());
        assert_eq!(twov.quote.as_ref().unwrap().dispname, "Arash Malek");
        assert_eq!(twov.quote.as_ref().unwrap().username, "@MinimalDuck");
        assert_eq!(twov.quote.as_ref().unwrap().tstamp, "Jun 9");
        assert_eq!(
            twov.quote.as_ref().unwrap().ctn,
            "What a 15min charge looks like. @Tesla @elonmusk @TeslaCharging\n0:01 / 0:31"
        );
    }
}
