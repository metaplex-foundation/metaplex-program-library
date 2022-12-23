import { Metadata, Metaplex, Nft } from "@metaplex-foundation/js";
import { PublicKey } from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";

const axios = require('axios')

const URL_BASE = `https://api.helius.xyz/v1/mintlist?api-key=`;

export class TraitLocation {
    path: string | null;
    mintAddress: string | null;

    constructor(path: string | null, mintAddress: string | null) {
        this.path = path;
        this.mintAddress = mintAddress;
    }
}

export class Trait {
    map: Map<string, TraitLocation>;

    constructor() {
        this.map = new Map();
    }

    public static from(json: any) {
        return Object.assign(new Trait(), json);
    }

    public add = (trait: string) => {
        if (!this.map.has(trait)) {
            // console.log("Adding trait: " + trait)
            this.map.set(trait, new TraitLocation(null, null));
        }
        // console.log(JSON.stringify(this.map));
    }

    public updatePath = (trait: string, path: string) => {
        if (this.map.has(trait)) {
            let traitLocation = this.map.get(trait);
            if (traitLocation) {
                traitLocation.path = path;
                this.map.set(trait, traitLocation);
            }
        }
    }
}

export class TraitManifest {
    map: Map<string, Trait>;

    constructor() {
        this.map = new Map();
    }

    public static from(json: any) {
        let traitManifest = new TraitManifest();
        Object.assign(traitManifest, json);
        for (const [key, value] of traitManifest.map.entries()) {
            traitManifest.map.set(key, Trait.from(value));
        }
        return traitManifest;
    }

    public add = (field: string, value: string) => {
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

    public updatePath = (field: string, value: string, path: string) => {
        if (this.map.has(field)) {
            let trait = this.map.get(field);
            if (trait?.map.has(value)) {
                trait.updatePath(value, path);
                this.map.set(field, trait);
            }
        }
    }
}

const getMintCache = (address: string) => {
    const filename = ".mintlist." + address + ".json";

    if (fs.existsSync(filename)) {
        const data = fs.readFileSync(filename);
        return JSON.parse(data.toString());
    }

    return null;
}

const setMintCache = (address: string, nftList: Nft[]) => {
    const filename = ".mintlist." + address + ".json";
    fs.writeFileSync(filename, JSON.stringify(nftList, null, 2));
}

export const getTraitManifestCache = (filename: string): TraitManifest => {
    const data = fs.readFileSync(filename);
    return TraitManifest.from(JSON.parse(data.toString(), reviver));
}

const setTraitManifestCache = (address: string, manifest: TraitManifest) => {
    const filename = ".traits." + address + ".json";
    fs.writeFileSync(filename, JSON.stringify(manifest, replacer, 2));
}

export const getMintlist = async (metaplex: Metaplex, collectionId: string | null, creatorAddress: string | null) => {
    let mintlist: Nft[] | null = null;
    if (collectionId) {
        mintlist = getMintCache(collectionId);

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
            let mintlist = data.result.map((data: any) => new PublicKey(data.mint));
            let metadatas = await metaplex.nfts().findAllByMintList({ mints: mintlist });
            let count = 0;
            let nftList = await Promise.all(metadatas.map((metadata) => {
                return metaplex.nfts().load({ metadata: metadata as Metadata }) as Promise<Nft>
            }));

            setMintCache(collectionId, nftList);
            return nftList;
        }
    }
    else if (creatorAddress) {
        mintlist = getMintCache(creatorAddress);

        if (!mintlist) {
            let metadatas = await metaplex.nfts().findAllByCreator({ creator: new PublicKey(creatorAddress) });
            let count = 0;
            let nftList = await Promise.all(metadatas.map((metadata) => {
                return metaplex.nfts().load({ metadata: metadata as Metadata }) as Promise<Nft>
            }));

            setMintCache(creatorAddress, nftList);
            return nftList;
        }
    }

    return mintlist as Nft[];
};

const writeAccountsFile = (accounts: any[]) => {
    const filename = "accounts.js";
    fs.writeFileSync(filename, "export const accounts = " + JSON.stringify(accounts, null, 2));
}

const nftToAccount = (nft: Nft) => {
    return [
        {
            label: nft.name,
            accountId: nft.mint.address.toString(),
        },
        {
            label: nft.name + " Metadata",
            accountId: nft.metadataAddress.toString(),
        },
    ]
}

export const getTraitManifest = async (nftList: Nft[]) => {
    let traitManifest = new TraitManifest();
    let accounts: any[] = [];
    for (const nft of nftList) {
        if (nft && nft.json && nft.json.attributes) {
            accounts.push(...nftToAccount(nft));

            for (const attribute of nft.json.attributes) {
                if (attribute.trait_type && attribute.value) {
                    // console.log("Adding " + attribute.trait_type + " : " + attribute.value);
                    traitManifest.add(attribute.trait_type, attribute.value);
                }
            }
        }
    }
    writeAccountsFile(accounts);
    // console.log(nftList[0].collection);
    setTraitManifestCache(nftList[0].collection?.address.toString() as string, traitManifest);
    // console.log(traitManifest);
}

export const getAllFiles = async (directory: string) => {
    let files: string[] = [];
    for (let file of fs.readdirSync(directory)) {
        let filepath = path.join(directory, file);
        if (fs.lstatSync(filepath).isDirectory()) {
            files.push(...(await getAllFiles(filepath)));
        }
        else {
            files.push(filepath);
        }
    }

    console.log(files);
    return files;
}

export const addFilesToTraitManifest = (files: string[], traitManifest: TraitManifest) => {
    // let manifest = new TraitManifest();
    // console.log(manifest);
    // console.log(typeof manifest);
    // console.log(manifest.constructor.name);
    // console.log(manifest.updatePath);
    for (const [traitType, traitObj] of traitManifest.map.entries()) {
        let found;
        for (const [trait, location] of traitObj.map.entries()) {
            found = false;
            for (const file of files) {
                if (file.includes(path.join(traitType, trait))) {
                    traitManifest.updatePath(traitType, trait, file);
                    found = true;
                    break;
                }
            }
            if (found) {
                continue;
            }
            for (const file of files) {
                if (file.includes(trait) && file.includes(traitType)) {
                    traitManifest.updatePath(traitType, trait, file);
                }
            }
        }
    }
    console.log(JSON.stringify(traitManifest, replacer, 2));
    return traitManifest;
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