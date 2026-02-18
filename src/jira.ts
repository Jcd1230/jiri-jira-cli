declare const Buffer: any;

import { env } from "./env";

export type FieldLookup = { idToName: Record<string, string>; nameToId: Record<string, string> };

class JiraClient {
  private fieldCache: FieldLookup | null = null;
  constructor(private cfg: { user: string; token: string; site: string }) {}

  async request(path: string, init: RequestInit = {}) {
    const url = `${this.cfg.site}${path}`;
    const res = await fetch(url, {
      method: init.method ?? "GET",
      body: init.body,
      headers: {
        Authorization: `Basic ${Buffer.from(`${this.cfg.user}:${this.cfg.token}`).toString("base64")}`,
        Accept: "application/json",
        ...(init.body ? { "Content-Type": "application/json" } : {}),
        ...(init.headers as Record<string, string> | undefined),
      },
    });

    if (!res.ok) {
      const text = await res.text();
      throw new Error(`Jira request failed ${res.status}: ${text}`);
    }

    return res.json();
  }

  projects() {
    return this.request("/rest/api/3/project/search");
  }

  search(jql: string, fields: string[], maxResults = 100, nextPageToken?: string) {
    const body = JSON.stringify({ jql, fields, maxResults, nextPageToken });
    return this.request("/rest/api/3/search/jql", { method: "POST", body });
  }

  async searchAll(jql: string, fields: string[], limit = 1000) {
    const pageSize = 100; // Jira caps pages; cursor-based pagination ignores startAt
    const clampedLimit = Math.max(1, limit);
    let token: string | undefined;
    const issues: any[] = [];
    let moreAvailable = false;

    while (issues.length < clampedLimit) {
      const remaining = clampedLimit - issues.length;
      const page = await this.search(jql, fields, Math.min(pageSize, remaining), token);
      const pageIssues = page.issues ?? [];
      issues.push(...pageIssues);
      token = page.nextPageToken;
      moreAvailable = Boolean(token);
      if (!token || pageIssues.length === 0) break;
    }

    return { issues, moreAvailable };
  }

  async fieldLookup(): Promise<FieldLookup> {
    if (this.fieldCache) return this.fieldCache;
    const data = await this.request("/rest/api/3/field");
    const idToName: Record<string, string> = {};
    const nameToId: Record<string, string> = {};
    for (const f of data as any[]) {
      if (f.id && f.name) {
        idToName[f.id] = f.name;
        nameToId[f.name.toLowerCase()] = f.id;
      }
    }
    this.fieldCache = { idToName, nameToId };
    return this.fieldCache;
  }
}

export const jira = new JiraClient(env);
