### result
```
imentus@imentus:~/Documents/im-client/mpl-fork/metaplex-program-library/fixed-price-sale/js$ node test/createMarket.test.js 
TAP version 13
# create-market: success
🚀 ~ file: createMarket.test.ts ~ line 16 ~ test ~ transactionHandler PayerTransactionHandler {
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
      _idleStart: 559,
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
      _idleStart: 876,
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
🚀 ~ file: createMarket.test.ts ~ line 16 ~ test ~ connection Connection {
  _commitment: 'confirmed',
  _confirmTransactionInitialTimeout: undefined,
  _rpcEndpoint: 'http://127.0.0.1:8899/',
  _rpcWsEndpoint: 'ws://127.0.0.1:8900/',
  _rpcClient: ClientBrowser {
    options: {
      reviver: null,
      replacer: null,
      generator: [Function (anonymous)],
      version: 2,
      notificationIdNull: false
    },
    callServer: [AsyncFunction (anonymous)]
  },
  _rpcRequest: [Function (anonymous)],
  _rpcBatchRequest: [Function (anonymous)],
  _rpcWebSocket: Client {
    _events: Events <[Object: null prototype] {}> {
      open: [EE],
      error: [EE],
      close: [EE],
      accountNotification: [EE],
      programNotification: [EE],
      slotNotification: [EE],
      slotsUpdatesNotification: [EE],
      signatureNotification: [EE],
      rootNotification: [EE],
      logsNotification: [EE]
    },
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
    socket: WebSocket {
      _events: [Object: null prototype],
      _eventsCount: 4,
      _maxListeners: undefined,
      _binaryType: 'nodebuffer',
      _closeCode: 1006,
      _closeFrameReceived: false,
      _closeFrameSent: false,
      _closeMessage: '',
      _closeTimer: null,
      _extensions: {},
      _protocol: '',
      _readyState: 1,
      _receiver: [Receiver],
      _sender: [Sender],
      _socket: [Socket],
      _bufferedAmount: 0,
      _isServer: false,
      _redirects: 0,
      _url: 'ws://127.0.0.1:8900/',
      _req: null,
      [Symbol(kCapture)]: false
    }
  },
  _rpcWebSocketConnected: false,
  _rpcWebSocketHeartbeat: Timeout {
    _idleTimeout: 5000,
    _idlePrev: [TimersList],
    _idleNext: [Timeout],
    _idleStart: 559,
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
    _idleStart: 876,
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
}
🚀 ~ file: createMarket.test.ts ~ line 16 ~ test ~ payer BN74PwEAPJZypPzshCeiH9P2A5mQi1Z6gqzsf4nqGFT9
ok 1 confirmed transaction has no error
🚀 ~ file: createMarket.test.ts ~ line 30 ~ test ~ store B11LXP4dyh8YeVpWko2jx8kARv4LWyV3K2HLyActuU6P
ok 2 confirmed transaction has no error
ok 3 confirmed transaction has no error
🚀 ~ file: createMarket.test.ts ~ line 33 ~ test ~ sellingResource ABzsTmfKFzYqdthajjq3hkqkiTCHZLsJbYgXDwkgFTWi
🚀 ~ file: createMarket.test.ts ~ line 43 ~ test ~ treasuryMint 8xKHNQSprU5vhz8A2mjmXXZak2aau5DR9qTqGNbksdqm
ok 4 confirmed transaction has no error
ok 5 confirmed transaction has no error
🚀 ~ file: createMarket.test.ts ~ line 63 ~ test ~ treasuryHolder BM2inY2Wmk374Jz2mB4PX8dAB4URXruVJrQeLxvD5hpA
🚀 ~ file: createMarket.test.ts ~ line 63 ~ test ~ market 7AEs76jZ5i8K9L2L4msSeqf4cwd8KE8gbK3Q76tFtBRp

1..5
# tests 5
# pass  5

# ok
```