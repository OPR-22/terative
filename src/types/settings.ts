export interface SellerProfile {
  name: string;
  title: string | null;
  registration_id: string | null;
  address: string | null;
  phone: string | null;
  email: string | null;
  signature_image?: number[] | null;
}

export interface CurrencyConfig {
  code: string;
  symbol: string;
  symbol_before: boolean;
  main_unit_name: string;
  sub_unit_name: string;
}

export type Theme = "Light" | "Dark";
export type Language = "fr" | "en";

export interface AppPreferences {
  theme: Theme;
  language: Language;
  pdf_output_dir: string;
}

export interface EmailConfig {
  smtp_host: string;
  smtp_port: number;
  sender_address: string;
  subject_template: string;
  body_template: string;
}

export interface SettingsSnapshot {
  seller: SellerProfile;
  currency: CurrencyConfig;
  preferences: AppPreferences;
  email: EmailConfig;
  has_email_password: boolean;
}
