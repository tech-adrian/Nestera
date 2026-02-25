import { Injectable, NotFoundException } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { User } from './entities/user.entity';
import { UpdateUserDto } from './dto/update-user.dto';

@Injectable()
export class UserService {
  constructor(
    @InjectRepository(User)
    private readonly userRepository: Repository<User>,
  ) {}

  async findById(id: string) {
    const user = await this.userRepository.findOne({
      where: { id },
      select: ['id', 'email', 'name', 'bio', 'createdAt', 'updatedAt'],
    });

    if (!user) {
      throw new NotFoundException('User not found');
    }

    return user;
  }

  async update(id: string, dto: UpdateUserDto) {
    await this.findById(id);

    await this.userRepository.update(id, dto);

    return this.findById(id);
  }

  async findByEmail(email: string) {
    const user = await this.userRepository.findOne({
      where: { email },
    });
    return user;
  }

  async findByPublicKey(publicKey: string) {
    const user = await this.userRepository.findOne({
      where: { publicKey },
    });
    return user;
  }

  async create(data: Partial<User>) {
    const newEntity = this.userRepository.create(data);
    const savedUser = await this.userRepository.save(newEntity);

    // Return with only selected fields to match old behavior
    return this.findById(savedUser.id);
  }

  async updateAvatar(userId: string, avatarUrl: string) {
    await this.findById(userId);

    await this.userRepository.update(userId, { avatarUrl });

    return this.findById(userId);
  }

  async remove(id: string) {
    const user = await this.findById(id);

    await this.userRepository.remove(user);

    return { message: 'User deleted successfully' };
  }
}
