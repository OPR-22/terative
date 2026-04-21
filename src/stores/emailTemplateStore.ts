import { create } from "zustand";
import {
  ipc,
  type EmailTemplateDto,
  type NewEmailTemplateDto,
  type UpdateEmailTemplateDto,
} from "../ipc";

interface EmailTemplateState {
  templates: EmailTemplateDto[];
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  create: (input: NewEmailTemplateDto) => Promise<EmailTemplateDto>;
  update: (input: UpdateEmailTemplateDto) => Promise<EmailTemplateDto>;
  remove: (id: string) => Promise<void>;
  setDefault: (id: string) => Promise<void>;
}

export const useEmailTemplateStore = create<EmailTemplateState>((set, get) => ({
  templates: [],
  loading: false,
  error: null,
  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const templates = await ipc.emailTemplateList();
      set({ templates, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  create: async (input) => {
    const t = await ipc.emailTemplateCreate(input);
    await get().refresh();
    return t;
  },
  update: async (input) => {
    const t = await ipc.emailTemplateUpdate(input);
    await get().refresh();
    return t;
  },
  remove: async (id) => {
    await ipc.emailTemplateDelete(id);
    await get().refresh();
  },
  setDefault: async (id) => {
    await ipc.emailTemplateSetDefault(id);
    await get().refresh();
  },
}));
