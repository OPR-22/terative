export type TemplateLayout = "Classic" | "Modern" | "Minimal";
export type FontChoice = "Serif" | "SansSerif" | "Mono";

export interface InvoiceTemplate {
  id: string;
  name: string;
  base_layout: TemplateLayout;
  logo_image?: number[] | null;
  accent_color: string | null;
  font_family: FontChoice;
  show_seller_phone: boolean;
  show_seller_email: boolean;
  show_registration_id: boolean;
  show_tax_id_numbers: boolean;
  show_signature: boolean;
  show_due_date: boolean;
  show_total_in_words: boolean;
  header_text: string | null;
  footer_text: string | null;
  is_default: boolean;
}

export interface NewInvoiceTemplate {
  name: string;
  base_layout: TemplateLayout;
  logo_image?: number[] | null;
  accent_color: string | null;
  font_family: FontChoice;
  show_seller_phone: boolean;
  show_seller_email: boolean;
  show_registration_id: boolean;
  show_tax_id_numbers: boolean;
  show_signature: boolean;
  show_due_date: boolean;
  show_total_in_words: boolean;
  header_text: string | null;
  footer_text: string | null;
}

export interface UpdateTemplateInput extends NewInvoiceTemplate {
  id: string;
}

export interface TemplateOverride {
  base_layout: TemplateLayout;
  accent_color: string | null;
  font_family: FontChoice;
  logo_image?: number[] | null;
  show_seller_phone: boolean;
  show_seller_email: boolean;
  show_registration_id: boolean;
  show_tax_id_numbers: boolean;
  show_signature: boolean;
  show_due_date: boolean;
  show_total_in_words: boolean;
  header_text: string | null;
  footer_text: string | null;
}

export interface PreviewTemplateInput {
  template_id?: string | null;
  overrides?: TemplateOverride | null;
}

export const defaultNewTemplate = (): NewInvoiceTemplate => ({
  name: "",
  base_layout: "Classic",
  logo_image: null,
  accent_color: null,
  font_family: "SansSerif",
  show_seller_phone: true,
  show_seller_email: true,
  show_registration_id: true,
  show_tax_id_numbers: true,
  show_signature: true,
  show_due_date: true,
  show_total_in_words: true,
  header_text: null,
  footer_text: null,
});
