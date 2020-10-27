FROM ethereum/client-go:v1.9.23

ADD geth-genesis.json ./
ADD entrypoint.sh ./

EXPOSE 8545 8546

ENTRYPOINT ["./entrypoint.sh"]
