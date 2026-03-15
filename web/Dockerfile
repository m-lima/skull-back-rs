FROM docker.io/node:22-bookworm-slim AS web

WORKDIR /web

COPY package.json yarn.lock tsconfig.json tsconfig.node.json vite.config.ts index.html ./
RUN yarn install

COPY src src
COPY public public
RUN yarn build

FROM docker.io/alpine:3.18.4
COPY --from=web /web/dist /web/dist
