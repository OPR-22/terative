import { create } from "zustand";
import {
  ipc,
  type InvoiceTemplateDto,
  type NewInvoiceTemplateDto,
  type UpdateTemplateDto,
} from "../ipc";

interface TemplateState {
  templates: InvoiceTemplateDto[];
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  create: (input: NewInvoiceTemplateDto) => Promise<InvoiceTemplateDto>;
  update: (input: UpdateTemplateDto) => Promise<InvoiceTemplateDto>;
  remove: (id: string) => Promise<void>;
  duplicate: (id: string) => Promise<InvoiceTemplateDto>;
  setDefault: (id: string) => Promise<void>;
}

export const useTemplateStore = create<TemplateState>((set, get) => ({
  templates: [],
  loading: false,
  error: null,
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const templates = await ipc.templateList();
      set({ templates, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  create: async (input) => {
    const t = await ipc.templateCreate(input);
    await get().refresh();
    return t;
  },
  update: async (input) => {
    const t = await ipc.templateUpdate(input);
    await get().refresh();
    return t;
  },
  remove: async (id) => {
    await ipc.templateDelete(id);
    await get().refresh();
  },
  duplicate: async (id) => {
    const t = await ipc.templateDuplicate(id);
    await get().refresh();
    return t;
  },
  setDefault: async (id) => {
    await ipc.templateSetDefault(id);
    await get().refresh();
  },
}));
