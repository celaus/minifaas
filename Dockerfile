FROM arm32v6/alpine

ENV builddeps="" \
  RUST_LOG="info"\
  MF_ADDR="0.0.0.0:6200" \
  MF_WEB_STATIC_DIR="static" \
  MF_DB_PATH="functions.db" \
  MF_ENV_ROOT_PATH="/minifaas" \
  MF_NO_RUNTIME_THREADS="15" \
  MF_TICK_EVERY_MS="1000" \
  MF_MAX_FUNCTION_RUNTIME_SECS="300" 

WORKDIR /app

RUN mkdir -p /app /minifaas "${MF_WEB_STATIC_DIR}"

RUN apk add --no-cache ${builddeps}
COPY minifaas-web/static/* /app/static

COPY target/arm-unknown-linux-musleabihf/debug/minifaas-web /app/minifaas


EXPOSE 6200

VOLUME /minifaas

CMD [ "./minifaas" ]