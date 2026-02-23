import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import {
  Asset,
  Horizon,
  Networks,
  rpc,
  Transaction,
  xdr,
} from '@stellar/stellar-sdk';

@Injectable()
export class StellarService implements OnModuleInit {
  private readonly logger = new Logger(StellarService.name);
  private rpcServer: rpc.Server;
  private horizonServer: Horizon.Server;

  constructor(private configService: ConfigService) {
    const rpcUrl = this.configService.get<string>('stellar.rpcUrl') || '';
    const horizonUrl =
      this.configService.get<string>('stellar.horizonUrl') || '';

    this.rpcServer = new rpc.Server(rpcUrl);
    this.horizonServer = new Horizon.Server(horizonUrl);
  }

  onModuleInit() {
    this.logger.log('Stellar Service Initialized');
    const network = this.configService.get<string>('stellar.network');
    this.logger.log(`Target Network: ${network}`);
  }

  getRpcServer() {
    return this.rpcServer;
  }

  getHorizonServer() {
    return this.horizonServer;
  }

  getNetworkPassphrase(): string {
    const network = this.configService.get<string>('stellar.network');
    return network === 'mainnet' ? Networks.PUBLIC : Networks.TESTNET;
  }

  async getHealth() {
    try {
      const health = await this.rpcServer.getHealth();
      return health;
    } catch (error) {
      this.logger.error('Failed to get Stellar RPC health', error);
      throw error;
    }
  }

  // Placeholder for Soroban contract interaction
  async queryContract(contractId: string, method: string, args: xdr.ScVal[]) {
    // Implementation for querying smart contracts
    this.logger.log(`Querying contract ${contractId}, method ${method}`);
    // return this.rpcServer.simulateTransaction(...)
  }
}
