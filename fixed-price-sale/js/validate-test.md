### result
```
imentus@imentus:~/Documents/im-client/mpl-fork/metaplex-program-library/fixed-price-sale/js$ node test/validate.test.js 
TAP version 13
# validate: successful purchase and validation
🚀 ~ file: validate.test.ts ~ line 28 ~ test ~ transactionHandler -  payer CDTZK5YcChoX31LBnVd9Ey1rVz4kKiEtvRmFVQeRXQ8f
🚀 ~ file: validate.test.ts ~ line 28 ~ test ~ connection - rpcendpoint http://127.0.0.1:8899/
🚀 ~ file: validate.test.ts ~ line 28 ~ test ~ payer CDTZK5YcChoX31LBnVd9Ey1rVz4kKiEtvRmFVQeRXQ8f
ok 1 confirmed transaction has no error
🚀 ~ file: validate.test.ts ~ line 42 ~ test ~ store C8JV2BhkBVZSsbTAumEafVDxKHg5bNgVsAHyqiX5Pzjd
ok 2 confirmed transaction has no error
ok 3 confirmed transaction has no error
🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ resourceMint 46FwZRGQJHBB35m6fQDiVXJTSm8b7kv5yx3a3mHa3CBp
🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ vaultOwnerBump 255
🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ vaultOwner 3KJKuPo4bGdpeRhQNPjM18tbfhoq9YGsYx88u8bm5KDS
🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ vault 4rrDvcVs9AZsLiVVsVn6EupTTubW94AsrgqWZVvkxd4z
🚀 ~ file: validate.test.ts ~ line 44 ~ test ~ sellingResource B2JTR8ASyvktkEsx27aaJSBiTwsjrk8et7cH5ULj6RvL
🚀 ~ file: validate.test.ts ~ line 60 ~ test ~ treasuryMint WQiP8QQsF6iTAsi5i3fNQRGFHNwV5TfKdexy6EY7RXV
🚀 ~ file: validate.test.ts ~ line 65 ~ test ~ userTokenAcc 52idCbmueSf3udaUtgT8MBndLBVA78rQVve7wxq81Txp
ok 4 confirmed transaction has no error
ok 5 confirmed transaction has no error
🚀 ~ file: validate.test.ts ~ line 79 ~ test ~ treasuryHolder 6eJ4bwZNH2HoW1CLHZunXyTAyYhnEUZth8j6rVhENRQa
🚀 ~ file: validate.test.ts ~ line 89 ~ test ~ market 5eZN4tpWc1YouakdaN7cnqT8NwHzM3xJfQz2PdEMM7kB
🚀 ~ file: validate.test.ts ~ line 92 ~ test ~ tradeHistoryBump 255
🚀 ~ file: validate.test.ts ~ line 95 ~ test ~ tradeHistory 2AeTVzrgXBT1jaSUXHeYuxpdcmS8uJEZXJTYURSnmcAk
🚀 ~ file: validate.test.ts ~ line 99 ~ test ~ newMint AkhMeZWkdJanWdKUSHxk7YPyy3zx6yMv1VAaBv96f9SH
🚀 ~ file: validate.test.ts ~ line 104 ~ test ~ newMintAta C6njWgWwVGjGL3vXigFM37pLbsjge8xV8pRYatfZYAov
🚀 ~ file: validate.test.ts ~ line 109 ~ test ~ newMintEdition 44ZiTeh4wYyoqf39Q9Ecv4xR9h2xJhXFNG9CDuWaSHkt
🚀 ~ file: validate.test.ts ~ line 111 ~ test ~ newMintMetadata 9mWfAB99TiJoX4jagADVzhFxg4jDgVfCqMQajueE4zkk
🚀 ~ file: validate.test.ts ~ line 114 ~ test ~ resourceMintMasterEdition 9wQWaE6u9zNyQhTgZDthKKdxd9ZbfRs1GFQXfssfWHiU
🚀 ~ file: validate.test.ts ~ line 116 ~ test ~ resourceMintMetadata 3baNnPKKSve6jtrSYjBSKCoaNaYMRyujhCKk49gzjWMj
🚀 ~ file: validate.test.ts ~ line 118 ~ test ~ resourceMintEditionMarker 7NwquVA8weUimAeYMq2o7RB6arKvBh3LiSrHeKCtgn5F
🚀 ~ file: validate.test.ts ~ line 123 ~ test ~ buyTx Transaction {
  signatures: [],
  feePayer: PublicKey {
    _bn: <BN: a6a18dd9dacb3713f0f61335413d044d2acd1a089cd2282ee969c48576e98e08>
  },
  instructions: [
    TransactionInstruction {
      keys: [Array],
      programId: [PublicKey],
      data: <Buffer 66 06 3d 12 01 da eb ea ff ff>
    }
  ],
  recentBlockhash: 'GNp8F8ZTdWsxERsyW62SDugQ1zeFHapaTuNvGLbgo2Zt',
  nonceInfo: undefined
}
🚀 ~ file: validate.test.ts ~ line 148 ~ test ~ buyRes 9bwwuCAiLV1ipXsuXQP6514FwPFYNZr3Pr4YJaqUsyWX1cDVigGQ6THB1k8jWrTJahyZQidqcviQ191tWmkLZML
ok 6 confirmed transaction has no error
🚀 ~ file: validate.test.ts ~ line 154 ~ test ~ MasterEdition 9wQWaE6u9zNyQhTgZDthKKdxd9ZbfRs1GFQXfssfWHiU
Built in console:  Master Edition me:  9wQWaE6u9zNyQhTgZDthKKdxd9ZbfRs1GFQXfssfWHiU resourceMintMasterEdition:  9wQWaE6u9zNyQhTgZDthKKdxd9ZbfRs1GFQXfssfWHiU userTokenAcc:  52idCbmueSf3udaUtgT8MBndLBVA78rQVve7wxq81Txp
🚀 ~ file: validate.test.ts ~ line 161 ~ test ~ TokenAccount C6njWgWwVGjGL3vXigFM37pLbsjge8xV8pRYatfZYAov
🚀 ~ file: validate.test.ts ~ line 163 ~ test ~ result true
ok 7 should be strictly equal
# validate: successful purchase and failed validation
ok 8 confirmed transaction has no error
🚀 ~ file: validate.test.ts ~ line 183 ~ test2 ~ store 7S8AcjVpdy7CCQpkeDXNmRhyKM2xiwsJ3Ad8ZqUruEvX
ok 9 confirmed transaction has no error
ok 10 confirmed transaction has no error
🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ resourceMint B2mTMFHp1UA6uV6k52zzKkJp35KRViCTMZwDjjs4qGUZ
🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ vaultOwnerBump 255
🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ vaultOwner 8hRub7SE8dLsiGeEoPE8h2mwSuPjNgKZa4KRN4VTpREs
🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ vault HrzqKFugSdhPARu5gCH7AMg5fZMfHN6CZRRDvhJzctc5
🚀 ~ file: validate.test.ts ~ line 186 ~ test2 ~ sellingResource BLf7tSQfA86KaY9bELgbjxgjSiD99141dRxfoZdemJWW
🚀 ~ file: validate.test.ts ~ line 201 ~ test2 ~ treasuryMint FZvRLHCHsA3kYXGpGjLstpcxfdDjeRsGToHNAoHe6TCM
🚀 ~ file: validate.test.ts ~ line 206 ~ test2 ~ userTokenAcc GJ3TXmNBXPdwbponcESSCkfywhnqKMjFhbjunJyd5ghD
ok 11 confirmed transaction has no error
ok 12 confirmed transaction has no error
🚀 ~ file: validate.test.ts ~ line 220 ~ test2 ~ market 8n55gw6Jvtjw4phLYus7QrTyynvhUADp4s1kgkdbs7NR
🚀 ~ file: validate.test.ts ~ line 220 ~ test2 ~ treasuryHolder 4CvrAhU3ckopjKpSVscaLYBq3AhuXyryNipD2ocNu1Qr
🚀 ~ file: validate.test.ts ~ line 237 ~ test2 ~ tradeHistory B2vgshb3E8358N9nA2pcLpTh9v772H9T7yPQvoRJLVse
🚀 ~ file: validate.test.ts ~ line 233 ~ test2 ~ tradeHistoryBump 255
🚀 ~ file: validate.test.ts ~ line 241 ~ test2 ~ newMint AYqSAAWXp86qfdMk4oji3Q1P4Ctbybdo5kCiCohnSG1U
🚀 ~ file: validate.test.ts ~ line 246 ~ test2 ~ newMintAta s6kSnD2BsXMKtitH3AUtxXNSQ42ApYFGGjh3EBpBrkk
🚀 ~ file: validate.test.ts ~ line 251 ~ test2 ~ newMintEdition 4ZjsrpdHkSSseFBBcbreUcZX225ehUDhkuQBL7hpiF3N
🚀 ~ file: validate.test.ts ~ line 253 ~ test2 ~ newMintMetadata BrQDvSma9oAuwDceQGjoXWAySPxnrLaDzv17ZMGe8LTs
🚀 ~ file: validate.test.ts ~ line 256 ~ test2 ~ resourceMintMasterEdition 2AaPE3pdr9PHXwZGaB7dfC3vtRhiFq5mkx5SaoMKqEHL
🚀 ~ file: validate.test.ts ~ line 257 ~ test2 ~ resourceMintMetadata 2y7uBwy9FGng8iLKoSE4y6atmQhNapBDKayZLjAKzT7y
🚀 ~ file: validate.test.ts ~ line 260 ~ test2 ~ resourceMintEditionMarker 8BkLA2xLz69WfqFroTMJgQZ5cbw23PSKzCoYxtsM3qzg
🚀 ~ file: validate.test.ts ~ line 265 ~ test2 ~ buyTx Transaction {
  signatures: [],
  feePayer: PublicKey {
    _bn: <BN: 998bd4d2b7c2f84fa1e3b1b653e9a59d0015eb5568a5efcfafb545551cc77b04>
  },
  instructions: [
    TransactionInstruction {
      keys: [Array],
      programId: [PublicKey],
      data: <Buffer 66 06 3d 12 01 da eb ea ff ff>
    }
  ],
  recentBlockhash: '87TuczocrSX3BxJx8zjqB5X6dcdwcECsv7iGybK5quQ6',
  nonceInfo: undefined
}
🚀 ~ file: validate.test.ts ~ line 291 ~ test2 ~ buyRes 2BRtCGSwRPhY5yxJ4JCagfKxCfDrk77r8CMVS7o1xWDixLcfyMVkuqd3zVknprae1JWdMCiQ93STLv5HbK8QJZM4
ok 13 confirmed transaction has no error
🚀 ~ file: validate.test.ts ~ line 297 ~ test2 ~ masterEdition HsC5HyjponesSEDMkYMt3DjSWv5ashbpiNWd5MvJDe8s
🚀 ~ file: validate.test.ts ~ line 280 ~ test2 ~ me HsC5HyjponesSEDMkYMt3DjSWv5ashbpiNWd5MvJDe8s
🚀 ~ file: validate.test.ts ~ line 280 ~ test2 ~ masterEdition HsC5HyjponesSEDMkYMt3DjSWv5ashbpiNWd5MvJDe8s
🚀 ~ file: validate.test.ts ~ line 308 ~ test2 ~ ta s6kSnD2BsXMKtitH3AUtxXNSQ42ApYFGGjh3EBpBrkk
🚀 ~ file: validate.test.ts ~ line 310 ~ test2 ~ result false
ok 14 should be strictly equal

1..14
# tests 14
# pass  14

# ok
```