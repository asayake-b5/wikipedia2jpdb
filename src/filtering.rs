use std::{borrow::Cow, collections::HashSet};

use regex::Regex;

lazy_static::lazy_static! {
    pub static ref IGNORED_ENTRIES: HashSet<&'static str> = {
        let mut set_to_ignore: HashSet<&str> = HashSet::new();
        set_to_ignore.insert("\n");
        set_to_ignore.insert("");
        set_to_ignore.insert("\n\n");
        set_to_ignore.insert(" ");
        set_to_ignore.insert("（");
        set_to_ignore.insert("）");
        set_to_ignore.insert("^");
        set_to_ignore.insert(" - ");

        set_to_ignore
    };
}

// Useless because of fiter ascii
// pub fn filter_urls(before: &str) -> Cow<str> {
//     lazy_static::lazy_static! {
//         // regex from https://github.com/rust-lang/regex/issues/127#issuecomment-1311560695
//         static ref URL_REGEX: Regex =
//             Regex::new(r"((https?://)?[^\s.]+\.[\w][^\s]+)").unwrap();
//     }
//     URL_REGEX.replace_all(before, "")
// }

pub fn filter_ascii(before: &str) -> Cow<str> {
    lazy_static::lazy_static! {
        static ref ALPHA_REGEX: Regex =
            Regex::new(r"[\x00-\x7F]+").unwrap();
    }
    ALPHA_REGEX.replace_all(before, "")
}

pub fn filter_other(before: &str) -> Cow<str> {
    lazy_static::lazy_static! {
        static ref OTHER_REGEX: Regex =
            Regex::new(r"[ \n（）、。『』 “\x3000-\x303F\x31F0-\x31FF\x3220-\x3243\x3280-\x337F]+").unwrap();
    }
    OTHER_REGEX.replace_all(before, "")
}

pub fn filter_noise(before: &str) -> String {
    // filter_ascii(&filter_urls(before)).to_string()
    filter_other(&filter_ascii(before)).to_string()
}
