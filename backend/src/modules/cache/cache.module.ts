import { Module } from '@nestjs/common';
import { CacheModule } from '@nestjs/cache-manager';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { redisStore } from 'cache-manager-redis-yet';

@Module({
  imports: [
    CacheModule.registerAsync({
      isGlobal: true,
      imports: [ConfigModule],
      inject: [ConfigService],
      useFactory: async (configService: ConfigService) => {
        const redisUrl = configService.get<string>('redis.url');

        if (redisUrl) {
          return {
            store: await redisStore({ url: redisUrl, ttl: 30000 }),
          };
        }

        return { ttl: 30000 };
      },
    }),
  ],
})
export class RedisCacheModule {}
