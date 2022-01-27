FROM projectserum/build:v0.20.1

ENV DEBIAN_FRONTEND=noninteractive

ARG MPL_PROGRAM=candy-machine
ARG MPL_PROGRAM_SO_FILENAME=mpl_candy_machine
ARG CLUSTER=devnet
ARG PAYER_KEY
ARG PROGRAM_ID_KEY=TODO

# Clone MPL
WORKDIR /workdir
RUN git clone https://github.com/metaplex-foundation/metaplex-program-library.git

# Build
WORKDIR /workdir/metaplex-program-library/${MPL_PROGRAM}/program
RUN anchor build

# Deploy
WORKDIR /workdir/metaplex-program-library/target/deploy
RUN touch key.json && echo ${PAYER_KEY} > key.json
RUN solana program deploy ${MPL_PROGRAM_SO_FILENAME}.so -u ${CLUSTER} -k key.json