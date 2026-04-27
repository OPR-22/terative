// Hardcoded bookmarks for the MVP. Replace with a DB-backed store when
// the feature graduates to "manage your own bookmarks".
export interface Bookmark {
  id: string;
  label: string;
  url: string;
}

export const BOOKMARKS: Bookmark[] = [
  { id: "google", label: "Google", url: "https://google.com" },
  { id: "example", label: "Example", url: "https://example.com" },
];

export const BOOKMARKS_BY_ID: Record<string, Bookmark> = Object.fromEntries(
  BOOKMARKS.map((b) => [b.id, b]),
);
