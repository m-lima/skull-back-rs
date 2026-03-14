FROM docker.io/node:21.2.0-bookworm-slim AS web
WORKDIR /web

COPY package.json yarn.lock .
RUN yarn

COPY . .
RUN yarn build

FROM docker.io/alpine:3.18.4
COPY --from=web /web/dist /web/dist
