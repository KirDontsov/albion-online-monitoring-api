# Roadmap: Albion Online Data API Integration

## Phase 1: Backend — Source Tracking & DB Schema
**Goal:** Add `source` field to all tables, update CRUD endpoints to set source on manual edits.

### Tasks
1. Add `source: String` field to `Item`, `Artefact`, `Resource` structs in Rust
2. Update `UPDATE` SQL queries in `items/update`, `artefacts/update`, `resources/update` to set `source = 'manual'`
3. Update `CREATE` SQL to default `source = 'api'`
4. Add migration: `UPDATE items SET source = 'api' WHERE source = ''`
5. Verify all CRUD operations work with new field

### UAT
- Create item → source = 'api'
- Update item → source = 'manual'
- List items → all have source field
- Old records get source = 'api' after migration

---

## Phase 2: Backend — Price Sync Endpoint
**Goal:** Create `/api/prices/sync` that fetches from Albion API and updates DB.

### Tasks
1. Create `src/prices/mod.rs` and `src/prices/prices.rs`
2. Implement `POST /api/prices/sync` handler:
   - Fetch all item_ids from DB (items + artefacts + resources)
   - Batch into groups of ~150 items
   - For each batch: `GET https://europe.albion-online-data.com/api/v2/stats/prices/{ids}.json?locations=Thetford,Fort Sterling,Martlock,Brecilien&qualities=1`
   - Parse response, map city names to column names
   - Update DB: only fields where `source = 'api'`
   - Return sync stats
3. Implement `GET /api/prices/status` handler (last sync time)
4. Register routes in `main.rs`
5. Add `reqwest` error handling and rate limit awareness

### UAT
- `POST /api/prices/sync` returns JSON with `items_updated`, `items_skipped`, `errors`
- DB prices updated for API-sourced fields
- Manual fields unchanged
- Rate limit respected (batching)

---

## Phase 3: Frontend — Sync Button & Auto-refresh
**Goal:** Add sync trigger and auto-refresh on page load.

### Tasks
1. Create `shared/api/prices/syncPrices.ts` — `POST /api/prices/sync`
2. Create `shared/api/prices/getSyncStatus.ts` — `GET /api/prices/status`
3. Create `entities/prices/model.ts`:
   - `$syncStatus` store (last sync time, loading, error)
   - `syncPricesFx` effect
   - `SyncPricesGate` for auto-trigger
4. Add "Обновить цены" button to Monitoring, Artefacts, Resources pages
5. Auto-trigger sync on page load (after list loads)
6. Show last sync time near the button
7. Add visual indicator for manual vs API-sourced fields (optional: small icon/color)

### UAT
- Page load → sync runs automatically → prices updated
- Click "Обновить цены" → sync runs → UI refreshes
- Last sync time displayed
- Manual edits preserved after sync
- Error state shown if sync fails

---

## Phase 4: Polish & Edge Cases
**Goal:** Handle edge cases and improve UX.

### Tasks
1. Rate limit handling: if 429 response, retry with backoff
2. Handle empty/zero prices from API (don't overwrite with 0)
3. Add loading skeleton during sync
4. Log sync history (optional: store in DB)
5. Handle new items added to DB (they need source='api' by default)

### UAT
- Rate limit → automatic retry
- API returns 0 → field unchanged
- New item created → gets prices on next sync
- Sync history visible (optional)
