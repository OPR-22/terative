export interface TaxDefinition {
  id: string;
  name: string;
  percentage: string; // rust_decimal serializes as string
  tax_id_number: string | null;
  active: boolean;
}

export interface NewTaxDefinition {
  name: string;
  percentage: string;
  tax_id_number?: string | null;
}

export interface UpdateTaxInput {
  id: string;
  name: string;
  percentage: string;
  tax_id_number: string | null;
}
