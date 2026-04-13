export interface Client {
  id: string;
  name: string;
  email: string | null;
  address: string | null;
  phone: string | null;
  notes: string | null;
  active: boolean;
  created_at: string;
}

export interface NewClient {
  name: string;
  email?: string | null;
  address?: string | null;
  phone?: string | null;
  notes?: string | null;
}

export interface UpdateClientInput {
  id: string;
  name: string;
  email: string | null;
  address: string | null;
  phone: string | null;
  notes: string | null;
}

export interface ListClientsQuery {
  search?: string | null;
  include_inactive?: boolean;
}
