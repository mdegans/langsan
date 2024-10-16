/// Sanitization functions for crate string types.
use crate::ranges::ENABLED_RANGES;

const FORBIDDEN_EMOJI: &[char] = &['🏴'];

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
}
