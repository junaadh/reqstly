# Reqstly Frontend

Minimal Vite + React + TypeScript frontend with Docker support.

## Local Development

```bash
# Install dependencies
npm install

# Start dev server (with HMR)
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Docker Compose

### Standalone Frontend Development

```bash
# From frontend directory - start dev server with hot reload
docker-compose -f docker-compose.dev.yml up

# Build and run in detached mode
docker-compose -f docker-compose.dev.yml up -d --build

# View logs
docker-compose -f docker-compose.dev.yml logs -f

# Stop container
docker-compose -f docker-compose.dev.yml down
```

### Full Stack (with Backend, Database, etc.)

```bash
# From project root - start all services
docker-compose -f infra/docker-compose.yml up

# Start specific services
docker-compose -f infra/docker-compose.yml up frontend backend

# Rebuild and start
docker-compose -f infra/docker-compose.yml up -d --build frontend

# View frontend logs
docker-compose -f infra/docker-compose.yml logs -f frontend
```

### Production Build

```bash
# Build production image
docker build -t reqstly-frontend --target production .

# Run production container
docker run -p 5173:5173 reqstly-frontend
```

## Key Docker Fixes

- **`host: '0.0.0.0'`** in vite.config.ts - Allows external access from Docker
- **`usePolling: true`** - Fixes file watching issues in Docker containers
- **Multi-stage build** - Reduces production image size
- **.dockerignore** - Excludes node_modules and build artifacts

## Tech Stack

- **Vite** - Fast build tool with HMR
- **React 18** - UI library
- **TypeScript** - Type safety
- **React Router** - Client-side routing (ready to use)
