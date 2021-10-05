FROM elixir:1.12.3 as build

WORKDIR /src
COPY src .

ARG MIX_ENV
ARG RELEASE_ENV

ENV MIX_ENV=${MIX_ENV}
ENV RELEASE_ENV=${RELEASE_ENV}

RUN mix local.hex --force
RUN mix local.rebar --force
RUN mix deps.get
RUN mix do compile, release 

FROM debian:buster-slim as final
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    apt-utils \
    locales \
    openssl \
    curl \
    wget && \
    rm -rf /var/lib/apt/lists/

# Set the locale
RUN sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen && \
    locale-gen
ENV LANG en_US.UTF-8  
ENV LANGUAGE en_US:en  
ENV LC_ALL en_US.UTF-8

ARG MIX_ENV
ARG RELEASE_ENV
ARG PORT
ENV MIX_ENV=${MIX_ENV}
ENV RELEASE_ENV=${RELEASE_ENV}
ENV PORT=${PORT}
WORKDIR /app
COPY --from=build /src/_build/${MIX_ENV}/rel/mora ./
EXPOSE ${PORT}

CMD ["bin/mora", "start"]
