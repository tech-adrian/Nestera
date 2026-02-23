import { Global, Module } from '@nestjs/common';
import { StellarService } from './stellar.service';
import { BlockchainController } from './blockchain.controller';

@Global()
@Module({
  controllers: [BlockchainController],
  providers: [StellarService],
  exports: [StellarService],
})
export class BlockchainModule {}
