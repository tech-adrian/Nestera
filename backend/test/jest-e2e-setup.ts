// Set environment variables for e2e tests
process.env.NODE_ENV = 'test';
process.env.PORT = '3001';
process.env.DATABASE_URL = 'postgresql://user:pass@localhost:5432/test';
process.env.JWT_SECRET = 'super-secret-key-for-testing-purposes_long_enough';
process.env.JWT_EXPIRATION = '24h';
process.env.STELLAR_NETWORK = 'testnet';
process.env.SOROBAN_RPC_URL = 'https://soroban-testnet.stellar.org';
process.env.HORIZON_URL = 'https://horizon-testnet.stellar.org';
process.env.CONTRACT_ID = 'CBWHJPY37LHMQJ726PN26YLWWP3CQQ7T7NYJT2KMXNQQPLR7QDNVWZGK';
process.env.STELLAR_WEBHOOK_SECRET = 'test_webhook_secret_key_123456_extra';
process.env.REDIS_URL = 'redis://localhost:6379';
process.env.MAIL_HOST = 'localhost';
process.env.MAIL_PORT = '1025';
process.env.MAIL_FROM = 'test@example.com';
