ARG NODE_VERSION=22.21

### Build server ###
FROM node:${NODE_VERSION}-slim AS build
WORKDIR /app

ENV NODE_ENV=production
ENV PNPM_HOME=/opt/pnpm
ENV PATH=$PNPM_HOME:$PATH
RUN corepack enable

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY clients/node/package.json ./clients/node/package.json
COPY demo/package.json ./demo/package.json
RUN --mount=type=cache,id=pnpm,target=/opt/pnpm \
    pnpm install --frozen-lockfile

COPY demo/src ./demo/src
COPY demo/public ./demo/public
COPY demo/tsconfig.json demo/vite.config.ts ./demo/
COPY clients/node ./clients/node
RUN cd demo && pnpm build


### Run server ###
FROM node:${NODE_VERSION}-slim AS run

# Create non-root user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/home/appuser" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    appuser
USER appuser

# Copy server files
COPY --from=build --chown=appuser /app/demo/.output /app/
COPY --from=build --chown=appuser /app/demo/package.json /app/package.json

# Run server
WORKDIR /app
ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=3000
CMD ["node", "server/index.mjs"]
