import { invoke } from "@tauri-apps/api/core";
import type {
  InvoiceTemplate,
  NewInvoiceTemplate,
  PreviewTemplateInput,
  UpdateTemplateInput,
} from "../types/template";

export const templatesApi = {
  list: () => invoke<InvoiceTemplate[]>("template_list"),
  create: (input: NewInvoiceTemplate) =>
    invoke<InvoiceTemplate>("template_create", { input }),
  update: (input: UpdateTemplateInput) =>
    invoke<InvoiceTemplate>("template_update", { input }),
  delete: (id: string) => invoke<void>("template_delete", { id }),
  duplicate: (id: string) => invoke<InvoiceTemplate>("template_duplicate", { id }),
  setDefault: (id: string) => invoke<void>("template_set_default", { id }),
  preview: async (input: PreviewTemplateInput): Promise<Uint8Array> => {
    const bytes = await invoke<number[]>("template_preview", { input });
    return new Uint8Array(bytes);
  },
};
