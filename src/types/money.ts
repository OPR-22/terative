export interface Money {
  amount_cents: number;
  currency: string;
}

export const money = (amountCents: number, currency = "EUR"): Money => ({
  amount_cents: amountCents,
  currency,
});

export const zero = (currency = "EUR"): Money => money(0, currency);
