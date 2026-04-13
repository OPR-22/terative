import { invoke } from "@tauri-apps/api/core";
import type {
  Client,
  ListClientsQuery,
  NewClient,
  UpdateClientInput,
} from "../types/client";

export const clientsApi = {
  list: (query: ListClientsQuery = {}) =>
    invoke<Client[]>("client_list", { query }),
  get: (id: string) => invoke<Client>("client_get", { id }),
  create: (input: NewClient) => invoke<Client>("client_create", { input }),
  update: (input: UpdateClientInput) =>
    invoke<Client>("client_update", { input }),
  delete: (id: string) => invoke<void>("client_delete", { id }),
};
