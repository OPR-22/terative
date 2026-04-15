use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateId(pub Uuid);

impl TemplateId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TemplateId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TemplateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateLayout {
    Classic,
    Modern,
    Minimal,
}

impl TemplateLayout {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Modern => "Modern",
            Self::Minimal => "Minimal",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Classic" => Some(Self::Classic),
            "Modern" => Some(Self::Modern),
            "Minimal" => Some(Self::Minimal),
            _ => None,
        }
    }
}

impl Default for TemplateLayout {
    fn default() -> Self {
        Self::Classic
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontChoice {
    Serif,
    SansSerif,
    Mono,
}

impl FontChoice {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Serif => "Serif",
            Self::SansSerif => "SansSerif",
            Self::Mono => "Mono",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Serif" => Some(Self::Serif),
            "SansSerif" => Some(Self::SansSerif),
            "Mono" => Some(Self::Mono),
            _ => None,
        }
    }
}

impl Default for FontChoice {
    fn default() -> Self {
        Self::SansSerif
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceTemplate {
    pub id: TemplateId,
    pub name: String,
    pub base_layout: TemplateLayout,
    pub logo_image: Option<Vec<u8>>,
    pub accent_color: Option<String>,
    pub font_family: FontChoice,
    pub show_seller_phone: bool,
    pub show_seller_email: bool,
    pub show_registration_id: bool,
    pub show_tax_id_numbers: bool,
    pub show_signature: bool,
    pub show_due_date: bool,
    pub show_total_in_words: bool,
    pub header_text: Option<String>,
    pub footer_text: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TemplateError {
    #[error("template name cannot be empty")]
    EmptyName,
    #[error("accent color must be a 7-char hex string like #2563EB")]
    InvalidAccentColor,
}

#[derive(Debug, Clone)]
pub struct NewInvoiceTemplate {
    pub name: String,
    pub base_layout: TemplateLayout,
    pub logo_image: Option<Vec<u8>>,
    pub accent_color: Option<String>,
    pub font_family: FontChoice,
    pub show_seller_phone: bool,
    pub show_seller_email: bool,
    pub show_registration_id: bool,
    pub show_tax_id_numbers: bool,
    pub show_signature: bool,
    pub show_due_date: bool,
    pub show_total_in_words: bool,
    pub header_text: Option<String>,
    pub footer_text: Option<String>,
}

impl Default for NewInvoiceTemplate {
    fn default() -> Self {
        Self {
            name: String::new(),
            base_layout: TemplateLayout::default(),
            logo_image: None,
            accent_color: None,
            font_family: FontChoice::default(),
            show_seller_phone: true,
            show_seller_email: true,
            show_registration_id: true,
            show_tax_id_numbers: true,
            show_signature: true,
            show_due_date: true,
            show_total_in_words: true,
            header_text: None,
            footer_text: None,
        }
    }
}

fn validate_color(color: &str) -> Result<(), TemplateError> {
    let bytes = color.as_bytes();
    if bytes.len() != 7 || bytes[0] != b'#' {
        return Err(TemplateError::InvalidAccentColor);
    }
    if !bytes[1..].iter().all(|b| b.is_ascii_hexdigit()) {
        return Err(TemplateError::InvalidAccentColor);
    }
    Ok(())
}

impl InvoiceTemplate {
    pub fn create(input: NewInvoiceTemplate) -> Result<Self, TemplateError> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(TemplateError::EmptyName);
        }
        let accent_color = match input.accent_color.as_deref().map(str::trim) {
            Some("") | None => None,
            Some(c) => {
                validate_color(c)?;
                Some(c.to_string())
            }
        };
        Ok(Self {
            id: TemplateId::new(),
            name,
            base_layout: input.base_layout,
            logo_image: input.logo_image,
            accent_color,
            font_family: input.font_family,
            show_seller_phone: input.show_seller_phone,
            show_seller_email: input.show_seller_email,
            show_registration_id: input.show_registration_id,
            show_tax_id_numbers: input.show_tax_id_numbers,
            show_signature: input.show_signature,
            show_due_date: input.show_due_date,
            show_total_in_words: input.show_total_in_words,
            header_text: input.header_text.and_then(non_empty),
            footer_text: input.footer_text.and_then(non_empty),
            is_default: false,
        })
    }
}

fn non_empty(s: String) -> Option<String> {
    let t = s.trim().to_string();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_template_valid() {
        let t = InvoiceTemplate::create(NewInvoiceTemplate {
            name: "Classic Blue".into(),
            base_layout: TemplateLayout::Classic,
            accent_color: Some("#2563EB".into()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(t.name, "Classic Blue");
        assert_eq!(t.accent_color.as_deref(), Some("#2563EB"));
        assert!(!t.is_default);
    }

    #[test]
    fn create_template_rejects_empty_name() {
        let err = InvoiceTemplate::create(NewInvoiceTemplate::default()).unwrap_err();
        assert_eq!(err, TemplateError::EmptyName);
    }

    #[test]
    fn create_template_rejects_bad_color() {
        let err = InvoiceTemplate::create(NewInvoiceTemplate {
            name: "X".into(),
            accent_color: Some("blue".into()),
            ..Default::default()
        })
        .unwrap_err();
        assert_eq!(err, TemplateError::InvalidAccentColor);
    }

    #[test]
    fn create_template_accepts_no_color() {
        let t = InvoiceTemplate::create(NewInvoiceTemplate {
            name: "X".into(),
            accent_color: None,
            ..Default::default()
        })
        .unwrap();
        assert!(t.accent_color.is_none());
    }

    #[test]
    fn layout_round_trip() {
        for l in [TemplateLayout::Classic, TemplateLayout::Modern, TemplateLayout::Minimal] {
            assert_eq!(TemplateLayout::parse(l.as_str()), Some(l));
        }
    }

    #[test]
    fn font_round_trip() {
        for f in [FontChoice::Serif, FontChoice::SansSerif, FontChoice::Mono] {
            assert_eq!(FontChoice::parse(f.as_str()), Some(f));
        }
    }
}
