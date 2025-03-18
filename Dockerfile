FROM archlinux:latest

WORKDIR /app

COPY target/debug/choki ./

CMD ["./choki"]
EXPOSE 3000