import { invoke } from "@tauri-apps/api/core";
import type {
  NewService,
  Service,
  UpdateServiceInput,
} from "../types/service";

export const servicesApi = {
  list: (includeInactive = false) =>
    invoke<Service[]>("service_list", { includeInactive }),
  create: (input: NewService) => invoke<Service>("service_create", { input }),
  update: (input: UpdateServiceInput) =>
    invoke<Service>("service_update", { input }),
  delete: (id: string) => invoke<void>("service_delete", { id }),
};
