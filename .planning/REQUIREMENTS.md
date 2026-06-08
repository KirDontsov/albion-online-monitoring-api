# Requirements: Albion Online Data API Integration

## Overview
Automatically fetch market prices from the Albion Online Data API and update the local SurrealDB database, while preserving manual edits.

## Core Requirements

### R1: Backend API Proxy Endpoint
- `POST /api/prices/sync` — triggers price sync for all items/artefacts/resources in DB
- `GET /api/prices/status` — returns last sync timestamp and stats
- Fetches from `https://europe.albion-online-data.com/api/v2/stats/prices/{item_ids}.json`
- Batches item IDs to stay under 4096 char URL limit (~200 items per batch)
- Maps API response fields to DB columns:
  - `sell_price_min` → `sell_price_{city}`
  - `buy_price_max` → `buy_price_{city}`
- Only updates fields where `source = 'api'` (preserves manual edits)

### R2: Source Tracking
- Add `source` field to items, artefacts, resources tables: `'api'` | `'manual'`
- When user edits a field via PUT endpoint → set `source = 'manual'`
- When API sync updates a field → set `source = 'api'`
- Default for new/existing records: `source = 'api'`

### R3: Frontend Integration
- Auto-fetch prices on Monitoring/Artefacts/Resources page load (after list loads)
- "Sync Prices" button on each page for manual trigger
- Show last sync time and sync status (loading/error/success)
- Visual indicator on fields: show which are manual vs API-sourced

### R4: Sync Logic
- On sync: fetch all item_ids from DB → batch into groups of ~150 → call API → update DB
- Only update `sell_price_*`, `buy_price_*` where `source = 'api'`
- Always update `updated_at` timestamp
- Return sync stats: items_updated, items_skipped (manual), errors

## Non-functional
- Respect rate limits (180 req/min)
- Handle API errors gracefully (network, rate limit, invalid response)
- Sync should not block other API operations
