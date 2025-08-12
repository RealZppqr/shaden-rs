# Shaden-RS Discord Bot

A production-grade Discord bot for managing Pterodactyl hosting services, built with Rust.

## Features

- **User Management**: Registration, coin balance, resource tracking
- **Server Management**: Create, manage, and control Pterodactyl servers
- **Economy System**: Coins, store, coupons, and payments via Stripe
- **Queue System**: Prevents overload with Redis-based job queuing
- **Admin Controls**: Comprehensive admin commands for management
- **Join Rewards**: Earn coins by joining partner Discord servers

## Prerequisites

- Rust 1.70+ (2021 edition)
- MongoDB database
- Redis server
- Discord bot application
- Pterodactyl panel with admin API access
- Stripe account (for payments)

## Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/RealZppqr/shaden-rs
   cd shaden-rs
   ```

2. **Install dependencies**
   ```bash
   cargo build --release
   ```

3. **Configure environment**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. **Set up databases**
   - Start MongoDB and Redis services
   - The bot will automatically create collections on first run

5. **Configure Discord bot**
   - Create a Discord application at https://discord.com/developers/applications
   - Create a bot user and copy the token
   - Add bot to your server with appropriate permissions

## Configuration

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `DISCORD_TOKEN` | Discord bot token | Yes |
| `DISCORD_APP_ID` | Discord application ID | Yes |
| `PTERODACTYL_URL` | Pterodactyl panel URL | Yes |
| `PTERODACTYL_API_KEY` | Pterodactyl admin API key | Yes |
| `MONGODB_URI` | MongoDB connection string | Yes |
| `REDIS_URI` | Redis connection string | Yes |
| `STRIPE_SECRET_KEY` | Stripe secret key | Yes |
| `STRIPE_PUBLIC_KEY` | Stripe public key | Yes |
| `AFK_PAGE_URL` | URL for AFK earning page | No |
| `LINKVERTISE_VERIFY_URL` | URL for Linkvertise verification | No |
| `ADMIN_DISCORD_IDS` | Comma-separated admin user IDs | No |

### Discord Permissions

The bot requires the following permissions:
- Send Messages
- Use Slash Commands
- Embed Links
- Read Message History

## Usage

### User Commands

- `/login` - Register or login to the bot
- `/coins balance` - Check your coin balance
- `/coins earn afk` - Earn coins via AFK page
- `/coins gift <user> <amount>` - Gift coins to another user
- `/servers list` - List your servers
- `/servers create <plan> <name>` - Create a new server
- `/store list` - View available store items
- `/store buy <item>` - Purchase store items

### Admin Commands

- `/admin coins set <user> <amount>` - Set user's coins
- `/admin coupons create <code> <coins>` - Create a coupon
- `/admin stats` - View system statistics

## Development

### Project Structure

```
src/
├── main.rs              # Bot entry point
├── config.rs            # Configuration management
├── errors.rs            # Error types
├── models/              # Data models
│   ├── user.rs
│   ├── server.rs
│   ├── coupon.rs
│   └── order.rs
├── services/            # External services
│   ├── db.rs            # Database operations
│   ├── pterodactyl.rs   # Pterodactyl API
│   ├── stripe.rs        # Stripe integration
│   └── queue.rs         # Queue management
└── commands/            # Slash command handlers
    ├── coins.rs
    ├── servers.rs
    ├── store.rs
    ├── admin.rs
    └── join_rewards.rs
```

### Running in Development

```bash
# Set up environment
cp .env.example .env
# Edit .env with your development configuration

# Run with cargo
cargo run

# Or with logging
RUST_LOG=info cargo run
```

### Building for Production

```bash
cargo build --release
./target/release/shaden-rs
```

## Extending the Bot

### Adding New Commands

1. Create a new module in `src/commands/`
2. Implement command handlers following existing patterns
3. Register commands in `src/commands/mod.rs`
4. Add command registration in the `register_commands` function

### Adding New Database Models

1. Create model struct in `src/models/`
2. Implement serialization with `serde`
3. Add database operations in `src/services/db.rs`
4. Export from `src/models/mod.rs`

### Adding New External Services

1. Create service module in `src/services/`
2. Implement async client with proper error handling
3. Add configuration options in `src/config.rs`
4. Export from `src/services/mod.rs`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For support and questions:
- Create an issue on GitHub
- Check the [documentation](https://rs.shadendash.com)

## Roadmap

- [ ] Web dashboard integration
- [ ] Advanced server templates
- [ ] Automated backups
- [ ] Resource usage monitoring
- [ ] Multi-language support
- [ ] Plugin system
