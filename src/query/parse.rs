use crate::query::{
    err_warn_support::{Message, MessageSeverity, MessageSink},
    lex::{Token, TokenType, lex},
};

#[derive(Debug, Clone, PartialEq)]
pub struct SearchQuery<'a> {
    source_range: std::ops::Range<usize>,
    query: SearchQueryTree<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchQueryTree<'a> {
    And(Vec<SearchQuery<'a>>),
    Or(Vec<SearchQuery<'a>>),
    Term(SearchTerm<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchTerm<'a> {
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
}

impl<'a> SearchTerm<'a> {
    pub fn from_token(tkn: &TokenType<'a>) -> Option<Self> {
        match tkn {
            TokenType::Term(s) => Some(Self::Term(*s)),
            TokenType::NegTerm(s) => Some(Self::NegTerm(*s)),
            TokenType::Exact(s) => Some(Self::Exact(*s)),
            TokenType::NegExact(s) => Some(Self::Term(*s)),

            TokenType::KeyVal(k, v) => Some(Self::KeyVal(*k, *v)),
            TokenType::KeyNeq(k, v) => Some(Self::KeyNeq(*k, *v)),
            TokenType::KeyGt(k, v) => Some(Self::KeyGt(*k, *v)),
            TokenType::KeyGte(k, v) => Some(Self::KeyGte(*k, *v)),
            TokenType::KeyLte(k, v) => Some(Self::KeyLte(*k, *v)),
            TokenType::KeyLt(k, v) => Some(Self::KeyLt(*k, *v)),

            TokenType::OpenParen | TokenType::Or | TokenType::CloseParen => None,
        }
    }
}

pub fn parse_str<'a>(src: &'a str, messages: impl MessageSink + 'a) -> Option<SearchQuery<'a>> {
    parse(&mut lex(src, &messages), &messages)
}

pub fn parse<'a, 'b>(
    tokens: &'b mut impl Iterator<Item = Token<'a>>,
    messages: &impl MessageSink,
) -> Option<SearchQuery<'a>> {
    let mut current_list = Vec::new();
    let mut num_combined_using_and = 0usize;
    let mut previous_token_or = false;

    let mut source_start = None;
    let mut source_end = 0;

    while let Some(t) = tokens.next() {
        source_start.get_or_insert(t.source_range.start);
        source_end = t.source_range.end;

        if let Some(term) = SearchTerm::from_token(&t.token) {
            add_query_tree(
                SearchQueryTree::Term(term),
                t,
                &mut previous_token_or,
                &mut current_list,
                &mut num_combined_using_and,
            );
            continue;
        }
        match t.token {
            TokenType::OpenParen => {
                add_query_element(
                    parse(&mut *tokens, messages)?,
                    &mut previous_token_or,
                    &mut current_list,
                    &mut num_combined_using_and,
                );
            }
            TokenType::CloseParen => break,
            TokenType::Or => {
                if current_list.len() == 0 {
                    messages.send(Message {
                        msg_type: MessageSeverity::Error,
                        msg_content: String::from("'or' operator is at the start of a list"),
                        byte_pos: t.source_range.start,
                        source_phase_index: 1,
                    });
                    return None;
                }

                if num_combined_using_and >= 2 {
                    messages.send(Message {
                        msg_type: MessageSeverity::Warning,
                        msg_content: String::from("Mixed 'or' operators without using parentheses. You can clarify your intent by grouping your search terms."),
                        byte_pos: t.source_range.start,
                        source_phase_index: 1,
                    });
                }

                previous_token_or = true;
            }
            _ => unreachable!("All other cases should be convertable into a token"),
        }
    }

    if current_list.len() == 1 {
        return Some(current_list.pop().unwrap());
    } else {
        return Some(SearchQuery {
            query: SearchQueryTree::And(current_list),
            source_range: (source_start.unwrap_or(0))..source_end,
        });
    }
}

fn add_query_element<'a, 'b>(
    q: SearchQuery<'a>,
    previous_token_or: &'b mut bool,
    current_list: &'b mut Vec<SearchQuery<'a>>,
    num_combined_using_and: &'b mut usize,
) {
    if *previous_token_or {
        *num_combined_using_and = 0;
        //this should always be 'some' because
        //the parser checks for a left-hand-side when it recognizes
        //an 'or' operator.
        if let Some(sq) = current_list.last_mut() {
            match sq.query {
                SearchQueryTree::Or(ref mut items) => items.push(q),
                SearchQueryTree::And(_) | SearchQueryTree::Term(_) => {
                    let or_parameters = vec![current_list.pop().unwrap(), q];
                    current_list.push(SearchQuery {
                        source_range: (or_parameters[0].source_range.start)
                            ..(or_parameters[1].source_range.end),
                        query: SearchQueryTree::Or(or_parameters),
                    })
                }
            }
            return;
        }
    }
    current_list.push(q);
}

fn add_query_tree<'a, 'b>(
    query: SearchQueryTree<'a>,
    t: Token<'a>,
    previous_token_or: &'b mut bool,
    current_list: &'b mut Vec<SearchQuery<'a>>,
    num_combined_using_and: &'b mut usize,
) {
    add_query_element(
        SearchQuery {
            source_range: t.source_range,
            query,
        },
        previous_token_or,
        current_list,
        num_combined_using_and,
    );
}

#[cfg(test)]
#[test]
fn test_parser() {
    use crate::query::err_warn_support::DebugPrintMessages;

    for src in [
        r#"!"sift through sands""#,
        r#"o:"~ enters tapped" "#,
        "not:reprint e:c16",
        "is:dual",
        "c>=br is:spell f:duel",
        "pow>tou c:w t:creature",
        "devotion:{u/b}{u/b}{u/b}",
        "-fire c:r t:instant",
        "through (depths or sands or mists)",
        "t:legendary (t:goblin or t:elf)",
    ] {
        dbg!(parse_str(src, DebugPrintMessages));
    }
}
