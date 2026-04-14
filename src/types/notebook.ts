export interface NotebookSection {
  id: string;
  name: string;
  sort_order: number;
}

export interface NotebookEntry {
  section_id: string;
  content: string;
}

export interface ClientNotebookSection {
  section: NotebookSection;
  content: string;
}

export interface ClientNotebookView {
  client_id: string;
  sections: ClientNotebookSection[];
}

export interface RenameNotebookSectionInput {
  id: string;
  name: string;
}

export interface SaveClientNotebookInput {
  client_id: string;
  entries: NotebookEntry[];
}

export interface ClientJournalEntry {
  id: string;
  client_id: string;
  entry_date: string; // YYYY-MM-DD
  content: string;
  created_at: string;
  updated_at: string;
}

export interface NewJournalEntry {
  client_id: string;
  entry_date: string;
  content: string;
}

export interface UpdateJournalEntryInput {
  id: string;
  entry_date: string;
  content: string;
}
