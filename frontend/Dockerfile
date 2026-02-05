FROM node:20-bullseye AS development

WORKDIR /usr/src/app

# Only copy package files first for install
COPY package*.json ./
RUN npm install

# Copy source into image for dev (optional: mount src only)
COPY . .

# Expose port
EXPOSE 5173

# Run Vite directly from image
CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]
