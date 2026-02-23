import * as Joi from 'joi';

export const envValidationSchema = Joi.object({
  NODE_ENV: Joi.string().valid('development', 'production', 'test').required(),

  PORT: Joi.number().port().default(3001).required(),

  DATABASE_URL: Joi.string().uri().required(),
  JWT_SECRET: Joi.string().min(10).required(),
  JWT_EXPIRATION: Joi.string().required(),
  STELLAR_NETWORK: Joi.string().valid('testnet', 'mainnet').default('testnet'),
  SOROBAN_RPC_URL: Joi.string().uri().required(),
  HORIZON_URL: Joi.string().uri().required(),
  CONTRACT_ID: Joi.string().required(),
  REDIS_URL: Joi.string().uri().optional(),

  MAIL_HOST: Joi.string().optional(),
  MAIL_PORT: Joi.number().port().default(587).optional(),
  MAIL_USER: Joi.string().optional(),
  MAIL_PASS: Joi.string().optional(),
  MAIL_FROM: Joi.string().optional(),
});
