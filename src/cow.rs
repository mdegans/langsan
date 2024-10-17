use std::{borrow::Cow, ops::Deref};

use crate::san::sanitize;

/// A wrapper around `Cow<str>` that [`sanitize`]s the string when it is
/// created. The string is only copied if it's necessary.
///
/// This is guaranteed to be a valid UTF-8 string with only the characters that
/// are enabled by feature.
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CowStr<'a> {
    pub(crate) inner: Cow<'a, str>,
}

#[cfg(feature = "serde")]
impl<'de, 'a> serde::Deserialize<'de> for CowStr<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let cow: Cow<'a, str> = Cow::deserialize(deserializer)?;
        Ok(cow.into())
    }
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

    /// Appends a string slice to the end of this `CowStr`. The string slice is
    /// sanitized before being appended. This will take ownership of the string
    /// if it's not already owned.
    pub fn push_str(&mut self, s: &str) {
        if let Some(sanitized) = sanitize(s) {
            if !sanitized.is_empty() {
                self.inner.to_mut().push_str(&sanitized);
            }
        } else {
            if !s.is_empty() {
                self.inner.to_mut().push_str(s);
            }
        }
    }

    pub fn is_owned(&self) -> bool {
        matches!(self.inner, Cow::Owned(_))
    }

    pub fn is_borrowed(&self) -> bool {
        matches!(self.inner, Cow::Borrowed(_))
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn into_inner(self) -> Cow<'a, str> {
        self.inner
    }
}

impl<'a> Into<Cow<'a, str>> for CowStr<'a> {
    fn into(self) -> Cow<'a, str> {
        self.into_inner()
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
    #[cfg(not(feature = "emoticons-emoji"))]
    fn test_cowstr() {
        let s = CowStr::from("Hello, world! That's all folks!");
        assert!(s.is_borrowed());
        assert!(!s.is_empty());
        assert_eq!(s.as_ref(), "Hello, world! That's all folks!");

        let s = CowStr::from("Hello, \u{1F600}world!");
        assert!(s.is_owned()); // because of the emoji
        #[cfg(feature = "verbose")]
        assert_eq!(s.as_ref(), "Hello, [4 BYTES SANITIZED]world!");
        #[cfg(not(feature = "verbose"))]
        assert_eq!(s.as_ref(), "Hello, world!");

        let s = CowStr::new("Hello, \u{1F600}world!");
        assert!(s.is_owned());
        #[cfg(feature = "verbose")]
        assert_eq!(s.as_ref(), "Hello, [4 BYTES SANITIZED]world!");
        #[cfg(not(feature = "verbose"))]
        assert_eq!(s.as_ref(), "Hello, world!");

        // for coverage
        let s: CowStr<'static> = s.into_static();
        let s: String = s.into_owned();
        let s = CowStr::from(s);
        #[cfg(not(feature = "verbose"))]
        assert_eq!(s.deref(), "Hello, world!");
        #[cfg(not(feature = "verbose"))]
        assert_eq!(s.as_ref(), "Hello, world!");
        #[cfg(not(feature = "verbose"))]
        assert_eq!(s.to_string(), "Hello, world!");

        assert_eq!("\u{1F600}\u{1F600}\u{1F600}".bytes().len(), 12);

        let s = CowStr::from("Hello, \u{1F600}\u{1F600}\u{1F600}world!".to_string());
        #[cfg(not(feature = "verbose"))]
        assert_eq!(s.as_ref(), "Hello, world!");
        #[cfg(feature = "verbose")]
        assert_eq!(s.as_ref(), "Hello, [12 BYTES SANITIZED]world!");
    }

    #[cfg(feature = "serde")]
    #[cfg(all(not(feature = "emoticons-emoji"), not(feature = "verbose")))]
    #[test]
    fn test_serde() {
        let s = CowStr::from("Hello, world!\u{1F600}");
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, r#""Hello, world!""#);

        let s: CowStr = serde_json::from_str(&json).unwrap();
        assert_eq!(s.as_ref(), "Hello, world!");

        // Test inside a struct
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Test<'a> {
            s: CowStr<'a>,
        }

        let t = Test {
            s: CowStr::from("Hello, world!\u{1F600}"),
        };
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, r#"{"s":"Hello, world!"}"#);
    }

    #[test]
    #[cfg(not(feature = "emoticons-emoji"))]
    fn test_push_str() {
        let mut s = CowStr::from("Hello, world!");
        s.push_str(" That's all folks!\u{1F600}");
        #[cfg(not(feature = "verbose"))]
        assert_eq!(s.as_ref(), "Hello, world! That's all folks!");
        #[cfg(feature = "verbose")]
        assert_eq!(
            s.as_ref(),
            "Hello, world! That's all folks![4 BYTES SANITIZED]"
        );

        let mut s = CowStr::from("Hello, \u{1F600}world!");
        s.push_str(" That's all folks!\u{1F600}");
        #[cfg(all(not(feature = "emoticons-emoji"), feature = "verbose"))]
        assert_eq!(
            s.as_ref(),
            "Hello, [4 BYTES SANITIZED]world! That's all folks![4 BYTES SANITIZED]"
        );
        #[cfg(all(not(feature = "emoticons-emoji"), not(feature = "verbose")))]
        assert_eq!(s.as_ref(), "Hello, world! That's all folks!");

        assert_eq!("\u{1F600}\u{1F600}\u{1F600}".bytes().len(), 12);

        let mut s = CowStr::from("Hello, \u{1F600}\u{1F600}\u{1F600}world!".to_string());
        s.push_str(" That's all folks!");

        #[cfg(not(feature = "verbose"))]
        assert_eq!(s.as_ref(), "Hello, world! That's all folks!");
        #[cfg(feature = "verbose")]
        assert_eq!(
            s.as_ref(),
            "Hello, [12 BYTES SANITIZED]world! That's all folks!"
        );
    }
}
