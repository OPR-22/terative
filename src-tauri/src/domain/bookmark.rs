use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BookmarkId(pub Uuid);

impl BookmarkId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for BookmarkId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BookmarkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bookmark {
    pub id: BookmarkId,
    pub label: String,
    pub url: String,
    pub sort_order: i32,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BookmarkError {
    #[error("bookmark label cannot be empty")]
    EmptyLabel,
    #[error("bookmark url cannot be empty")]
    EmptyUrl,
    #[error("bookmark url is not a valid url")]
    InvalidUrl,
    #[error("bookmark url scheme must be http or https")]
    UnsupportedScheme,
}

fn validate_label(s: &str) -> Result<String, BookmarkError> {
    let trimmed = s.trim().to_string();
    if trimmed.is_empty() {
        return Err(BookmarkError::EmptyLabel);
    }
    Ok(trimmed)
}

fn validate_url(s: &str) -> Result<String, BookmarkError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(BookmarkError::EmptyUrl);
    }
    let parsed = url::Url::parse(trimmed).map_err(|_| BookmarkError::InvalidUrl)?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err(BookmarkError::UnsupportedScheme),
    }
    Ok(trimmed.to_string())
}

impl Bookmark {
    pub fn create(label: String, url: String, sort_order: i32) -> Result<Self, BookmarkError> {
        let label = validate_label(&label)?;
        let url = validate_url(&url)?;
        Ok(Self {
            id: BookmarkId::new(),
            label,
            url,
            sort_order,
        })
    }

    pub fn rename(&mut self, label: String) -> Result<(), BookmarkError> {
        self.label = validate_label(&label)?;
        Ok(())
    }

    pub fn relocate(&mut self, url: String) -> Result<(), BookmarkError> {
        self.url = validate_url(&url)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_trims_label() {
        let b = Bookmark::create("  Google  ".into(), "https://google.com".into(), 0).unwrap();
        assert_eq!(b.label, "Google");
    }

    #[test]
    fn create_rejects_empty_label() {
        let err = Bookmark::create("   ".into(), "https://google.com".into(), 0).unwrap_err();
        assert_eq!(err, BookmarkError::EmptyLabel);
    }

    #[test]
    fn create_rejects_empty_url() {
        let err = Bookmark::create("Google".into(), "  ".into(), 0).unwrap_err();
        assert_eq!(err, BookmarkError::EmptyUrl);
    }

    #[test]
    fn create_rejects_unparseable_url() {
        let err = Bookmark::create("X".into(), "not a url".into(), 0).unwrap_err();
        assert_eq!(err, BookmarkError::InvalidUrl);
    }

    #[test]
    fn create_rejects_non_http_scheme() {
        let err =
            Bookmark::create("Local".into(), "file:///etc/passwd".into(), 0).unwrap_err();
        assert_eq!(err, BookmarkError::UnsupportedScheme);
    }

    #[test]
    fn create_accepts_http_and_https() {
        assert!(Bookmark::create("X".into(), "http://example.com".into(), 0).is_ok());
        assert!(Bookmark::create("X".into(), "https://example.com".into(), 0).is_ok());
    }

    #[test]
    fn rename_validates() {
        let mut b = Bookmark::create("X".into(), "https://x.com".into(), 0).unwrap();
        b.rename("  New  ".into()).unwrap();
        assert_eq!(b.label, "New");
        assert_eq!(b.rename("".into()), Err(BookmarkError::EmptyLabel));
    }

    #[test]
    fn relocate_validates() {
        let mut b = Bookmark::create("X".into(), "https://x.com".into(), 0).unwrap();
        b.relocate("https://y.com".into()).unwrap();
        assert_eq!(b.url, "https://y.com");
        assert_eq!(
            b.relocate("ftp://oops".into()),
            Err(BookmarkError::UnsupportedScheme),
        );
    }
}
