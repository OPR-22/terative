import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import {
  CornerDownLeft,
  FileText,
  Package,
  Search,
  Users,
  type LucideIcon,
} from "lucide-react";

import { ipc, type SearchEntityKindDto, type SearchHitDto } from "../../ipc";

/** Order the result groups are rendered in. */
const GROUP_ORDER: SearchEntityKindDto[] = ["Client", "Invoice", "CatalogItem"];

const KIND_ICON: Record<SearchEntityKindDto, LucideIcon> = {
  Client: Users,
  Invoice: FileText,
  CatalogItem: Package,
};

const GROUP_I18N: Record<SearchEntityKindDto, string> = {
  Client: "search.group_client",
  Invoice: "search.group_invoice",
  CatalogItem: "search.group_catalog_item",
};

/** Route a hit opens to when selected. */
function routeFor(hit: SearchHitDto): string {
  switch (hit.kind) {
    case "Client":
      return `/clients/${hit.entity_id}`;
    case "Invoice":
      return `/invoices/${hit.entity_id}/edit`;
    case "CatalogItem":
      return `/catalog/${hit.entity_id}/edit`;
  }
}

/**
 * Folds one character for accent- and case-insensitive matching: strips
 * diacritics and lower-cases it, staying a single unit so a match index
 * still maps back onto the original string.
 */
function foldChar(c: string): string {
  return c.normalize("NFD")[0]?.toLowerCase() ?? c;
}

/** The query split into folded search terms, on the same word boundaries
 *  the backend tokenises on. */
function queryTerms(query: string): string[] {
  return [
    ...new Set(
      query
        .split(/[^\p{L}\p{N}]+/u)
        .filter(Boolean)
        .map((term) => [...term].map(foldChar).join("")),
    ),
  ];
}

/** True when `ch` is a letter or digit — anything else (space, +, -, @, …)
 *  counts as a separator. */
function isContentChar(ch: string): boolean {
  return /[\p{L}\p{N}]/u.test(ch);
}

/**
 * Renders `text` with every run that matches a query `term` wrapped in a
 * bold `<strong>`. Matching is accent- and case-insensitive (so "cafe"
 * bolds "Café") and separator-transparent: a query "4022" highlights a
 * phone shown as "40 22" — the same separator-blind way the backend
 * matched its digits.
 */
function Highlight({ text, terms }: { text: string; terms: string[] }) {
  if (terms.length === 0 || text === "") return <>{text}</>;
  const chars = [...text];
  const folded = chars.map(foldChar);
  const isSep = chars.map((c) => !isContentChar(c));
  const matched = new Array<boolean>(chars.length).fill(false);

  // For each term, scan every start position, matching the term's chars
  // against the text's content chars while skipping over separators. A
  // full match marks the whole run — interior separators included.
  for (const term of terms) {
    for (let start = 0; start < chars.length; start++) {
      if (isSep[start]) continue;
      let ti = 0;
      let i = start;
      while (ti < term.length && i < chars.length) {
        if (isSep[i]) {
          i += 1;
        } else if (folded[i] === term[ti]) {
          ti += 1;
          i += 1;
        } else {
          break;
        }
      }
      if (ti === term.length) {
        for (let k = start; k < i; k++) matched[k] = true;
      }
    }
  }

  const segments: { text: string; match: boolean }[] = [];
  chars.forEach((ch, i) => {
    const last = segments[segments.length - 1];
    if (last && last.match === matched[i]) last.text += ch;
    else segments.push({ text: ch, match: matched[i] });
  });
  return (
    <>
      {segments.map((seg, i) =>
        seg.match ? (
          <strong key={i} className="font-bold text-ink">
            {seg.text}
          </strong>
        ) : (
          <span key={i}>{seg.text}</span>
        ),
      )}
    </>
  );
}

interface SearchPaletteProps {
  onClose: () => void;
}

/**
 * ⌘K global search overlay (T1.07). Full-text search across clients,
 * invoices and catalog items, with keyboard navigation. Mounted only while
 * open, so its query/result state resets on every dismissal.
 */
export function SearchPalette({ onClose }: SearchPaletteProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchHitDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);
  const terms = useMemo(() => queryTerms(query), [query]);

  // Debounced search
  useEffect(() => {
    const q = query.trim();
    if (!q) {
      setResults([]);
      setLoading(false);
      return;
    }
    setLoading(true);
    let cancelled = false;
    const timer = window.setTimeout(() => {
      ipc
        .globalSearch(q)
        .then((hits) => {
          if (cancelled) return;
          setResults(hits);
          setActiveIndex(0);
        })
        .catch(() => {
          if (!cancelled) setResults([]);
        })
        .finally(() => {
          if (!cancelled) setLoading(false);
        });
    }, 160);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [query]);

  // Keep the highlighted row visible as the user arrows through results.
  useEffect(() => {
    listRef.current
      ?.querySelector('[data-active="true"]')
      ?.scrollIntoView({ block: "nearest" });
  }, [activeIndex]);

  const groups = GROUP_ORDER.map((kind) => ({
    kind,
    hits: results.filter((h) => h.kind === kind),
  })).filter((g) => g.hits.length > 0);
  // Flattened in render order — the index space the active highlight and
  // arrow keys operate on.
  const flat = groups.flatMap((g) => g.hits);
  const trimmed = query.trim();

  const open = (hit: SearchHitDto) => {
    navigate(routeFor(hit));
    onClose();
  };

  const onKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setActiveIndex((i) => Math.min(i + 1, Math.max(flat.length - 1, 0)));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActiveIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const hit = flat[activeIndex];
      if (hit) open(hit);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-ink/30 px-4 pt-[12vh]"
      onClick={onClose}
    >
      <div
        role="dialog"
        aria-modal="true"
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-[560px] bg-paper border border-line rounded-card shadow-card flex flex-col overflow-hidden max-h-[68vh]"
      >
        {/* Query field */}
        <div className="flex items-center gap-2.5 px-4 border-b border-line-soft">
          <Search size={16} strokeWidth={1.5} className="text-ink-3 shrink-0" />
          <input
            autoFocus
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={onKeyDown}
            placeholder={t("search.placeholder")}
            className="flex-1 py-3.5 bg-transparent text-[14px] text-ink outline-none placeholder:text-ink-4"
          />
          <kbd className="font-mono text-[10px] px-1.5 py-px bg-paper-2 border border-line-soft text-ink-3 rounded-sm shrink-0">
            esc
          </kbd>
        </div>

        {/* Results */}
        <div ref={listRef} className="overflow-y-auto">
          {trimmed === "" ? (
            <div className="px-4 py-8 text-center text-[12px] text-ink-3">
              {t("search.hint")}
            </div>
          ) : loading && results.length === 0 ? (
            <div className="px-4 py-8 text-center text-[12px] text-ink-3">
              {t("search.loading")}
            </div>
          ) : flat.length === 0 ? (
            <div className="px-4 py-8 text-center text-[12px] text-ink-3">
              {t("search.empty", { query: trimmed })}
            </div>
          ) : (
            groups.map((group) => (
              <div key={group.kind} className="py-1.5">
                <div className="px-4 py-1 text-[10px] font-medium uppercase tracking-[0.06em] text-ink-4">
                  {t(GROUP_I18N[group.kind])}
                </div>
                {group.hits.map((hit) => {
                  const index = flat.indexOf(hit);
                  const isActive = index === activeIndex;
                  const Icon = KIND_ICON[hit.kind];
                  return (
                    <button
                      key={`${hit.kind}-${hit.entity_id}`}
                      type="button"
                      data-active={isActive}
                      onClick={() => open(hit)}
                      onMouseEnter={() => setActiveIndex(index)}
                      className={[
                        "flex items-center gap-3 w-full px-4 py-2 text-left cursor-pointer",
                        isActive ? "bg-paper-3" : "",
                      ].join(" ")}
                    >
                      <Icon
                        size={15}
                        strokeWidth={1.5}
                        className="text-ink-3 shrink-0"
                      />
                      <div className="min-w-0 flex-1">
                        <div className="text-[13px] text-ink truncate">
                          <Highlight text={hit.title} terms={terms} />
                        </div>
                        {hit.snippet ? (
                          <div className="text-[11px] text-ink-3 truncate">
                            <Highlight text={hit.snippet} terms={terms} />
                          </div>
                        ) : null}
                      </div>
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>

        {/* Keyboard hints */}
        {flat.length > 0 ? (
          <div className="flex items-center gap-4 px-4 py-2 border-t border-line-soft bg-paper-2 text-[10px] text-ink-3">
            <span className="flex items-center gap-1.5">
              <kbd className="font-mono px-1.5 py-px bg-paper-3 border border-line-soft rounded-sm">
                ↑↓
              </kbd>
            </span>
            <span className="flex items-center gap-1.5">
              <kbd className="font-mono px-1 py-px bg-paper-3 border border-line-soft rounded-sm">
                <CornerDownLeft size={10} />
              </kbd>
              {t("search.open_hint")}
            </span>
          </div>
        ) : null}
      </div>
    </div>
  );
}
