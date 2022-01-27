FROM projectserum/build:v0.20.1

ENV DEBIAN_FRONTEND=noninteractive

ARG CLUSTER=devnet
ARG PAYER_KEY
ARG PROGRAM_ID_KEY=TEMP_ID

# Clone MPL
WORKDIR /mpl-workspace
RUN git clone https://github.com/metaplex-foundation/metaplex-program-library.git

# Build
WORKDIR /mpl-workspace/metaplex-program-library/candy-machine/program
RUN anchor build

# Deploy
WORKDIR /mpl-workspace/metaplex-program-library/target/deploy
RUN touch payer_key.json && echo ${PAYER_KEY} > payer_key.json
RUN touch program_id.json && echo ${PROGRAM_ID_KEY} > program_id.json
RUN solana program deploy nft_candy_machine.so -u ${CLUSTER} -k payer_key.json