FROM docker.io/node:21.2.0-bookworm-slim as web

WORKDIR web

RUN apt update && apt install -y curl zip

COPY package.json yarn.lock tsconfig.json ./
RUN yarn install

COPY src src
COPY public public

RUN curl https://use.fontawesome.com/releases/v5.15.4/fontawesome-free-5.15.4-web.zip -o /fontawesome.zip && \
    unzip /fontawesome.zip -d /font

RUN cp /web/src/util.prod.ts /web/src/util.ts && \
    mkdir -p /web/public/font/css && \
    mv /font/fontawesome-free-5.15.4-web/css/all.min.css /web/public/font/css/. && \
    mv /font/fontawesome-free-5.15.4-web/webfonts /web/public/font/. && \
    mv /web/public/index.prod.html /web/public/index.html

RUN yarn build

FROM docker.io/alpine:3.18.4
COPY --from=web /web/build /web/build
