import { Controller, Post, Get, Body, HttpCode, HttpStatus, Query } from '@nestjs/common';
import { AuthService } from './auth.service';
import { RegisterDto, LoginDto, GetNonceDto, VerifySignatureDto } from './dto/auth.dto';

@Controller('auth')
export class AuthController {
  constructor(private readonly authService: AuthService) {}

  @Post('register')
  register(@Body() dto: RegisterDto) {
    return this.authService.register(dto);
  }

  @Post('login')
  @HttpCode(HttpStatus.OK)
  login(@Body() dto: LoginDto) {
    return this.authService.login(dto);
  }

  @Get('nonce')
  getNonce(@Query('publicKey') publicKey: string) {
    return this.authService.generateNonce(publicKey);
  }

  @Post('verify-signature')
  @HttpCode(HttpStatus.OK)
  verifySignature(@Body() dto: VerifySignatureDto) {
    return this.authService.verifySignature(dto);
  }
}
