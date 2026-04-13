import type { Money } from "./money";

export interface Service {
  id: string;
  name: string;
  default_price: Money;
  active: boolean;
}

export interface NewService {
  name: string;
  default_price: Money;
}

export interface UpdateServiceInput {
  id: string;
  name: string;
  default_price: Money;
}
