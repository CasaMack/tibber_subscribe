FROM alpine as stripper

RUN apk add binutils
RUN apk --no-cache add ca-certificates

COPY tibber_subscribe /tibber_subscribe
RUN strip /tibber_subscribe

FROM scratch as run

COPY --from=stripper /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=stripper /tibber_subscribe /tibber_subscribe

CMD ["/tibber_subscribe"]
