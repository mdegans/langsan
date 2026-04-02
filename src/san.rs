/// Sanitization functions for crate string types.
use crate::ranges::ENABLED_RANGES;

const FORBIDDEN_EMOJI: &[char] = &['🏴'];

/// Invisible and bidirectional control characters that are denied even when
/// their containing Unicode block is enabled. These characters can be used
/// for hidden text attacks (invisible to humans, visible to models).
///
/// Enable the `bidi` feature to allow these characters (needed for RTL
/// language support).
#[cfg(not(feature = "bidi"))]
const FORBIDDEN_BIDI: &[char] = &[
    '\u{200B}', // Zero Width Space
    '\u{200C}', // Zero Width Non-Joiner
    '\u{200D}', // Zero Width Joiner
    '\u{200E}', // Left-to-Right Mark
    '\u{200F}', // Right-to-Left Mark
    '\u{202A}', // Left-to-Right Embedding
    '\u{202B}', // Right-to-Left Embedding
    '\u{202C}', // Pop Directional Formatting
    '\u{202D}', // Left-to-Right Override
    '\u{202E}', // Right-to-Left Override
    '\u{2060}', // Word Joiner
    '\u{2061}', // Function Application (invisible)
    '\u{2062}', // Invisible Times
    '\u{2063}', // Invisible Separator
    '\u{2064}', // Invisible Plus
    '\u{2066}', // Left-to-Right Isolate
    '\u{2067}', // Right-to-Left Isolate
    '\u{2068}', // First Strong Isolate
    '\u{2069}', // Pop Directional Isolate
    '\u{206A}', // Inhibit Symmetric Swapping (deprecated)
    '\u{206B}', // Activate Symmetric Swapping (deprecated)
    '\u{206C}', // Inhibit Arabic Form Shaping (deprecated)
    '\u{206D}', // Activate Arabic Form Shaping (deprecated)
    '\u{206E}', // National Digit Shapes (deprecated)
    '\u{206F}', // Nominal Digit Shapes (deprecated)
];

#[cfg(feature = "bidi")]
const FORBIDDEN_BIDI: &[char] = &[];

/// Return `Some(string)` if the input `&str` has been sanitized, otherwise
/// `None`. Sanitization is performed by removing any characters that are not in
/// the enabled [`RANGES`] and then **removing any charachters in between the
/// first invalid character and the last invalid character**.
//
// This is pretty aggressive, but also very simple. It's not perfect, but it
// should be good enough for most use cases. It attempts to cover all cases
// where there might be hidden or invalid characters in the input. Because all
// possible problematic start and end characters are known, we simply remove
// everything in between them. This is probably more than what is necessary, but
// it follows the principle of least privilege. We only allow what we know is
// safe. The output in the case of verbose is also designed to be as clear as
// possible to the chat agent so they can ask the user for clarification if
// necessary.
pub fn sanitize(s: &str) -> Option<String> {
    let mut first_invalid = None;
    let mut last_invalid = None;

    for (i, c) in s.char_indices() {
        if FORBIDDEN_EMOJI.contains(&c)
            || FORBIDDEN_BIDI.contains(&c)
            || !ENABLED_RANGES
                .iter()
                .any(|range| range.contains(&(c as u32)))
        {
            if let Some(_) = first_invalid {
                last_invalid = Some(i);
                continue;
            } else {
                first_invalid = Some(i);
                last_invalid = Some(i);
                continue;
            }
        }
    }

    if let (Some(first), Some(last)) = (first_invalid, last_invalid) {
        let begin = &s[..first];
        // Last is the next character after the last invalid character
        let last = last + s[last..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        let end = &s[last..];

        let sanitized = if cfg!(feature = "verbose") {
            format!("{}[{} BYTES SANITIZED]{}", begin, last - first, end)
        } else {
            format!("{}{}", begin, end)
        };
        Some(sanitized)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize() {
        // Whitespace and basic latin are enabled by default with the exception
        // of control characters and DEL
        assert_eq!(sanitize("hello \t\n"), None);
        // Latin-1 Supplement is not enabled
        #[cfg(feature = "latin-1-supplement")]
        assert_eq!(sanitize("Ā"), None);
        #[cfg(all(not(feature = "latin-1-supplement"), feature = "verbose"))]
        assert_eq!(sanitize("Ā"), Some("[2 BYTES SANITIZED]".to_string()));
        #[cfg(all(not(feature = "latin-1-supplement"), not(feature = "verbose")))]
        assert_eq!(sanitize("Ā"), Some("".to_string()));
        // A hidden sequence of characters with the verbose feature enabled. Use
        // this if it's desirable for the chat agent to see that the user input
        // was sanitized.
        #[cfg(all(not(feature = "tags"), feature = "verbose"))]
        assert_eq!(
            sanitize("https://wuzzi.net/copirate/󠀁󠁔󠁨󠁥󠀠󠁳󠁡󠁬󠁥󠁳󠀠󠁦󠁯󠁲󠀠󠁓󠁥󠁡󠁴󠁴󠁬󠁥󠀠󠁷󠁥󠁲󠁥󠀠󠁕󠁓󠁄󠀠󠀱󠀲󠀰󠀰󠀰󠀰󠁿"),
            Some("https://wuzzi.net/copirate/[156 BYTES SANITIZED]".to_string())
        );
        // A hidden sequence of characters without the verbose feature enabled
        #[cfg(all(not(feature = "tags"), not(feature = "verbose")))]
        assert_eq!(
            sanitize("https://wuzzi.net/copirate/󠀁󠁔󠁨󠁥󠀠󠁳󠁡󠁬󠁥󠁳󠀠󠁦󠁯󠁲󠀠󠁓󠁥󠁡󠁴󠁴󠁬󠁥󠀠󠁷󠁥󠁲󠁥󠀠󠁕󠁓󠁄󠀠󠀱󠀲󠀰󠀰󠀰󠀰󠁿"),
            Some("https://wuzzi.net/copirate/".to_string())
        );
        // Black flag emoji is not enabled
        #[cfg(not(feature = "verbose"))]
        assert_eq!(sanitize("🏴").unwrap(), "");
        // Sane emoji is not sanitized
        #[cfg(feature = "emoji")]
        assert_eq!(sanitize("👍"), None);
        #[cfg(feature = "emoji")]
        assert_eq!(sanitize("🙏"), None);
    }

    #[test]
    #[cfg(all(not(feature = "bidi"), feature = "general-punctuation"))]
    fn test_bidi_denied_with_general_punctuation() {
        // Em dash should pass (it's in general-punctuation which is a default)
        assert!(
            sanitize("hello \u{2014} world").is_none(),
            "em dash should be allowed"
        );
        // Zero-width space should be stripped even though general-punctuation is enabled
        assert!(
            sanitize("hello\u{200B}world").is_some(),
            "zero-width space should be denied"
        );
        // RTL override should be stripped
        assert!(
            sanitize("hello\u{202E}world").is_some(),
            "RTL override should be denied"
        );
        // LTR mark should be stripped
        assert!(
            sanitize("hello\u{200E}world").is_some(),
            "LTR mark should be denied"
        );
    }
}
