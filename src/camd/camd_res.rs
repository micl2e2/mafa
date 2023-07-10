use crate::error::Result;

#[derive(Debug, Default)]
pub struct LevelExpained {
    pronun: Option<String>,
}

impl LevelExpained {
    fn from_str(s: &str) -> Result<Self> {
        let mut lines = Vec::<&str>::new();

        let bytes = s.as_bytes();

        let mut begi = 0usize;
        let mut endi = 0usize;

        for i in 0..bytes.len() {
            if i + 2 <= bytes.len() && &bytes[i..i + 2] == br"\n" {
                endi = i;
                lines.push(&s[begi..endi]);
                begi = endi + 1;
            }
        }

        dbgg!(&lines);

        Ok(Default::default())
    }
}

#[derive(Debug, Default)]
pub struct CamdResult(Vec<LevelExpained>);

impl CamdResult {
    pub fn from_string(s: String) -> Result<Self> {
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
