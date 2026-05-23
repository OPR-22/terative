use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::template_usecases::{
    PreviewTemplateInput, TemplateOverride, UpdateTemplateInput,
};
use crate::domain::template::{
    FontChoice, InvoiceTemplate, NewInvoiceTemplate, TemplateId, TemplateLayout,
};

// ---- TemplateLayoutDto ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum TemplateLayoutDto {
    Classic,
    Modern,
    Minimal,
}

impl From<TemplateLayout> for TemplateLayoutDto {
    fn from(l: TemplateLayout) -> Self {
        match l {
            TemplateLayout::Classic => Self::Classic,
            TemplateLayout::Modern => Self::Modern,
            TemplateLayout::Minimal => Self::Minimal,
        }
    }
}

impl From<TemplateLayoutDto> for TemplateLayout {
    fn from(dto: TemplateLayoutDto) -> Self {
        match dto {
            TemplateLayoutDto::Classic => Self::Classic,
            TemplateLayoutDto::Modern => Self::Modern,
            TemplateLayoutDto::Minimal => Self::Minimal,
        }
    }
}

impl Default for TemplateLayoutDto {
    fn default() -> Self {
        Self::Classic
    }
}

// ---- FontChoiceDto ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum FontChoiceDto {
    Serif,
    SansSerif,
    Mono,
}

impl From<FontChoice> for FontChoiceDto {
    fn from(f: FontChoice) -> Self {
        match f {
            FontChoice::Serif => Self::Serif,
            FontChoice::SansSerif => Self::SansSerif,
            FontChoice::Mono => Self::Mono,
        }
    }
}

impl From<FontChoiceDto> for FontChoice {
    fn from(dto: FontChoiceDto) -> Self {
        match dto {
            FontChoiceDto::Serif => Self::Serif,
            FontChoiceDto::SansSerif => Self::SansSerif,
            FontChoiceDto::Mono => Self::Mono,
        }
    }
}

impl Default for FontChoiceDto {
    fn default() -> Self {
        Self::SansSerif
    }
}

// ---- InvoiceTemplateDto ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct InvoiceTemplateDto {
    pub id: Uuid,
    pub name: String,
    pub base_layout: TemplateLayoutDto,
    pub logo_image: Option<Vec<u8>>,
    pub accent_color: Option<String>,
    pub font_family: FontChoiceDto,
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

impl From<&InvoiceTemplate> for InvoiceTemplateDto {
    fn from(t: &InvoiceTemplate) -> Self {
        Self {
            id: t.id.0,
            name: t.name.clone(),
            base_layout: t.base_layout.into(),
            logo_image: t.logo_image.clone(),
            accent_color: t.accent_color.clone(),
            font_family: t.font_family.into(),
            show_seller_phone: t.show_seller_phone,
            show_seller_email: t.show_seller_email,
            show_registration_id: t.show_registration_id,
            show_tax_id_numbers: t.show_tax_id_numbers,
            show_signature: t.show_signature,
            show_due_date: t.show_due_date,
            show_total_in_words: t.show_total_in_words,
            header_text: t.header_text.clone(),
            footer_text: t.footer_text.clone(),
            is_default: t.is_default,
        }
    }
}

// ---- NewInvoiceTemplateDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewInvoiceTemplateDto {
    pub name: String,
    pub base_layout: TemplateLayoutDto,
    pub logo_image: Option<Vec<u8>>,
    pub accent_color: Option<String>,
    pub font_family: FontChoiceDto,
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

impl From<NewInvoiceTemplateDto> for NewInvoiceTemplate {
    fn from(dto: NewInvoiceTemplateDto) -> Self {
        NewInvoiceTemplate {
            name: dto.name,
            base_layout: dto.base_layout.into(),
            logo_image: dto.logo_image,
            accent_color: dto.accent_color,
            font_family: dto.font_family.into(),
            show_seller_phone: dto.show_seller_phone,
            show_seller_email: dto.show_seller_email,
            show_registration_id: dto.show_registration_id,
            show_tax_id_numbers: dto.show_tax_id_numbers,
            show_signature: dto.show_signature,
            show_due_date: dto.show_due_date,
            show_total_in_words: dto.show_total_in_words,
            header_text: dto.header_text,
            footer_text: dto.footer_text,
        }
    }
}

// ---- UpdateTemplateDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateTemplateDto {
    pub id: Uuid,
    pub name: String,
    pub base_layout: TemplateLayoutDto,
    pub logo_image: Option<Vec<u8>>,
    pub accent_color: Option<String>,
    pub font_family: FontChoiceDto,
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

impl From<UpdateTemplateDto> for UpdateTemplateInput {
    fn from(dto: UpdateTemplateDto) -> Self {
        UpdateTemplateInput {
            id: TemplateId(dto.id),
            name: dto.name,
            base_layout: dto.base_layout.into(),
            logo_image: dto.logo_image,
            accent_color: dto.accent_color,
            font_family: dto.font_family.into(),
            show_seller_phone: dto.show_seller_phone,
            show_seller_email: dto.show_seller_email,
            show_registration_id: dto.show_registration_id,
            show_tax_id_numbers: dto.show_tax_id_numbers,
            show_signature: dto.show_signature,
            show_due_date: dto.show_due_date,
            show_total_in_words: dto.show_total_in_words,
            header_text: dto.header_text,
            footer_text: dto.footer_text,
        }
    }
}

// ---- TemplateOverrideDto ----

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct TemplateOverrideDto {
    pub base_layout: TemplateLayoutDto,
    pub accent_color: Option<String>,
    pub font_family: FontChoiceDto,
    pub logo_image: Option<Vec<u8>>,
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

impl From<TemplateOverrideDto> for TemplateOverride {
    fn from(dto: TemplateOverrideDto) -> Self {
        TemplateOverride {
            base_layout: dto.base_layout.into(),
            accent_color: dto.accent_color,
            font_family: dto.font_family.into(),
            logo_image: dto.logo_image,
            show_seller_phone: dto.show_seller_phone,
            show_seller_email: dto.show_seller_email,
            show_registration_id: dto.show_registration_id,
            show_tax_id_numbers: dto.show_tax_id_numbers,
            show_signature: dto.show_signature,
            show_due_date: dto.show_due_date,
            show_total_in_words: dto.show_total_in_words,
            header_text: dto.header_text,
            footer_text: dto.footer_text,
        }
    }
}

// ---- PreviewTemplateInputDto ----

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct PreviewTemplateInputDto {
    #[serde(default)]
    pub template_id: Option<Uuid>,
    #[serde(default)]
    pub overrides: Option<TemplateOverrideDto>,
}

impl From<PreviewTemplateInputDto> for PreviewTemplateInput {
    fn from(dto: PreviewTemplateInputDto) -> Self {
        PreviewTemplateInput {
            template_id: dto.template_id.map(TemplateId),
            overrides: dto.overrides.map(Into::into),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::template::NewInvoiceTemplate;

    #[test]
    fn layout_round_trips() {
        for layout in [
            TemplateLayout::Classic,
            TemplateLayout::Modern,
            TemplateLayout::Minimal,
        ] {
            let dto: TemplateLayoutDto = layout.into();
            let back: TemplateLayout = dto.into();
            assert_eq!(back, layout);
        }
    }

    #[test]
    fn font_round_trips() {
        for font in [FontChoice::Serif, FontChoice::SansSerif, FontChoice::Mono] {
            let dto: FontChoiceDto = font.into();
            let back: FontChoice = dto.into();
            assert_eq!(back, font);
        }
    }

    #[test]
    fn invoice_template_to_dto_preserves_all_fields() {
        let mut domain = InvoiceTemplate::create(NewInvoiceTemplate {
            name: "Classic Blue".into(),
            base_layout: TemplateLayout::Classic,
            accent_color: Some("#2563EB".into()),
            font_family: FontChoice::SansSerif,
            ..Default::default()
        })
        .unwrap();
        domain.is_default = true;
        let dto: InvoiceTemplateDto = (&domain).into();
        assert_eq!(dto.id, domain.id.0);
        assert_eq!(dto.name, "Classic Blue");
        assert!(matches!(dto.base_layout, TemplateLayoutDto::Classic));
        assert_eq!(dto.accent_color.as_deref(), Some("#2563EB"));
        assert!(dto.is_default);
    }

    #[test]
    fn new_template_dto_round_trip_via_input() {
        let dto = NewInvoiceTemplateDto {
            name: "X".into(),
            base_layout: TemplateLayoutDto::Modern,
            logo_image: None,
            accent_color: None,
            font_family: FontChoiceDto::Serif,
            show_seller_phone: true,
            show_seller_email: true,
            show_registration_id: true,
            show_tax_id_numbers: true,
            show_signature: true,
            show_due_date: true,
            show_total_in_words: true,
            header_text: None,
            footer_text: None,
        };
        let input: NewInvoiceTemplate = dto.into();
        assert_eq!(input.name, "X");
        assert_eq!(input.base_layout, TemplateLayout::Modern);
        assert_eq!(input.font_family, FontChoice::Serif);
    }
}
