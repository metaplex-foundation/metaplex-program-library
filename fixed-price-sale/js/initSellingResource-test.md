### result
```
imentus@imentus:~/Documents/im-client/mpl-fork/metaplex-program-library/fixed-price-sale/js$ node test/initSellingResource.test.js 
TAP version 13
# init-selling-resource: success
🚀 ~ file: initSellingResource.test.ts ~ line 10 ~ test ~ transactionHandler PayerTransactionHandler {
  connection: Connection {
    _commitment: 'confirmed',
    _confirmTransactionInitialTimeout: undefined,
    _rpcEndpoint: 'http://127.0.0.1:8899/',
    _rpcWsEndpoint: 'ws://127.0.0.1:8900/',
    _rpcClient: ClientBrowser {
      options: [Object],
      callServer: [AsyncFunction (anonymous)]
    },
    _rpcRequest: [Function (anonymous)],
    _rpcBatchRequest: [Function (anonymous)],
    _rpcWebSocket: Client {
      _events: [Events <Complex prototype>],
      _eventsCount: 10,
      webSocketFactory: [Function: _default],
      queue: {},
      rpc_id: 1,
      address: 'ws://127.0.0.1:8900/',
      autoconnect: false,
      ready: true,
      reconnect: true,
      reconnect_interval: 1000,
      max_reconnects: Infinity,
      rest_options: {},
      current_reconnects: 0,
      generate_request_id: [Function (anonymous)],
      socket: [WebSocket]
    },
    _rpcWebSocketConnected: false,
    _rpcWebSocketHeartbeat: Timeout {
      _idleTimeout: 5000,
      _idlePrev: [TimersList],
      _idleNext: [Timeout],
      _idleStart: 576,
      _onTimeout: [Function (anonymous)],
      _timerArgs: undefined,
      _repeat: 5000,
      _destroyed: false,
      [Symbol(refed)]: true,
      [Symbol(kHasPrimitive)]: false,
      [Symbol(asyncId)]: 45,
      [Symbol(triggerId)]: 36
    },
    _rpcWebSocketIdleTimeout: Timeout {
      _idleTimeout: 500,
      _idlePrev: [TimersList],
      _idleNext: [TimersList],
      _idleStart: 590,
      _onTimeout: [Function (anonymous)],
      _timerArgs: undefined,
      _repeat: null,
      _destroyed: false,
      [Symbol(refed)]: true,
      [Symbol(kHasPrimitive)]: false,
      [Symbol(asyncId)]: 53,
      [Symbol(triggerId)]: 0
    },
    _disableBlockhashCaching: false,
    _pollingBlockhash: false,
    _blockhashInfo: {
      recentBlockhash: null,
      lastFetch: 0,
      transactionSignatures: [],
      simulatedSignatures: []
    },
    _accountChangeSubscriptionCounter: 0,
    _accountChangeSubscriptions: {},
    _programAccountChangeSubscriptionCounter: 0,
    _programAccountChangeSubscriptions: {},
    _rootSubscriptionCounter: 0,
    _rootSubscriptions: {},
    _signatureSubscriptionCounter: 1,
    _signatureSubscriptions: {},
    _slotSubscriptionCounter: 0,
    _slotSubscriptions: {},
    _logsSubscriptionCounter: 0,
    _logsSubscriptions: {},
    _slotUpdateSubscriptionCounter: 0,
    _slotUpdateSubscriptions: {}
  },
  payer: Keypair {
    _keypair: { publicKey: [Uint8Array], secretKey: [Uint8Array] }
  }
}
🚀 ~ file: initSellingResource.test.ts ~ line 10 ~ test ~ connection http://127.0.0.1:8899/
🚀 ~ file: initSellingResource.test.ts ~ line 10 ~ test ~ payer ECYSAUfL2hhemPkfNg43RDVQqXBuYK76JywuTe5U6id4
ok 1 confirmed transaction has no error
🚀 ~ file: initSellingResource.test.ts ~ line 24 ~ test ~ store DV6d3t13DXDzhZK3eRfa996cC3rnV2Kevi27dZ81CGdZ
ok 2 confirmed transaction has no error
ok 3 confirmed transaction has no error
🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ metadata GiG7bX7mbgzr7A9c3EZCyqZzucspEj6zJoPs3GLhNoe2
🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ resourceMint BH5VmGzgPFTdZmkJWHy5uae3z9oSH9JDoJW7ZxBf5n1q
🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ vaultOwnerBump 253
🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ vaultOwner 6xZkxdGdjLokUB8XAdufQbbjFU3T45BMGeT4928TxTNS
🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ vault D6v5okg6JRDCawbdEk1D6HnKpwbPfCU5VLq2VeRKktF3
🚀 ~ file: initSellingResource.test.ts ~ line 41 ~ test ~ sellingResource J4oM8jHg4LtKtjKtSGGERSiPhC5pegP3mzKyWymJNJgB

1..3
# tests 3
# pass  3

# ok
```