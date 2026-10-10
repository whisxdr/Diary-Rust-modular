import { requireUser } from './accounts';
import * as db from './db';
import type { EntryRow } from './types';

export interface SearchHit {
  entry_id: string;
  date: string;
  title: string;
  snippet: string;
  rank: number;
}

const SNIPPET_RADIUS = 60;

/** Reads the search term and paging the API accepts as query parameters. */
export function queryFromParams(params: URLSearchParams) {
  return query(
    params.get('q') ?? '',
    params.has('limit') ? Number(params.get('limit')) : 20,
    params.has('offset') ? Number(params.get('offset')) : 0,
  );
}

/** Substring search over title and body, newest entries first. */
export async function query(term: string, limit = 20, offset = 0) {
  const user = await requireUser();
  const entries = await db.getByIndex<EntryRow>(db.STORES.entries, 'byUser', user.id);

  const needle = term.trim().toLowerCase();
  const matches = needle
    ? entries
        .filter(
          (entry) =>
            entry.title.toLowerCase().includes(needle) ||
            entry.content.toLowerCase().includes(needle),
        )
        .sort((left, right) => right.date.localeCompare(left.date))
    : [];

  return {
    items: matches.slice(offset, offset + limit).map((entry) => ({
      entry_id: entry.id,
      date: entry.date,
      title: entry.title,
      snippet: snippetAround(entry.content, needle),
      rank: rank(entry, needle),
    })),
    total: matches.length,
    limit,
    offset,
  };
}

function snippetAround(content: string, needle: string): string {
  const index = content.toLowerCase().indexOf(needle);
  if (index < 0) {
    return content.slice(0, SNIPPET_RADIUS * 2);
  }

  const start = Math.max(0, index - SNIPPET_RADIUS);
  const end = Math.min(content.length, index + needle.length + SNIPPET_RADIUS);
  const slice = content.slice(start, end);

  // The API marks matches with <mark>, so the results view can highlight them.
  return `${start > 0 ? '…' : ''}${slice.replace(needle, `<mark>${needle}</mark>`)}${
    end < content.length ? '…' : ''
  }`;
}

/** Title matches rank above body matches, matching how search feels elsewhere. */
function rank(entry: EntryRow, needle: string): number {
  const title = entry.title.toLowerCase();
  const body = entry.content.toLowerCase();

  if (title === needle) {
    return 1;
  }
  if (title.includes(needle)) {
    return 0.8;
  }
  return body.split(needle).length > 1 ? 0.5 : 0.2;
}