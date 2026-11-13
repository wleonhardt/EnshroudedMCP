//! Admission of a request to the loopback endpoint, ported from ValheimMCP's
//! `RequestPolicy`: no browser requests (any `Origin`), no non-loopback `Host`
//! (DNS rebinding), a token when one is configured, and a body cap.

/// The largest request body the endpoint reads.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Why a request is refused, with the HTTP status it gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    CrossOrigin,
    NotLoopbackHost,
    Token,
    TooLarge,
}

impl Refusal {
    #[must_use]
    pub fn status(self) -> u16 {
        match self {
            Self::Token => 401,
            Self::TooLarge => 413,
            Self::CrossOrigin | Self::NotLoopbackHost => 403,
        }
    }

    #[must_use]
    pub fn message(self) -> &'static str {
        match self {
            Self::CrossOrigin => "cross-origin request refused (Origin header present)",
            Self::NotLoopbackHost => "Host header refused (not a loopback name)",
            Self::Token => "missing or wrong token",
            Self::TooLarge => "request body too large (limit 1 MB)",
        }
    }
}

/// The headers admission looks at, as the client sent them.
#[derive(Debug, Default, Clone, Copy)]
pub struct Admission<'a> {
    pub origin: Option<&'a str>,
    pub host: Option<&'a str>,
    pub authorization: Option<&'a str>,
    pub content_length: usize,
}

/// `None` when the request may proceed. `token` is the configured token;
/// `None` or empty means no token is required.
#[must_use]
pub fn reject(req: &Admission<'_>, token: Option<&str>) -> Option<Refusal> {
    if req.origin.is_some_and(|o| !o.is_empty()) {
        return Some(Refusal::CrossOrigin);
    }
    if !is_loopback_name(host_name(req.host.unwrap_or(""))) {
        return Some(Refusal::NotLoopbackHost);
    }
    if let Some(want) = token.filter(|t| !t.is_empty()) {
        let got = req
            .authorization
            .and_then(|a| {
                a.get(..7)
                    .filter(|p| p.eq_ignore_ascii_case("Bearer "))
                    .map(|_| a[7..].trim())
            })
            .unwrap_or("");
        if !fixed_time_eq(got.as_bytes(), want.as_bytes()) {
            return Some(Refusal::Token);
        }
    }
    if req.content_length > MAX_BODY_BYTES {
        return Some(Refusal::TooLarge);
    }
    None
}

/// The `Host` header without its port: `127.0.0.1:8751` gives `127.0.0.1`,
/// `[::1]:8751` gives `[::1]`.
#[must_use]
pub fn host_name(host: &str) -> &str {
    if host.starts_with('[') {
        return host.find(']').map_or(host, |end| &host[..=end]);
    }
    host.split(':').next().unwrap_or("")
}

#[must_use]
pub fn is_loopback_name(h: &str) -> bool {
    h == "127.0.0.1" || h == "[::1]" || h.eq_ignore_ascii_case("localhost")
}

/// Compares in a time that depends on the lengths only, not on where the
/// inputs differ.
#[must_use]
pub fn fixed_time_eq(a: &[u8], b: &[u8]) -> bool {
    let mut diff = a.len() ^ b.len();
    for (x, y) in a.iter().zip(b) {
        diff |= usize::from(x ^ y);
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok() -> Admission<'static> {
        Admission {
            host: Some("127.0.0.1:8751"),
            ..Admission::default()
        }
    }

    #[test]
    fn a_plain_loopback_request_passes() {
        assert_eq!(reject(&ok(), None), None);
        assert_eq!(
            reject(
                &Admission {
                    host: Some("localhost"),
                    ..ok()
                },
                None
            ),
            None
        );
        assert_eq!(
            reject(
                &Admission {
                    host: Some("[::1]:8751"),
                    ..ok()
                },
                None
            ),
            None
        );
    }

    #[test]
    fn any_origin_is_refused() {
        let r = Admission {
            origin: Some("http://localhost:3000"),
            ..ok()
        };
        assert_eq!(reject(&r, None), Some(Refusal::CrossOrigin));
        assert_eq!(Refusal::CrossOrigin.status(), 403);
    }

    #[test]
    fn a_rebinding_host_is_refused() {
        for host in [
            "evil.example",
            "127.0.0.1.evil.example",
            "",
            "10.0.0.5:8751",
        ] {
            assert_eq!(
                reject(
                    &Admission {
                        host: Some(host),
                        ..ok()
                    },
                    None
                ),
                Some(Refusal::NotLoopbackHost),
                "{host}"
            );
        }
        assert_eq!(
            reject(&Admission { host: None, ..ok() }, None),
            Some(Refusal::NotLoopbackHost)
        );
    }

    #[test]
    fn the_token_is_required_when_configured() {
        assert_eq!(reject(&ok(), Some("s3cret")), Some(Refusal::Token));
        assert_eq!(
            reject(
                &Admission {
                    authorization: Some("Bearer wrong"),
                    ..ok()
                },
                Some("s3cret")
            ),
            Some(Refusal::Token)
        );
        assert_eq!(
            reject(
                &Admission {
                    authorization: Some("bearer s3cret"),
                    ..ok()
                },
                Some("s3cret")
            ),
            None
        );
        assert_eq!(reject(&ok(), Some("")), None, "an empty token means none");
        assert_eq!(Refusal::Token.status(), 401);
    }

    #[test]
    fn a_large_body_is_refused() {
        let r = Admission {
            content_length: MAX_BODY_BYTES + 1,
            ..ok()
        };
        assert_eq!(reject(&r, None), Some(Refusal::TooLarge));
        assert_eq!(Refusal::TooLarge.status(), 413);
    }

    #[test]
    fn fixed_time_eq_compares_whole_values() {
        assert!(fixed_time_eq(b"abc", b"abc"));
        assert!(!fixed_time_eq(b"abc", b"abd"));
        assert!(!fixed_time_eq(b"abc", b"ab"));
        assert!(!fixed_time_eq(b"", b"a"));
    }
}
