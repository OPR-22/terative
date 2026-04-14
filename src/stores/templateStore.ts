import { create } from "zustand";
import { templatesApi } from "../api/templates";
import type {
  InvoiceTemplate,
  NewInvoiceTemplate,
  UpdateTemplateInput,
} from "../types/template";

interface TemplateState {
  templates: InvoiceTemplate[];
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  create: (input: NewInvoiceTemplate) => Promise<InvoiceTemplate>;
  update: (input: UpdateTemplateInput) => Promise<InvoiceTemplate>;
  remove: (id: string) => Promise<void>;
  duplicate: (id: string) => Promise<InvoiceTemplate>;
  setDefault: (id: string) => Promise<void>;
}

export const useTemplateStore = create<TemplateState>((set, get) => ({
  templates: [],
  loading: false,
  error: null,
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const templates = await templatesApi.list();
      set({ templates, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  create: async (input) => {
    const t = await templatesApi.create(input);
    await get().refresh();
    return t;
  },
  update: async (input) => {
    const t = await templatesApi.update(input);
    await get().refresh();
    return t;
  },
  remove: async (id) => {
    await templatesApi.delete(id);
    await get().refresh();
  },
  duplicate: async (id) => {
    const t = await templatesApi.duplicate(id);
    await get().refresh();
    return t;
  },
  setDefault: async (id) => {
    await templatesApi.setDefault(id);
    await get().refresh();
  },
}));
