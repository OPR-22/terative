import { invoke } from "@tauri-apps/api/core";
import type { NewTaxDefinition, TaxDefinition, UpdateTaxInput } from "../types/tax";

export const taxesApi = {
  list: (includeInactive = false) =>
    invoke<TaxDefinition[]>("tax_list", { includeInactive }),
  create: (input: NewTaxDefinition) => invoke<TaxDefinition>("tax_create", { input }),
  update: (input: UpdateTaxInput) => invoke<TaxDefinition>("tax_update", { input }),
  delete: (id: string) => invoke<void>("tax_delete", { id }),
};
