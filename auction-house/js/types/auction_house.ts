export type AuctionHouse = {
  version: '0.1.0';
  name: 'auction_house';
  instructions: [
    {
      name: 'withdrawFromFee';
      accounts: [
        {
          name: 'authority';
          isMut: false;
          isSigner: true;
        },
        {
          name: 'feeWithdrawalDestination';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'auctionHouseFeeAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'auctionHouse';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        },
      ];
      args: [
        {
          name: 'amount';
          type: 'u64';
        },
      ];
    },
    {
      name: 'withdrawFromTreasury';
      accounts: [
        {
          name: 'treasuryMint';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: true;
        },
        {
          name: 'treasuryWithdrawalDestination';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'auctionHouseTreasury';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'auctionHouse';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        },
      ];
      args: [
        {
          name: 'amount';
          type: 'u64';
        },
      ];
    },
    {
      name: 'updateAuctionHouse';
      accounts: [
        {
          name: 'treasuryMint';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'payer';
          isMut: false;
          isSigner: true;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: true;
        },
        {
          name: 'newAuthority';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'feeWithdrawalDestination';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'treasuryWithdrawalDestination';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'treasuryWithdrawalDestinationOwner';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouse';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'ataProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'rent';
          isMut: false;
          isSigner: false;
        },
      ];
      args: [
        {
          name: 'sellerFeeBasisPoints';
          type: {
            option: 'u16';
          };
        },
        {
          name: 'requiresSignOff';
          type: {
            option: 'bool';
          };
        },
        {
          name: 'canChangeSalePrice';
          type: {
            option: 'bool';
          };
        },
      ];
    },
    {
      name: 'createAuctionHouse';
      accounts: [
        {
          name: 'treasuryMint';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'payer';
          isMut: false;
          isSigner: true;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'feeWithdrawalDestination';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'treasuryWithdrawalDestination';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'treasuryWithdrawalDestinationOwner';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouse';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'auctionHouseFeeAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'auctionHouseTreasury';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'ataProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'rent';
          isMut: false;
          isSigner: false;
        },
      ];
      args: [
        {
          name: 'bump';
          type: 'u8';
        },
        {
          name: 'feePayerBump';
          type: 'u8';
        },
        {
          name: 'treasuryBump';
          type: 'u8';
        },
        {
          name: 'sellerFeeBasisPoints';
          type: 'u16';
        },
        {
          name: 'requiresSignOff';
          type: 'bool';
        },
        {
          name: 'canChangeSalePrice';
          type: 'bool';
        },
      ];
    },
    {
      name: 'withdraw';
      accounts: [
        {
          name: 'wallet';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'receiptAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'escrowPaymentAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'treasuryMint';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouse';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouseFeeAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'ataProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'rent';
          isMut: false;
          isSigner: false;
        },
      ];
      args: [
        {
          name: 'escrowPaymentBump';
          type: 'u8';
        },
        {
          name: 'amount';
          type: 'u64';
        },
      ];
    },
    {
      name: 'deposit';
      accounts: [
        {
          name: 'wallet';
          isMut: false;
          isSigner: true;
        },
        {
          name: 'paymentAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'transferAuthority';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'escrowPaymentAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'treasuryMint';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouse';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouseFeeAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'rent';
          isMut: false;
          isSigner: false;
        },
      ];
      args: [
        {
          name: 'escrowPaymentBump';
          type: 'u8';
        },
        {
          name: 'amount';
          type: 'u64';
        },
      ];
    },
    {
      name: 'cancel';
      accounts: [
        {
          name: 'wallet';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenMint';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouse';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouseFeeAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tradeState';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        },
      ];
      args: [
        {
          name: 'buyerPrice';
          type: 'u64';
        },
        {
          name: 'tokenSize';
          type: 'u64';
        },
      ];
    },
    {
      name: 'executeSale';
      accounts: [
        {
          name: 'buyer';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'seller';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenMint';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'metadata';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'treasuryMint';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'escrowPaymentAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'sellerPaymentReceiptAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'buyerReceiptTokenAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouse';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouseFeeAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'auctionHouseTreasury';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'buyerTradeState';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'sellerTradeState';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'freeTradeState';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'ataProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'programAsSigner';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'rent';
          isMut: false;
          isSigner: false;
        },
      ];
      args: [
        {
          name: 'escrowPaymentBump';
          type: 'u8';
        },
        {
          name: 'freeTradeStateBump';
          type: 'u8';
        },
        {
          name: 'programAsSignerBump';
          type: 'u8';
        },
        {
          name: 'buyerPrice';
          type: 'u64';
        },
        {
          name: 'tokenSize';
          type: 'u64';
        },
      ];
    },
    {
      name: 'sell';
      accounts: [
        {
          name: 'wallet';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'tokenAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'metadata';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouse';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouseFeeAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'sellerTradeState';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'freeSellerTradeState';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'programAsSigner';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'rent';
          isMut: false;
          isSigner: false;
        },
      ];
      args: [
        {
          name: 'tradeStateBump';
          type: 'u8';
        },
        {
          name: 'freeTradeStateBump';
          type: 'u8';
        },
        {
          name: 'programAsSignerBump';
          type: 'u8';
        },
        {
          name: 'buyerPrice';
          type: 'u64';
        },
        {
          name: 'tokenSize';
          type: 'u64';
        },
      ];
    },
    {
      name: 'buy';
      accounts: [
        {
          name: 'wallet';
          isMut: false;
          isSigner: true;
        },
        {
          name: 'paymentAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'transferAuthority';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'treasuryMint';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'tokenAccount';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'metadata';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'escrowPaymentAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouse';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'auctionHouseFeeAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'buyerTradeState';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'rent';
          isMut: false;
          isSigner: false;
        },
      ];
      args: [
        {
          name: 'tradeStateBump';
          type: 'u8';
        },
        {
          name: 'escrowPaymentBump';
          type: 'u8';
        },
        {
          name: 'buyerPrice';
          type: 'u64';
        },
        {
          name: 'tokenSize';
          type: 'u64';
        },
      ];
    },
  ];
  accounts: [
    {
      name: 'auctionHouse';
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'auctionHouseFeeAccount';
            type: 'publicKey';
          },
          {
            name: 'auctionHouseTreasury';
            type: 'publicKey';
          },
          {
            name: 'treasuryWithdrawalDestination';
            type: 'publicKey';
          },
          {
            name: 'feeWithdrawalDestination';
            type: 'publicKey';
          },
          {
            name: 'treasuryMint';
            type: 'publicKey';
          },
          {
            name: 'authority';
            type: 'publicKey';
          },
          {
            name: 'creator';
            type: 'publicKey';
          },
          {
            name: 'bump';
            type: 'u8';
          },
          {
            name: 'treasuryBump';
            type: 'u8';
          },
          {
            name: 'feePayerBump';
            type: 'u8';
          },
          {
            name: 'sellerFeeBasisPoints';
            type: 'u16';
          },
          {
            name: 'requiresSignOff';
            type: 'bool';
          },
          {
            name: 'canChangeSalePrice';
            type: 'bool';
          },
        ];
      };
    },
  ];
  errors: [
    {
      code: 6000;
      name: 'PublicKeyMismatch';
      msg: 'PublicKeyMismatch';
    },
    {
      code: 6001;
      name: 'InvalidMintAuthority';
      msg: 'InvalidMintAuthority';
    },
    {
      code: 6002;
      name: 'UninitializedAccount';
      msg: 'UninitializedAccount';
    },
    {
      code: 6003;
      name: 'IncorrectOwner';
      msg: 'IncorrectOwner';
    },
    {
      code: 6004;
      name: 'PublicKeysShouldBeUnique';
      msg: 'PublicKeysShouldBeUnique';
    },
    {
      code: 6005;
      name: 'StatementFalse';
      msg: 'StatementFalse';
    },
    {
      code: 6006;
      name: 'NotRentExempt';
      msg: 'NotRentExempt';
    },
    {
      code: 6007;
      name: 'NumericalOverflow';
      msg: 'NumericalOverflow';
    },
    {
      code: 6008;
      name: 'ExpectedSolAccount';
      msg: 'Expected a sol account but got an spl token account instead';
    },
    {
      code: 6009;
      name: 'CannotExchangeSOLForSol';
      msg: 'Cannot exchange sol for sol';
    },
    {
      code: 6010;
      name: 'SOLWalletMustSign';
      msg: 'If paying with sol, sol wallet must be signer';
    },
    {
      code: 6011;
      name: 'CannotTakeThisActionWithoutAuctionHouseSignOff';
      msg: 'Cannot take this action without auction house signing too';
    },
    {
      code: 6012;
      name: 'NoPayerPresent';
      msg: 'No payer present on this txn';
    },
    {
      code: 6013;
      name: 'DerivedKeyInvalid';
      msg: 'Derived key invalid';
    },
    {
      code: 6014;
      name: 'MetadataDoesntExist';
      msg: "Metadata doesn't exist";
    },
    {
      code: 6015;
      name: 'InvalidTokenAmount';
      msg: 'Invalid token amount';
    },
    {
      code: 6016;
      name: 'BothPartiesNeedToAgreeToSale';
      msg: 'Both parties need to agree to this sale';
    },
    {
      code: 6017;
      name: 'CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff';
      msg: 'Cannot match free sales unless the auction house or seller signs off';
    },
    {
      code: 6018;
      name: 'SaleRequiresSigner';
      msg: 'This sale requires a signer';
    },
    {
      code: 6019;
      name: 'OldSellerNotInitialized';
      msg: 'Old seller not initialized';
    },
    {
      code: 6020;
      name: 'SellerATACannotHaveDelegate';
      msg: 'Seller ata cannot have a delegate set';
    },
    {
      code: 6021;
      name: 'BuyerATACannotHaveDelegate';
      msg: 'Buyer ata cannot have a delegate set';
    },
    {
      code: 6022;
      name: 'NoValidSignerPresent';
      msg: 'No valid signer present';
    },
    {
      code: 6023;
      name: 'InvalidBasisPoints';
      msg: 'BP must be less than or equal to 10000';
    },
  ];
};

export const IDL: AuctionHouse = {
  version: '0.1.0',
  name: 'auction_house',
  instructions: [
    {
      name: 'withdrawFromFee',
      accounts: [
        {
          name: 'authority',
          isMut: false,
          isSigner: true,
        },
        {
          name: 'feeWithdrawalDestination',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'auctionHouseFeeAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'auctionHouse',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'amount',
          type: 'u64',
        },
      ],
    },
    {
      name: 'withdrawFromTreasury',
      accounts: [
        {
          name: 'treasuryMint',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'authority',
          isMut: false,
          isSigner: true,
        },
        {
          name: 'treasuryWithdrawalDestination',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'auctionHouseTreasury',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'auctionHouse',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'amount',
          type: 'u64',
        },
      ],
    },
    {
      name: 'updateAuctionHouse',
      accounts: [
        {
          name: 'treasuryMint',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'payer',
          isMut: false,
          isSigner: true,
        },
        {
          name: 'authority',
          isMut: false,
          isSigner: true,
        },
        {
          name: 'newAuthority',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'feeWithdrawalDestination',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'treasuryWithdrawalDestination',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'treasuryWithdrawalDestinationOwner',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouse',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'ataProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'rent',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'sellerFeeBasisPoints',
          type: {
            option: 'u16',
          },
        },
        {
          name: 'requiresSignOff',
          type: {
            option: 'bool',
          },
        },
        {
          name: 'canChangeSalePrice',
          type: {
            option: 'bool',
          },
        },
      ],
    },
    {
      name: 'createAuctionHouse',
      accounts: [
        {
          name: 'treasuryMint',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'payer',
          isMut: false,
          isSigner: true,
        },
        {
          name: 'authority',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'feeWithdrawalDestination',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'treasuryWithdrawalDestination',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'treasuryWithdrawalDestinationOwner',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouse',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'auctionHouseFeeAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'auctionHouseTreasury',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'ataProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'rent',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'bump',
          type: 'u8',
        },
        {
          name: 'feePayerBump',
          type: 'u8',
        },
        {
          name: 'treasuryBump',
          type: 'u8',
        },
        {
          name: 'sellerFeeBasisPoints',
          type: 'u16',
        },
        {
          name: 'requiresSignOff',
          type: 'bool',
        },
        {
          name: 'canChangeSalePrice',
          type: 'bool',
        },
      ],
    },
    {
      name: 'withdraw',
      accounts: [
        {
          name: 'wallet',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'receiptAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'escrowPaymentAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'treasuryMint',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'authority',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouse',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouseFeeAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'ataProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'rent',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'escrowPaymentBump',
          type: 'u8',
        },
        {
          name: 'amount',
          type: 'u64',
        },
      ],
    },
    {
      name: 'deposit',
      accounts: [
        {
          name: 'wallet',
          isMut: false,
          isSigner: true,
        },
        {
          name: 'paymentAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'transferAuthority',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'escrowPaymentAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'treasuryMint',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'authority',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouse',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouseFeeAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'rent',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'escrowPaymentBump',
          type: 'u8',
        },
        {
          name: 'amount',
          type: 'u64',
        },
      ],
    },
    {
      name: 'cancel',
      accounts: [
        {
          name: 'wallet',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenMint',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'authority',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouse',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouseFeeAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tradeState',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenProgram',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'buyerPrice',
          type: 'u64',
        },
        {
          name: 'tokenSize',
          type: 'u64',
        },
      ],
    },
    {
      name: 'executeSale',
      accounts: [
        {
          name: 'buyer',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'seller',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenMint',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'metadata',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'treasuryMint',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'escrowPaymentAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'sellerPaymentReceiptAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'buyerReceiptTokenAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'authority',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouse',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouseFeeAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'auctionHouseTreasury',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'buyerTradeState',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'sellerTradeState',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'freeTradeState',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'ataProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'programAsSigner',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'rent',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'escrowPaymentBump',
          type: 'u8',
        },
        {
          name: 'freeTradeStateBump',
          type: 'u8',
        },
        {
          name: 'programAsSignerBump',
          type: 'u8',
        },
        {
          name: 'buyerPrice',
          type: 'u64',
        },
        {
          name: 'tokenSize',
          type: 'u64',
        },
      ],
    },
    {
      name: 'sell',
      accounts: [
        {
          name: 'wallet',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'tokenAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'metadata',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'authority',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouse',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouseFeeAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'sellerTradeState',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'freeSellerTradeState',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'programAsSigner',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'rent',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'tradeStateBump',
          type: 'u8',
        },
        {
          name: 'freeTradeStateBump',
          type: 'u8',
        },
        {
          name: 'programAsSignerBump',
          type: 'u8',
        },
        {
          name: 'buyerPrice',
          type: 'u64',
        },
        {
          name: 'tokenSize',
          type: 'u64',
        },
      ],
    },
    {
      name: 'buy',
      accounts: [
        {
          name: 'wallet',
          isMut: false,
          isSigner: true,
        },
        {
          name: 'paymentAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'transferAuthority',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'treasuryMint',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'tokenAccount',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'metadata',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'escrowPaymentAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'authority',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouse',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'auctionHouseFeeAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'buyerTradeState',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'tokenProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
        {
          name: 'rent',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'tradeStateBump',
          type: 'u8',
        },
        {
          name: 'escrowPaymentBump',
          type: 'u8',
        },
        {
          name: 'buyerPrice',
          type: 'u64',
        },
        {
          name: 'tokenSize',
          type: 'u64',
        },
      ],
    },
  ],
  accounts: [
    {
      name: 'auctionHouse',
      type: {
        kind: 'struct',
        fields: [
          {
            name: 'auctionHouseFeeAccount',
            type: 'publicKey',
          },
          {
            name: 'auctionHouseTreasury',
            type: 'publicKey',
          },
          {
            name: 'treasuryWithdrawalDestination',
            type: 'publicKey',
          },
          {
            name: 'feeWithdrawalDestination',
            type: 'publicKey',
          },
          {
            name: 'treasuryMint',
            type: 'publicKey',
          },
          {
            name: 'authority',
            type: 'publicKey',
          },
          {
            name: 'creator',
            type: 'publicKey',
          },
          {
            name: 'bump',
            type: 'u8',
          },
          {
            name: 'treasuryBump',
            type: 'u8',
          },
          {
            name: 'feePayerBump',
            type: 'u8',
          },
          {
            name: 'sellerFeeBasisPoints',
            type: 'u16',
          },
          {
            name: 'requiresSignOff',
            type: 'bool',
          },
          {
            name: 'canChangeSalePrice',
            type: 'bool',
          },
        ],
      },
    },
  ],
  errors: [
    {
      code: 6000,
      name: 'PublicKeyMismatch',
      msg: 'PublicKeyMismatch',
    },
    {
      code: 6001,
      name: 'InvalidMintAuthority',
      msg: 'InvalidMintAuthority',
    },
    {
      code: 6002,
      name: 'UninitializedAccount',
      msg: 'UninitializedAccount',
    },
    {
      code: 6003,
      name: 'IncorrectOwner',
      msg: 'IncorrectOwner',
    },
    {
      code: 6004,
      name: 'PublicKeysShouldBeUnique',
      msg: 'PublicKeysShouldBeUnique',
    },
    {
      code: 6005,
      name: 'StatementFalse',
      msg: 'StatementFalse',
    },
    {
      code: 6006,
      name: 'NotRentExempt',
      msg: 'NotRentExempt',
    },
    {
      code: 6007,
      name: 'NumericalOverflow',
      msg: 'NumericalOverflow',
    },
    {
      code: 6008,
      name: 'ExpectedSolAccount',
      msg: 'Expected a sol account but got an spl token account instead',
    },
    {
      code: 6009,
      name: 'CannotExchangeSOLForSol',
      msg: 'Cannot exchange sol for sol',
    },
    {
      code: 6010,
      name: 'SOLWalletMustSign',
      msg: 'If paying with sol, sol wallet must be signer',
    },
    {
      code: 6011,
      name: 'CannotTakeThisActionWithoutAuctionHouseSignOff',
      msg: 'Cannot take this action without auction house signing too',
    },
    {
      code: 6012,
      name: 'NoPayerPresent',
      msg: 'No payer present on this txn',
    },
    {
      code: 6013,
      name: 'DerivedKeyInvalid',
      msg: 'Derived key invalid',
    },
    {
      code: 6014,
      name: 'MetadataDoesntExist',
      msg: "Metadata doesn't exist",
    },
    {
      code: 6015,
      name: 'InvalidTokenAmount',
      msg: 'Invalid token amount',
    },
    {
      code: 6016,
      name: 'BothPartiesNeedToAgreeToSale',
      msg: 'Both parties need to agree to this sale',
    },
    {
      code: 6017,
      name: 'CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff',
      msg: 'Cannot match free sales unless the auction house or seller signs off',
    },
    {
      code: 6018,
      name: 'SaleRequiresSigner',
      msg: 'This sale requires a signer',
    },
    {
      code: 6019,
      name: 'OldSellerNotInitialized',
      msg: 'Old seller not initialized',
    },
    {
      code: 6020,
      name: 'SellerATACannotHaveDelegate',
      msg: 'Seller ata cannot have a delegate set',
    },
    {
      code: 6021,
      name: 'BuyerATACannotHaveDelegate',
      msg: 'Buyer ata cannot have a delegate set',
    },
    {
      code: 6022,
      name: 'NoValidSignerPresent',
      msg: 'No valid signer present',
    },
    {
      code: 6023,
      name: 'InvalidBasisPoints',
      msg: 'BP must be less than or equal to 10000',
    },
  ],
};
