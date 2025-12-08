use std::ops::Range;

use crate::query::err_warn_support::{Message, MessageSink};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType<'a> {
    Term(&'a str),
    NegTerm(&'a str),

    Exact(&'a str),
    NegExact(&'a str),

    KeyVal(&'a str, &'a str),

    KeyNeq(&'a str, &'a str),
    KeyGt(&'a str, &'a str),
    KeyLt(&'a str, &'a str),
    KeyLte(&'a str, &'a str),
    KeyGte(&'a str, &'a str),

    OpenParen,
    CloseParen,
    Or,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub source_range: Range<usize>,
    pub token: TokenType<'a>,
}

pub fn lex<'b, 'a: 'b>(
    src: &'a str,
    messages: &'b impl MessageSink,
) -> impl Iterator<Item = Token<'a>> + 'b {
    let mut current_word = 0..0;
    let mut iter = src.char_indices();

    let mut in_quote = false;
    let mut next_should_end_word = false;
    //only state that's initially true: a quote _can_ be the first
    // character of the source query
    let mut quote_may_start = true;

    std::iter::from_fn(move || {
        loop {
            let Some((i, ch)) = iter.next() else {
                break;
            };

            if ch == '"' {
                if in_quote {
                    in_quote = false;

                    //words always end after the quote exits.
                    //this follows from the principle set out in the next part.
                    next_should_end_word = true;

                    //since we won't be falling through to the base case,
                    //update the current word to include the quote.
                    current_word.end = i + 1;

                    //more for housekeeping than for anything else.
                    quote_may_start = false;

                    continue;
                } else {
                    in_quote = quote_may_start;

                    // if the user starts a quote somewhere where there shouldn't be one,
                    // such as in the middle of a word, then complain. this is probably because
                    // of something like 'ampersand" "and', which isn't good style and isn't compatible.
                    if !quote_may_start {
                        messages.send(Message {
                            msg_type: super::err_warn_support::MessageSeverity::Error,
                            msg_content: String::from("Unexpected start quote. Quotes can only be at the start of a term, after a modifier like '!' or '-', or directly after a keyword, like 'o:'."),
                            byte_pos: i,
                            source_phase_index: 0,
                        });
                        return None;
                    }
                }
            }

            //catch attempts to use regex and let
            // the user know we don't support that yet
            if quote_may_start && ch == '/' {
                messages.send(Message {
                    msg_type: super::err_warn_support::MessageSeverity::Warning,
                    msg_content: String::from("On Scryfall, forward slashes are used for regular expressions, but we don't support that yet. Please use quotes to get rid of this warning."),
                    byte_pos: i,
                    source_phase_index: 0,
                });
            }

            let is_word_end = ch.is_ascii_whitespace()
                || &src[current_word.clone()] == "("
                || &src[current_word.clone()] == ")"
                || ch == ')'
                || ch == '(';

            if !in_quote && is_word_end {
                next_should_end_word = false;

                let word_src_range = current_word.clone();
                let word_src = &src[word_src_range.clone()];

                quote_may_start = true;

                if ch.is_ascii_whitespace() {
                    current_word = (i + 1)..(i + 1);
                } else {
                    current_word = (i)..(i + 1);
                }

                if !word_src.is_empty() {
                    let tkn =
                        word_to_token(word_src, &messages, word_src_range.clone()).map(|token| {
                            Token {
                                token,
                                source_range: word_src_range,
                            }
                        });

                    return tkn;
                }
            } else if next_should_end_word {
                //we _should_ have ended a word, but we didn't. complain!
                messages.send(Message {
                    msg_type: super::err_warn_support::MessageSeverity::Error,
                    msg_content: String::from(
                        "Unexpected quote. Quotes should only close at the end of a search term.",
                    ),
                    byte_pos: i,
                    source_phase_index: 0,
                });
                return None;
            } else {
                quote_may_start =
                    (quote_may_start && (ch == '!' || ch == '-')) || (!in_quote && ch == ':');
                current_word.end = i + 1;
            }
        }

        if in_quote {
            messages.send(Message {
                msg_type: super::err_warn_support::MessageSeverity::Error,
                msg_content: String::from(
                    "There are unbalanced quotes in your search. Keep in mind that backslash-escapes (like \\\") aren't supported.",
                ),
                byte_pos: src.len() - 1,
                source_phase_index: 0,
            });
            return None;
        }

        if current_word.is_empty() {
            return None;
        } else {
            let word = &src[current_word.clone()];

            let token = word_to_token(word, &messages, current_word.clone()).map(|token| Token {
                source_range: current_word.clone(),
                token,
            });

            current_word = src.len()..src.len();

            return token;
        }
    })
}

macro_rules! check_binop {
    ($src_op:literal, $variant_unnegated:ident, $variant_negated:ident, $word:expr, $messages:expr, $negated:expr, $exact:expr, $idx:expr) => {
        let kv = $word.split_once($src_op);
    if kv.is_some_and(|(k, _)| valid_keyword(k)) {
        let (k, v) = kv.unwrap();

        if $exact {
            ($messages).send(Message {
                msg_type: super::err_warn_support::MessageSeverity::Error,
                msg_content: String::from("Unexpected exclaimation mark. Only basic search terms can use the exact search; it does not work for keywords."),
                byte_pos: $idx.start,
                source_phase_index: 0,
            });
            return None;
        }

        if $negated {
            return Some(TokenType::$variant_negated(k, unwrap_quotes(v)));
        } else {
            return Some(TokenType::$variant_unnegated(k, unwrap_quotes(v)));
        }
    }
    };
}

fn word_to_token<'a>(
    mut word: &'a str,
    mut messages: impl MessageSink,
    idx: Range<usize>,
) -> Option<TokenType<'a>> {
    let negated = word.starts_with("-");
    if negated {
        word = &word[1..];
    }
    let exact = word.starts_with("!");
    if exact {
        word = &word[1..];
    }
    let entirely_quoted = word.starts_with('"') && word.ends_with('"');
    if entirely_quoted {
        word = &word[1..(word.len() - 1)];
    }

    //ignore further parsing if it was quoted, of course
    if entirely_quoted {
        match (exact, negated) {
            (true, true) => return Some(TokenType::NegExact(word)),
            (true, false) => return Some(TokenType::Exact(word)),
            (false, true) => return Some(TokenType::NegTerm(word)),
            (false, false) => return Some(TokenType::Term(word)),
        }
    }

    if word == "(" {
        return Some(TokenType::OpenParen);
    }
    if word == ")" {
        return Some(TokenType::CloseParen);
    }

    if word == "or" || word == "OR" {
        return Some(TokenType::Or);
    }

    //check for weird casing of ORs

    if word.eq_ignore_ascii_case("or") {
        messages.send(Message {
            msg_type: crate::query::err_warn_support::MessageSeverity::Warning,
            msg_content: format!("You used '{word}'; try 'or'/'OR' instead. If you want to search cards' names, try \"or\"."),
            byte_pos: idx.start,
            source_phase_index: 0,
        });
    }

    check_binop!(":", KeyVal, KeyNeq, word, messages, negated, exact, idx);
    check_binop!("!=", KeyNeq, KeyVal, word, messages, negated, exact, idx);
    check_binop!("==", KeyVal, KeyNeq, word, messages, negated, exact, idx);
    check_binop!(">=", KeyGte, KeyLt, word, messages, negated, exact, idx);
    check_binop!(">", KeyGt, KeyLte, word, messages, negated, exact, idx);
    check_binop!("<=", KeyLte, KeyGt, word, messages, negated, exact, idx);
    check_binop!("<", KeyLt, KeyGte, word, messages, negated, exact, idx);

    match (exact, negated) {
        (true, true) => return Some(TokenType::NegExact(word)),
        (true, false) => return Some(TokenType::Exact(word)),
        (false, true) => return Some(TokenType::NegTerm(word)),
        (false, false) => return Some(TokenType::Term(word)),
    }
}

fn unwrap_quotes(s: &str) -> &str {
    if s.starts_with('"') && s.ends_with('"') {
        return &s[1..(s.len() - 1)];
    } else {
        return s;
    }
}

fn valid_keyword(k: &str) -> bool {
    k.bytes().all(|x| x.is_ascii_alphanumeric())
}

#[cfg(test)]
#[test]
fn test_lexer() {
    use crate::query::err_warn_support::DebugPrintMessages;

    assert_eq!(
        lex(r#"!"sift through sands""#, &DebugPrintMessages)
            .map(|x| x.token)
            .collect::<Vec<_>>(),
        vec![TokenType::Exact("sift through sands")]
    );

    assert_eq!(
        lex(r#"o:"~ enters tapped" "#, &DebugPrintMessages)
            .map(|x| x.token)
            .collect::<Vec<_>>(),
        vec![TokenType::KeyVal("o", "~ enters tapped")]
    );

    assert_eq!(
        lex("not:reprint e:c16", &DebugPrintMessages)
            .map(|x| x.token)
            .collect::<Vec<_>>(),
        vec![
            TokenType::KeyVal("not", "reprint"),
            TokenType::KeyVal("e", "c16")
        ]
    );

    assert_eq!(
        lex("is:dual", &DebugPrintMessages)
            .map(|x| x.token)
            .collect::<Vec<_>>(),
        vec![TokenType::KeyVal("is", "dual")]
    );

    assert_eq!(
        lex("c>=br is:spell f:duel", &DebugPrintMessages)
            .map(|x| x.token)
            .collect::<Vec<_>>(),
        vec![
            TokenType::KeyGte("c", "br"),
            TokenType::KeyVal("is", "spell"),
            TokenType::KeyVal("f", "duel")
        ]
    );

    assert_eq!(
        lex("pow>tou c:w t:creature", &DebugPrintMessages)
            .map(|x| x.token)
            .collect::<Vec<_>>(),
        vec![
            TokenType::KeyGt("pow", "tou"),
            TokenType::KeyVal("c", "w"),
            TokenType::KeyVal("t", "creature")
        ]
    );

    assert_eq!(
        lex("devotion:{u/b}{u/b}{u/b}", &DebugPrintMessages)
            .map(|x| x.token)
            .collect::<Vec<_>>(),
        vec![TokenType::KeyVal("devotion", "{u/b}{u/b}{u/b}")]
    );

    assert_eq!(
        lex("-fire c:r t:instant", &DebugPrintMessages)
            .map(|x| x.token)
            .collect::<Vec<_>>(),
        vec![
            TokenType::NegTerm("fire"),
            TokenType::KeyVal("c", "r"),
            TokenType::KeyVal("t", "instant")
        ]
    );

    assert_eq!(
        lex("through (depths or sands or mists)", &DebugPrintMessages)
            .map(|x| x.token)
            .collect::<Vec<_>>(),
        vec![
            TokenType::Term("through"),
            TokenType::OpenParen,
            TokenType::Term("depths"),
            TokenType::Or,
            TokenType::Term("sands"),
            TokenType::Or,
            TokenType::Term("mists"),
            TokenType::CloseParen,
        ]
    );

    assert_eq!(
        lex("t:legendary (t:goblin or t:elf)", &DebugPrintMessages)
            .map(|x| x.token)
            .collect::<Vec<_>>(),
        vec![
            TokenType::KeyVal("t", "legendary"),
            TokenType::OpenParen,
            TokenType::KeyVal("t", "goblin"),
            TokenType::Or,
            TokenType::KeyVal("t", "elf"),
            TokenType::CloseParen
        ]
    );
}
