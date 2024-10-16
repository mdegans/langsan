use std::{borrow::Cow, ops::Deref};

use crate::san::sanitize;

/// A wrapper around `Cow<str>` that [`sanitize`]s the string when it is
/// created. The string is only copied if it's necessary.
///
/// This is guaranteed to be a valid UTF-8 string with only the characters that
/// are enabled by feature.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CowStr<'a> {
    pub(crate) inner: Cow<'a, str>,
}

impl<'a> CowStr<'a> {
    pub fn new(s: impl Into<Cow<'a, str>>) -> Self {
        let inner: Cow<'a, str> = s.into();
        inner.into()
    }

    /// Converts the `CowStr` into a `CowStr` with a `'static` lifetime. This
    /// will copy the string if it's not already owned.
    pub fn into_static(self) -> CowStr<'static> {
        CowStr {
            inner: self.into_owned().into(),
        }
    }

    pub fn into_owned(self) -> String {
        self.inner.into_owned()
    }
}

impl<'a> From<Cow<'a, str>> for CowStr<'a> {
    fn from(cow: Cow<'a, str>) -> Self {
        if let Some(sanitized) = sanitize(cow.as_ref()) {
            CowStr {
                inner: sanitized.into(),
            }
        } else {
            CowStr { inner: cow }
        }
    }
}

impl<'a> From<&'a str> for CowStr<'a> {
    fn from(s: &'a str) -> Self {
        Cow::Borrowed(s).into()
    }
}

impl<'a> From<String> for CowStr<'a> {
    fn from(s: String) -> Self {
        let cow: Cow<'a, str> = Cow::Owned(s);
        cow.into()
    }
}

impl<'a> AsRef<str> for CowStr<'a> {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}

impl<'a> Deref for CowStr<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.inner.deref()
    }
}

impl std::fmt::Display for CowStr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cowstr() {
        let s = CowStr::from("Hello, world! That's all folks!");
        assert_eq!(s.as_ref(), "Hello, world! That's all folks!");

        let s = CowStr::from("Hello, \u{1F600}world!");
        #[cfg(all(not(feature = "emoticons-emoji"), feature = "verbose"))]
        assert_eq!(s.as_ref(), "Hello, �world!");
        #[cfg(all(not(feature = "emoticons-emoji"), not(feature = "verbose")))]
        assert_eq!(s.as_ref(), "Hello, world!");

        #[cfg(not(feature = "emoticons-emoji"))]
        {
            assert_eq!("\u{1F600}\u{1F600}\u{1F600}".bytes().len(), 12);

            let s = CowStr::from("Hello, \u{1F600}\u{1F600}\u{1F600}world!".to_string());
            #[cfg(not(feature = "verbose"))]
            assert_eq!(s.as_ref(), "Hello, world!");
            #[cfg(feature = "verbose")]
            assert_eq!(s.as_ref(), "Hello, [12 BYTES SANITIZED]world!");
        }
    }
}
