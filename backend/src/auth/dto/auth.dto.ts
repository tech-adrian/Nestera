import { IsEmail, IsString, MinLength, MaxLength } from 'class-validator';

export class RegisterDto {
  @IsEmail()
  email: string;

  @IsString()
  @MinLength(8)
  @MaxLength(32)
  password: string;

  @IsString()
  name?: string;
}

export class LoginDto {
  @IsEmail()
  email: string;

  @IsString()
  password: string;
}

export class GetNonceDto {
  @IsString()
  publicKey: string;
}

export class VerifySignatureDto {
  @IsString()
  publicKey: string;

  @IsString()
  signature: string;
}
