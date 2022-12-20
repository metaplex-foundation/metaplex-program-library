import { Metadata, Metaplex, Nft } from "@metaplex-foundation/js";
import { PublicKey } from "@solana/web3.js";
import * as fs from "fs";

const axios = require('axios')

const URL_BASE = `https://api.helius.xyz/v1/mintlist?api-key=`;

export class TraitLocation {
    path: String | null;
    mintAddress: String | null;

    constructor(path: String | null, mintAddress: String | null) {
        this.path = path;
        this.mintAddress = mintAddress;
    }
}

export class Trait {
    map: Map<String, TraitLocation>;

    constructor() {
        this.map = new Map();
    }

    public add(trait: String) {
        if (!this.map.has(trait)) {
            // console.log("Adding trait: " + trait)
            this.map.set(trait, new TraitLocation(null, null));
        }
        // console.log(JSON.stringify(this.map));
    }
}

export class TraitManifest {
    map: Map<String, Trait>;

    constructor() {
        this.map = new Map();
    }

    public add(field: String, value: String) {
        if (!this.map.has(field)) {
            let newTrait = new Trait();
            newTrait.add(value);
            // console.log("Adding field: " + field);
            this.map.set(field, newTrait);
        }
        else {
            // console.log("Modding field: " + field);
            this.map.get(field)?.add(value);
        }
        // console.log(JSON.stringify(this.map));
    }
}

const getMintCache = (collectionId: String) => {
    const filename = ".mintlist." + collectionId + ".json";

    if (fs.existsSync(filename)) {
        const data = fs.readFileSync(filename);
        return JSON.parse(data.toString());
    }

    return null;
}

const setMintCache = (collectionId: String, nftList: Nft[]) => {
    const filename = ".mintlist." + collectionId + ".json";
    fs.writeFileSync(filename, JSON.stringify(nftList, null, 2));
}

export const getMintlist = async (metaplex: Metaplex, collectionId: String) => {
    let mintlist = getMintCache(collectionId);
    if (!mintlist) {
        const url = URL_BASE + process.env.HELIUS_API_KEY;
        console.log(url);
        const { data } = await axios.post(url, {
            "query": {
                "verifiedCollectionAddresses": [collectionId]
            },
            "options": {
                "limit": 10000
            }
        });
        // console.log("Mintlist: ", data.result);
        // console.log(data.result[0].mint);
        let mintlist = data.result.map((data: any) => new PublicKey(data.mint));
        let metadatas = await metaplex.nfts().findAllByMintList({ mints: mintlist });
        // console.log("Metadatas: ", metadatas);
        let count = 0;
        let nftList = await Promise.all(metadatas.map((metadata) => {
            return metaplex.nfts().load({ metadata: metadata as Metadata }) as Promise<Nft>
        }));
        
        setMintCache(collectionId, nftList);
        return nftList;
    }
    return mintlist;
};

export const getTraitManifest = async (nftList: Nft[]) => {
    let traitManifest = new TraitManifest();
    for (const nft of nftList) {
        if (nft && nft.json && nft.json.attributes) {
            for (const attribute of nft.json.attributes) {
                if (attribute.trait_type && attribute.value) {
                    // console.log("Adding " + attribute.trait_type + " : " + attribute.value);
                    traitManifest.add(attribute.trait_type, attribute.value);
                }
            }
        }
    }
    console.log(JSON.stringify(traitManifest, replacer, 2));
}

function replacer(key, value) {
    if (value instanceof Map) {
        return {
            dataType: 'Map',
            value: Array.from(value.entries()), // or with spread: value: [...value]
        };
    } else {
        return value;
    }
}

function reviver(key, value) {
    if (typeof value === 'object' && value !== null) {
        if (value.dataType === 'Map') {
            return new Map(value.value);
        }
    }
    return value;
}