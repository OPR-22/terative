import { invoke } from "@tauri-apps/api/core";
import type {
  ClientJournalEntry,
  ClientNotebookView,
  NewJournalEntry,
  NotebookSection,
  RenameNotebookSectionInput,
  SaveClientNotebookInput,
  UpdateJournalEntryInput,
} from "../types/notebook";

export const notebookApi = {
  // sections
  listSections: () => invoke<NotebookSection[]>("notebook_section_list"),
  createSection: (name: string) =>
    invoke<NotebookSection>("notebook_section_create", { name }),
  renameSection: (input: RenameNotebookSectionInput) =>
    invoke<NotebookSection>("notebook_section_rename", { input }),
  deleteSection: (id: string) =>
    invoke<void>("notebook_section_delete", { id }),
  countSectionEntries: (id: string) =>
    invoke<number>("notebook_section_count_entries", { id }),
  reorderSections: (orderedIds: string[]) =>
    invoke<void>("notebook_section_reorder", { orderedIds }),

  // client notebook
  getClientNotebook: (clientId: string) =>
    invoke<ClientNotebookView>("client_notebook_get", { clientId }),
  saveClientNotebook: (input: SaveClientNotebookInput) =>
    invoke<void>("client_notebook_save", { input }),

  // journal
  listJournal: (clientId: string) =>
    invoke<ClientJournalEntry[]>("journal_list_for_client", { clientId }),
  getJournalEntry: (id: string) =>
    invoke<ClientJournalEntry>("journal_entry_get", { id }),
  createJournalEntry: (input: NewJournalEntry) =>
    invoke<ClientJournalEntry>("journal_entry_create", { input }),
  updateJournalEntry: (input: UpdateJournalEntryInput) =>
    invoke<ClientJournalEntry>("journal_entry_update", { input }),
  deleteJournalEntry: (id: string) =>
    invoke<void>("journal_entry_delete", { id }),
};
