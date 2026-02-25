import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { MedicalClaim, ClaimStatus } from './entities/medical-claim.entity';
import { CreateClaimDto } from './dto/create-claim.dto';

@Injectable()
export class ClaimsService {
  constructor(
    @InjectRepository(MedicalClaim)
    private readonly claimsRepository: Repository<MedicalClaim>,
  ) {}

  async createClaim(createClaimDto: CreateClaimDto): Promise<MedicalClaim> {
    const claim = this.claimsRepository.create({
      ...createClaimDto,
      patientDateOfBirth: new Date(createClaimDto.patientDateOfBirth),
      status: ClaimStatus.PENDING,
    });

    return await this.claimsRepository.save(claim);
  }

  async findAll(): Promise<MedicalClaim[]> {
    return await this.claimsRepository.find({ order: { createdAt: 'DESC' } });
  }

  async findOne(id: string): Promise<MedicalClaim | null> {
    return await this.claimsRepository.findOneBy({ id });
  }
}
