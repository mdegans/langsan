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
    let mut ret: Option<String> = None;

    for (i, c) in s.char_indices() {
        if FORBIDDEN_EMOJI.contains(&c)
            || !ENABLED_RANGES
                .iter()
                .any(|range| range.contains(&(c as u32)))
        {
            // Character is not in any of the enabled ranges
            if let Some(ret) = &mut ret {
                ret.push('�');
                continue;
            } else {
                ret = Some(s[..i].to_string() + "�");
                continue;
            }
        }

        if let Some(ret) = &mut ret {
            ret.push(c);
        }
    }

    if let Some(ret) = ret {
        // The string had invalid characters. We need to remove any characters
        // in between the first invalid character and the last invalid
        // character.
        let first_invalid = ret.find('�').unwrap();
        let last_invalid = ret.rfind('�').unwrap();

        if first_invalid != last_invalid {
            let begin = &ret[..first_invalid];
            let end = &ret[last_invalid + 3..];

            #[cfg(feature = "verbose")]
            {
                // 6 because the string "�" is 3 bytes long in UTF-8 and at this
                // point we have already removed the first invalid character.
                // The last invalid character is also removed.
                let n_invalid_bytes = last_invalid - first_invalid + 6;
                return Some(format!(
                    "{}[{} BYTES SANITIZED]{}",
                    begin, n_invalid_bytes, end
                ));
            }
            #[cfg(not(feature = "verbose"))]
            return Some(format!("{}{}", begin, end));
        } else {
            // The string only has one invalid character. In the case of verbose
            // we're already done. In the case of not verbose we need to remove
            // the invalid character.
            #[cfg(feature = "verbose")]
            {
                return Some(ret);
            }
            #[cfg(not(feature = "verbose"))]
            return Some(ret.replace("�", ""));
        }
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
        assert_eq!(sanitize("Ā"), Some("�".to_string()));
        #[cfg(all(not(feature = "latin-1-supplement"), not(feature = "verbose")))]
        assert_eq!(sanitize("Ā"), Some("".to_string()));
        // A hidden sequence of characters with the verbose feature enabled. Use
        // this if it's desirable for the chat agent to see that the user input
        // was sanitized.
        #[cfg(all(not(feature = "tags"), feature = "verbose"))]
        assert_eq!(
            sanitize("https://wuzzi.net/copirate/󠀁󠁔󠁨󠁥󠀠󠁳󠁡󠁬󠁥󠁳󠀠󠁦󠁯󠁲󠀠󠁓󠁥󠁡󠁴󠁴󠁬󠁥󠀠󠁷󠁥󠁲󠁥󠀠󠁕󠁓󠁄󠀠󠀱󠀲󠀰󠀰󠀰󠀰󠁿"),
            Some("https://wuzzi.net/copirate/[120 BYTES SANITIZED]".to_string())
        );
        // A hidden sequence of characters without the verbose feature enabled
        #[cfg(all(not(feature = "tags"), not(feature = "verbose")))]
        assert_eq!(
            sanitize("https://wuzzi.net/copirate/󠀁󠁔󠁨󠁥󠀠󠁳󠁡󠁬󠁥󠁳󠀠󠁦󠁯󠁲󠀠󠁓󠁥󠁡󠁴󠁴󠁬󠁥󠀠󠁷󠁥󠁲󠁥󠀠󠁕󠁓󠁄󠀠󠀱󠀲󠀰󠀰󠀰󠀰󠁿"),
            Some("https://wuzzi.net/copirate/".to_string())
        );
        // Black flag emoji is not enabled
        assert_eq!(sanitize("🏴").unwrap(), "�");
    }
}
