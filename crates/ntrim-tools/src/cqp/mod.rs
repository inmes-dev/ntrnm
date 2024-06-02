use std::collections::HashMap;
use std::fmt::Display;
use anyhow::Error;
use prost::Message;

#[derive(Debug)]
pub enum CQCode {
    Special {
        cq_type: String,
        params: HashMap<String, String>,
    },
    Text(String),
}

impl Display for CQCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CQCode::Special { cq_type, params } => {
                write!(f, "[CQ:{},{}]", cq_type, params
                    .iter()
                    .map(|(k, v)| {
                        let encoded_v = v
                            .replace("&", "&amp;")
                            .replace("[", "&#91;")
                            .replace("]", "&#93;")
                            .replace(",", "&#44;");
                        format!("{}={}", k, encoded_v)
                    })
                    .collect::<Vec<_>>()
                    .join(",")
                )
            }
            CQCode::Text(text) => write!(f, "{}", text
                .replace("&", "&amp;")
                .replace("[", "&#91;")
                .replace("]", "&#93;")
            )
        }
    }
}

fn utf8_next_len(str: &[u8], offset: usize) -> usize {
    let c = str[offset];
    if c >= 0xfc {
        6
    } else if c >= 0xf8 {
        5
    } else if c >= 0xf0 {
        4
    } else if c >= 0xe0 {
        3
    } else if c >= 0xc0 {
        2
    } else if c > 0x0 {
        1
    } else {
        0
    }
}

pub fn parse_cq_by_myself(data: &[u8]) -> Result<Vec<CQCode>, Error> {
    let mut start = false;
    let mut cache = String::new();
    let mut result = Vec::new();
    let mut cq_type = String::new();
    let mut key = String::new();
    let mut params = HashMap::new();

    let mut i = 0;
    while i < data.len() {
        let utf_char_len = utf8_next_len(data, i);
        if utf_char_len == 0 {
            continue;
        }
        let utf_char = &data[i..i+utf_char_len];
        let c = unsafe { std::str::from_utf8_unchecked(utf_char) };
        if c == "[" {
            if start {
                return Err(Error::msg("Illegal code"))
            } else {
                if !cache.is_empty() {
                    let text = CQCode::Text(cache
                        .replace("&#91;", "[")
                        .replace("&#93;", "]")
                        .replace("&amp;", "&")
                    );
                    result.push(text);
                    cache.clear();
                }
                let cq_flag = unsafe { std::str::from_utf8_unchecked(&data[i..i + 4]) };
                if cq_flag == "[CQ:" {
                    start = true;
                    i += 3;
                } else {
                    cache.push_str(c);
                }
            }
        } else if c == "=" {
            if start {
                if cache.is_empty() {
                    return Err(Error::msg("Illegal code"))
                }
                if key.is_empty() {
                    key = cache.clone();
                    cache.clear();
                } else {
                    cache.push_str(c);
                }
            } else {
                cache.push_str(c);
            }
        } else if c == "," {
            if start {
                if cache.is_empty() {
                    return Err(Error::msg("Illegal code"))
                }
                if cq_type.is_empty() {
                    cq_type = cache.clone();
                    cache.clear();
                } else {
                    if !key.is_empty() {
                        params.insert(key.clone(), cache.
                            replace("&#91;", "[")
                            .replace("&#93;", "]")
                            .replace("&#44;", ",")
                            .replace("&amp;", "&")
                        );
                        key.clear();
                        cache.clear();
                    }
                }
            } else {
                cache.push_str(c);
            }
        } else if c == "]" {
            if start {
                if !cache.is_empty() {
                    if !key.is_empty() {
                        params.insert(key.clone(), cache
                            .replace("&#91;", "[")
                            .replace("&#93;", "]")
                            .replace("&#44;", ",")
                            .replace("&amp;", "&")
                        );
                    } else {
                        cq_type = cache.clone();
                        cache.clear();
                    }
                    let cq_code = CQCode::Special {
                        cq_type: cq_type.clone(),
                        params: params.clone(),
                    };
                    result.push(cq_code);
                    cq_type.clear();
                    params.clear();
                    key.clear();
                    cache.clear();
                    start = false;
                } else {
                    cache.push_str(c);
                }
            } else {
                cache.push_str(c);
            }
        } else {
            cache.push_str(c);
            i += utf_char_len - 1;
        }
        i += 1;
    }
    if !cache.is_empty() {
        let text = CQCode::Text(cache
            .replace("&#91;", "[")
            .replace("&#93;", "]")
            .replace("&amp;", "&")
        );
        result.push(text);
    }
    Ok(result)
}
