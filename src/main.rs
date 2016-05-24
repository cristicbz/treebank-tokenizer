extern crate regex;
extern crate fnv;

use std::io;
use std::io::prelude::*;
use fnv::FnvHasher;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::ascii::AsciiExt;
use regex::bytes::Regex;


const SUBSTITUTIONS: &'static [(&'static str, &'static [u8])] = &[
    // Whitespace.
    (r#"\s+"#, b" "),

    // Starting quotes.
    (r#"^""#, br#"``"#),
    (r#"(``)"#, br#" $1 "#),
    (r#"([ (\[{<])""#, br#"$1 `` "#),

    // Punctation.
    (r#"([:,])([^\d])"#, br#" $1 $2"#),
    (r#"([:,])$"#, br#" $1 "#),
    (r#"\.\.\."#, br#" ... "#),
    (r#"[;@#$%&]"#, br#" $0 "#),
    (r#"([^\.])(\.)([\]\)}>"']*)\s*$"#, br#"$1 $2$3 "#),
    (r#"[?!]"#, br#" $0 "#),

    // Contractions.
    (r#"([^'])' "#, br#"$1 ' "#),
    (r#"(?i)\b(can)(not)\b"#, b"$1 $2"),
    (r#"(?i)\b(d)('ye)\b"#, b"$1 $2"),
    (r#"(?i)\b(gim)(me)\b"#, b"$1 $2"),
    (r#"(?i)\b(gon)(na)\b"#,  b"$1 $2"),
    (r#"(?i)\b(got)(ta)\b"#, b"$1 $2"),
    (r#"(?i)\b(lem)(me)\b"#, b"$1 $2"),
    (r#"(?i)\b(mor)('n)\b"#, b"$1 $2"),
    (r#"(?i)\b(wan)(na)(?:$|\s)"#, b"$1 $2"),
    (r#"(?i)(?:^|\s)('t)(is)\b"#, b"$1 $2"),
    (r#"(?i)(?:^|\s)('t)(was)\b"#, b"$1 $2"),

    // Parentheses and brackets.
    (r#"[\]\[\(\)\{\}<>]"#, br#" $0 "#),
    (r#"--"#, br#" -- "#),

    // End quotes.
    (r#"""#, br#" '' "#),
    (r#"(\S)('')"#, br#"$1 $2 "#),
    (r#"([^' ])('s|'m|'d|')(?:\s|$)"#, br#"$1 $2 "#),
    (r#"([^' ])('ll|'re|'ve|n't)(?:\s|$)"#, br#"$1 $2 "#),
];

fn main() {
    let mut word_counts = HashMap::<Vec<u8>, i64, _>::with_hasher(
        BuildHasherDefault::<FnvHasher>::default());
    let substitutions = SUBSTITUTIONS.iter()
                                         .map(|&(regex, replace)| {
                                             (Regex::new(regex).expect("Regex failed."), replace)
                                         })
                                         .collect::<Vec<_>>();
    let (stdin, stdout) = (io::stdin(), io::stdout());
    let (mut locked_stdin, mut locked_stdout) = (stdin.lock(), stdout.lock());

    loop {
        let mut bytes = vec![];
        let num_bytes = locked_stdin.read_until(b'\n', &mut bytes).expect("Failed to read line");
        if num_bytes == 0 {
            break;
        }

        for &(ref regex, replace) in &substitutions {
            bytes = regex.replace_all(&bytes, replace);
        }

        for word in bytes.split(|&c| c == b' ') {
            if !word.is_empty() {
                *word_counts.entry(word.to_ascii_lowercase()).or_insert(0) += 1
            }
        }
    }

    let mut word_count_pairs = word_counts.into_iter().collect::<Vec<_>>();
    word_count_pairs.sort_by_key(|&(_, count)| -count);

    for (word, count) in word_count_pairs {
        locked_stdout.write(&word).expect("Failed write");
        writeln!(locked_stdout, "\t{}", count).expect("Failed write");
    }
}
