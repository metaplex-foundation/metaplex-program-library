import { Provider, InstructionNamespace } from '@project-serum/anchor';
import { Transaction } from '@metaplex-foundation/mpl-core';
import { AuctionHouseProgram } from '../AuctionHouseProgram';
import { AuctionHouse } from '../../types/auction_house';

type CreateAuctionHouseParams = Parameters<
  InstructionNamespace<AuctionHouse>['createAuctionHouse']
>;

export class CreateAuctionHouse extends Transaction {
  constructor(provider: Provider, params: CreateAuctionHouseParams) {
    super();

    const program = new AuctionHouseProgram(provider);

    this.add(program.instruction.createAuctionHouse(...params));
  }
}
