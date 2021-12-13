import { Provider, InstructionNamespace } from '@project-serum/anchor';
import { Transaction } from '@metaplex-foundation/mpl-core';
import { AuctionHouseProgram } from '../AuctionHouseProgram';
import { AuctionHouse } from '../../types/auction_house';

type UpdateAuctionHouseParams = Parameters<
  InstructionNamespace<AuctionHouse>['updateAuctionHouse']
>;

export class UpdateAuctionHouse extends Transaction {
  constructor(provider: Provider, params: UpdateAuctionHouseParams) {
    super();

    const program = new AuctionHouseProgram(provider);

    this.add(program.instruction.updateAuctionHouse(...params));
  }
}
