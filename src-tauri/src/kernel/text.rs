//! String helpers.

/// Lower-cases `input` and reduces it to a slug safe for filenames and URLs:
/// runs of non-alphanumeric characters collapse to a single `-`, with no
/// leading or trailing `-`, and the result is capped at 60 characters.
///
/// Unicode letters are preserved (e.g. accented Latin characters), so it is
/// lossless for most names — only punctuation, whitespace, and path-unsafe
/// characters are dropped. An input with no alphanumeric characters yields
/// an empty string.
pub fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.extend(ch.to_lowercase());
        } else {
            pending_dash = true;
        }
    }
    let mut slug: String = out.chars().take(60).collect();
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_lowercases_collapses_separators_and_trims() {
        assert_eq!(slugify("Acme Corp."), "acme-corp");
        assert_eq!(slugify("  Hello / World  "), "hello-world");
        assert_eq!(slugify("A&&&B"), "a-b");
        assert_eq!(slugify("***"), "");
        // Accented (French) letters survive — only punctuation is stripped.
        assert_eq!(slugify("Élise Café"), "élise-café");
    }

    #[test]
    fn slugify_caps_length_at_sixty_chars() {
        let long = "a".repeat(100);
        assert_eq!(slugify(&long).chars().count(), 60);
    }
}
