# Technology Stack

**Analysis Date:** 2026-01-24

## Languages

**Primary:**
- TypeScript 5.3.2 - All client-side code and build configuration
- JavaScript (ES2020) - Compiled output target for modern browsers

**Secondary:**
- Binary formats - Zod Engine map file parsing (MapLoader.ts)

## Runtime

**Environment:**
- Node.js >= 18.0.0

**Package Manager:**
- npm (implied by package.json)
- Lockfile: Present (package-lock.json expected)

## Frameworks

**Core:**
- Phaser 3.70.0 - 2D game framework and rendering engine
- Vite 5.0.0 - Build tool and development server

**Pathfinding:**
- easystarjs 0.4.4 - A* pathfinding algorithm for unit movement

**Testing:**
- Not currently implemented - Framework detected in TODO.md as unstarted

**Build/Dev:**
- TypeScript 5.3.2 - Type checking and compilation
- Vite 5.0.0 - ES module bundling, dev server (port 3000)

## Key Dependencies

**Critical:**
- phaser (3.70.0) - Entire game rendering, physics (arcade), scene management, input handling, audio system
- socket.io-client (4.7.2) - WebSocket client for multiplayer networking (not yet integrated per TODO.md)
- easystarjs (0.4.4) - Pathfinding for unit movement AI and waypoint following

**Type Support:**
- @types/easystarjs (0.1.29) - TypeScript definitions for easystarjs
- @types/node (20.10.0) - Node.js type definitions for build tools

## Configuration

**TypeScript Compiler:**
- Target: ES2020
- Module: ESNext
- Strict mode enabled (all strict checks active)
- Path alias: `@/*` resolves to `src/*`
- Base URL: `.` (relative to tsconfig.json location)
- No external library emission, isolated modules mode

**Vite Configuration:**
- Port: 3000 (dev server)
- Source maps: Enabled for production builds
- Public directory: Parent directory (`../`)
- Asset serving: `publicDir` includes parent assets
- Build output: `dist/`
- No public directory copying during build
- Phaser polyfill: `process.env` stub required

**Development Settings:**
- Hot module reloading: Enabled by default
- Auto-open browser: Enabled
- File serving: Allows parent directory access via `fs.allow`

## Platform Requirements

**Development:**
- Node.js 18+
- npm package manager
- Modern browser with ES2020 support
- WebGL or Canvas support (Phaser requirement)

**Production:**
- Browser with ES2020 JavaScript support
- WebGL or Canvas 2D context
- 800x600px minimum recommended (game resolution configurable)
- WebSocket support (for future multiplayer via socket.io-client)

---

*Stack analysis: 2026-01-24*
