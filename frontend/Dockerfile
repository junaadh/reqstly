# Base stage with dependencies
FROM node:20-alpine AS base
RUN apk add --no-cache libc6-compat
WORKDIR /usr/src/app

# Dependencies stage
FROM base AS deps
COPY package.json package-lock.json* ./
RUN npm ci

# Development stage - for hot reload
FROM base AS development
COPY package.json package-lock.json* ./
RUN npm ci
COPY . .
EXPOSE 5173
CMD ["npm", "run", "dev"]

# Build stage
FROM base AS builder
COPY --from=deps /usr/src/app/node_modules ./node_modules
COPY . .
RUN npm run build

# Production stage
FROM base AS production
ENV NODE_ENV=production
COPY --from=builder /usr/src/app/dist ./dist
COPY package.json ./
RUN npm add -g serve
EXPOSE 5173
CMD ["serve", "-s", "dist", "-l", "5173"]
