export default () => ({
  port: parseInt(process.env.PORT || '3001', 10),
  database: {
    url: process.env.DATABASE_URL,
  },
  jwt: {
    secret: process.env.JWT_SECRET,
    expiration: process.env.JWT_EXPIRATION,
  },
  stellar: {
    network: process.env.STELLAR_NETWORK || 'testnet',
    rpcUrl: process.env.SOROBAN_RPC_URL,
    horizonUrl: process.env.HORIZON_URL,
    // Fallback RPC URLs (comma-separated, in priority order)
    rpcFallbackUrls: process.env.SOROBAN_RPC_FALLBACK_URLS?.split(',').map(
      (url) => url.trim(),
    ) || [],
    // Fallback Horizon URLs (comma-separated, in priority order)
    horizonFallbackUrls: process.env.HORIZON_FALLBACK_URLS?.split(',').map(
      (url) => url.trim(),
    ) || [],
    contractId: process.env.CONTRACT_ID,
    webhookSecret: process.env.STELLAR_WEBHOOK_SECRET,
    // Retry configuration
    rpcMaxRetries: parseInt(process.env.RPC_MAX_RETRIES || '3', 10),
    rpcRetryDelay: parseInt(process.env.RPC_RETRY_DELAY || '1000', 10),
    rpcTimeout: parseInt(process.env.RPC_TIMEOUT || '10000', 10),
  },
  redis: {
    url: process.env.REDIS_URL,
  },
  mail: {
    host: process.env.MAIL_HOST,
    port: parseInt(process.env.MAIL_PORT || '587', 10),
    user: process.env.MAIL_USER,
    pass: process.env.MAIL_PASS,
    from: process.env.MAIL_FROM || '"Nestera" <noreply@nestera.io>',
  },
  hospital: {
    endpoints: {
      // Hospital endpoints from environment variables
      // Format: HOSPITAL_<ID>_ENDPOINT
      ...(process.env.HOSPITAL_1_ENDPOINT && { 'hospital-1': process.env.HOSPITAL_1_ENDPOINT }),
      ...(process.env.HOSPITAL_2_ENDPOINT && { 'hospital-2': process.env.HOSPITAL_2_ENDPOINT }),
      ...(process.env.HOSPITAL_3_ENDPOINT && { 'hospital-3': process.env.HOSPITAL_3_ENDPOINT }),
    },
    maxRetries: parseInt(process.env.HOSPITAL_MAX_RETRIES || '3', 10),
    retryDelay: parseInt(process.env.HOSPITAL_RETRY_DELAY || '1000', 10),
    requestTimeout: parseInt(process.env.HOSPITAL_REQUEST_TIMEOUT || '10000', 10),
    circuitBreakerThreshold: parseInt(process.env.HOSPITAL_CIRCUIT_BREAKER_THRESHOLD || '5', 10),
    circuitBreakerTimeout: parseInt(process.env.HOSPITAL_CIRCUIT_BREAKER_TIMEOUT || '60000', 10),
  },
});
