//! Memory-safe wrapper for secret strings.
//!
//! [`SecretString`] is a newtype around [`zeroize::Zeroizing<String>`]. Three
//! properties matter:
//!
//! 1. The buffer is zero-wiped on drop, so an API token does not linger in
//!    process memory after it falls out of scope.
//! 2. The `Debug` impl prints `"<redacted>"` rather than the contents, so a
//!    stray `tracing::debug!("{cred:?}")` cannot leak the token.
//! 3. There is no [`Clone`], no [`serde::Serialize`], no [`serde::Deserialize`]
//!    — the only way to obtain the underlying bytes is to ask explicitly via
//!    [`SecretString::expose`].

use std::fmt;

use zeroize::Zeroizing;

/// A string that is wiped from memory when dropped and never appears in logs.
///
/// Construct via [`SecretString::new`]; read via [`SecretString::expose`].
/// Intentionally not [`Clone`]: every copy of a secret is another buffer the
/// allocator might leave on the heap, defeating the whole point of the
/// wrapper.
pub struct SecretString(Zeroizing<String>);

impl SecretString {
    /// Wrap the supplied `String`. Ownership is taken so the caller cannot
    /// retain a reference to the un-zeroized buffer.
    pub fn new(s: String) -> Self {
        Self(Zeroizing::new(s))
    }

    /// Borrow the underlying bytes as `&str`. Callers should pass this
    /// directly into a request builder and avoid copying into a `String`,
    /// because any copy is *not* zeroized.
    pub fn expose(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString(<redacted>)")
    }
}

// `Display` is intentionally NOT implemented: there is no safe default for
// "what should printing a secret look like?". Anyone who prints a secret has
// to call `.expose()` themselves, which makes the leak grep-able.

#[cfg(test)]
mod tests {
    use super::SecretString;

    #[test]
    fn debug_is_redacted() {
        let s = SecretString::new("super-secret".to_owned());
        let rendered = format!("{s:?}");
        assert_eq!(rendered, "SecretString(<redacted>)");
        assert!(!rendered.contains("super-secret"));
    }

    #[test]
    fn expose_returns_inner() {
        let s = SecretString::new("abc".to_owned());
        assert_eq!(s.expose(), "abc");
    }
}
