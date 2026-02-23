import { Controller, Post } from '@nestjs/common';
import { StellarService } from './stellar.service';

@Controller('blockchain')
export class BlockchainController {
  constructor(private readonly stellarService: StellarService) {}

  @Post('wallets/generate')
  generateWallet() {
    return this.stellarService.generateKeypair();
  }
}
