# Albion Online Market Monitoring API

## Description
Rust/Actix-web API + SurrealDB backend for Albion Online market monitoring. Tracks items, artefacts, and resources with prices across 4 cities (Thetford, Fort Sterling, Martlock, Brecilien). Frontend is Next.js with effector + MUI.

## Tech Stack
- **Backend:** Rust, Actix-web 4, SurrealDB v3 (Docker), reqwest
- **Frontend:** Next.js, TypeScript, effector, MUI, react-hook-form
- **Database:** SurrealDB v3 on localhost:8000
- **API Server:** localhost:8082

## Key Architecture Decisions
- SurrealDB v3 with `protocol-ws` feature, `LazyLock<Surreal<Client>>`
- Items use `item_id` field as business key (not SurrealDB record ID)
- Frontend joins artefacts with items client-side via `artefact_id` field
- All prices stored as strings in 4 city columns + orders columns
- Tables defined on startup via `DEFINE TABLE` SQL

## External References
- Albion Online Data API: https://www.albion-online-data.com/api/
- API host (Europe): https://europe.albion-online-data.com
- Price endpoint: `/api/v2/stats/prices/{item_ids}.json?locations=Thetford,Fort Sterling,Martlock,Brecilien&qualities=1`
- Rate limits: 180 req/min, 300 req/5min
- URL length limit: 4096 chars

## Constraints
- Manual price edits must persist across API refreshes
- API updates should not overwrite user-entered data
- Rate limiting must be respected (batch requests)
