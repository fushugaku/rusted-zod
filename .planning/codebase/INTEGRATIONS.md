# External Integrations

**Analysis Date:** 2026-01-24

## APIs & External Services

**Asset Loading:**
- HTTP Fetch API - Load binary map files (.map)
  - Implementation: `src/map/MapLoader.ts` (line 91-99)
  - Purpose: Fetch map data from public directory
  - Error handling: Checks response.ok, throws on 4xx/5xx
  - Tile info loading: `src/map/TileInfoLoader.ts`

**Multiplayer (Planned, Not Implemented):**
- WebSocket via socket.io-client (4.7.2)
  - Status: In package.json but not yet integrated
  - Expected server URL: `ws://localhost:3001` (defined in `src/config/GameConfig.ts`, line 23)
  - Tick rate: 20 ticks/second (configurable in GameConfig.ts)
  - Server implementation: TODO item Phase 3.3 - "Create Node.js server entry point"

## Data Storage

**Databases:**
- Not applicable - Single-player game with no persistent server state
- Client-side only - No ORM or database client used

**File Storage:**
- Local filesystem only - Serves static map files and assets
  - Maps directory: `/maps/` (contains binary .map files)
  - Assets directory: `/assets/` (sprites, textures, sounds)
  - No cloud storage integration

**Client-Side Storage:**
- Not detected - No localStorage, sessionStorage, or IndexedDB usage
- Game state exists only in memory during play session

**Caching:**
- Vite dev server caching (development)
- Browser HTTP caching (production)
- No explicit cache headers configured

## Authentication & Identity

**Auth Provider:**
- None currently implemented
- Multiplayer auth: Planned but not started (Phase 3.3-3.5)

**Game State Authority:**
- Currently: Client-side only (single-player)
- Planned: Server-authoritative architecture (TODO.md Phase 3.4)

## Monitoring & Observability

**Error Tracking:**
- None - No external error tracking service

**Logs:**
- Console logging only
- Examples: `src/map/MapLoader.ts` uses console.log for debug output
- No centralized logging or log aggregation

## CI/CD & Deployment

**Hosting:**
- Not configured - Current setup is development-focused
- Target: Static web hosting (Vite builds to `/dist/`)
- Requires separate server for multiplayer features

**CI Pipeline:**
- Not implemented - No GitHub Actions, GitLab CI, or other pipeline

**Build Process:**
- `npm run build` - TypeScript compilation + Vite bundling
- `npm run dev` - Vite dev server with HMR
- `npm run preview` - Preview production build locally
- `npm run typecheck` - Type checking without emission

## Environment Configuration

**Required env vars:**
- None currently required for client
- DEFAULT_SERVER_URL in `src/config/GameConfig.ts` hardcoded to `ws://localhost:3001`

**Runtime Configuration:**
- Game dimensions: `GAME_WIDTH: 800`, `GAME_HEIGHT: 600` (configurable)
- Network: `DEFAULT_SERVER_URL: ws://localhost:3001`
- Pathfinding costs: Tile cost modifiers for terrain types
- Camera: Scroll speed, zoom ranges, edge scroll zone
- Animation: Frame rates (default: 10, fast: 15, slow: 5)
- Z-depth: Layering constants for rendering order

**Build Artifacts:**
- `client/dist/` - Output directory
- Source maps enabled in production builds

## Webhooks & Callbacks

**Incoming:**
- None - No server endpoints

**Outgoing:**
- Socket.io events (not yet implemented)
- Planned broadcast events: Game state updates, command acknowledgments (Phase 3.4)

## Asset Serving

**Static Files:**
- Public directory configuration: `/client/../` (parent directory)
- Serves: Maps (`.map`), Images/Sprites, Sound assets
- Vite public asset handling with custom filesystem rules

**Map Loading:**
- Fetch from HTTP endpoint (e.g., `/maps/mapname.map`)
- Binary format parsing: `src/map/MapLoader.ts`
  - Header: 62 bytes (width, height, name, counts, type)
  - Objects: 16 bytes each (position, type, team, health)
  - Tiles: 2 bytes each (uint16 tile ID)

---

*Integration audit: 2026-01-24*
