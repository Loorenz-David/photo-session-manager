//! PH3: the phone's token, from `Authorization: Bearer <token>` or the `pair=<token>` query (images
//! and the WebSocket cannot set headers). The header wins when both are given.
use axum::http::HeaderMap;

/// The token a request carries, if any (never logged).
pub fn token_of(headers: &HeaderMap, query: Option<&str>) -> Option<String> {
    if let Some(value) = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        && let Some(token) = value.strip_prefix("Bearer ").map(str::trim)
        && !token.is_empty()
    {
        return Some(token.to_owned());
    }
    query?
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| *key == "pair")
        .map(|(_, value)| percent_decode(value))
        .filter(|token| !token.is_empty())
}

/// `%XX` and `+` as a query value carries them (a base64url token has neither, but a phone may
/// encode).
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => match u8::from_str_radix(&value[i + 1..i + 3], 16) {
                Ok(b) => {
                    out.push(b);
                    i += 3;
                    continue;
                }
                Err(_) => out.push(b'%'),
            },
            b'+' => out.push(b' '),
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn bearer(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            axum::http::header::AUTHORIZATION,
            HeaderValue::from_str(value).unwrap(),
        );
        h
    }

    #[test]
    fn the_bearer_header_carries_the_token() {
        assert_eq!(
            token_of(&bearer("Bearer abc"), None).as_deref(),
            Some("abc")
        );
        assert_eq!(
            token_of(&bearer("Bearer abc"), Some("pair=zzz")).as_deref(),
            Some("abc")
        );
    }

    #[test]
    fn the_pair_query_carries_the_token() {
        assert_eq!(
            token_of(&HeaderMap::new(), Some("pair=abc")).as_deref(),
            Some("abc")
        );
        assert_eq!(
            token_of(&HeaderMap::new(), Some("x=1&pair=a-b_c")).as_deref(),
            Some("a-b_c")
        );
        assert_eq!(
            token_of(&HeaderMap::new(), Some("pair=a%2Db")).as_deref(),
            Some("a-b")
        );
    }

    #[test]
    fn no_token_is_none() {
        assert_eq!(token_of(&HeaderMap::new(), None), None);
        assert_eq!(token_of(&HeaderMap::new(), Some("pair=")), None);
        assert_eq!(token_of(&HeaderMap::new(), Some("other=abc")), None);
        assert_eq!(token_of(&bearer("Basic abc"), None), None);
        assert_eq!(token_of(&bearer("Bearer "), None), None);
    }
}
