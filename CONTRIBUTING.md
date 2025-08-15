# Contributing to VX0 Network

Thank you for your interest in contributing to VX0 Network! This project aims to create a censorship-resistant, decentralized internet infrastructure.

## 🚀 Quick Start for Contributors

1. **Fork the repository**
2. **Clone your fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/vx0net-daemon.git
   cd vx0net-daemon
   ```
3. **Set up development environment**:
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install Docker
   curl -fsSL https://get.docker.com | sh
   
   # Build the project
   cargo build
   ```

## 🎯 Ways to Contribute

### 🐛 Bug Reports
- Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md)
- Include steps to reproduce
- Provide system information (OS, Docker version, etc.)
- Include relevant logs

### ✨ Feature Requests
- Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md)
- Explain the use case and benefit
- Consider backward compatibility

### 📝 Documentation
- Improve installation guides
- Add troubleshooting sections
- Translate to other languages
- Create video tutorials

### 🌐 Network Infrastructure
- Host backbone/regional nodes
- Contribute to bootstrap registry
- Improve auto-discovery mechanisms
- Performance optimizations

### 💻 Code Contributions
- Fix bugs and issues
- Implement new features
- Improve security
- Add tests
- Performance optimizations

## 🛠️ Development Guidelines

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Run Clippy for linting (`cargo clippy`)
- Write meaningful commit messages
- Add tests for new functionality

### Testing
```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Test Docker deployment
./scripts/docker-deploy.sh edge
```

### Documentation
- Update relevant `.md` files
- Add inline code documentation
- Update API documentation if applicable

## 🔒 Security

### Reporting Security Issues
- **Do NOT** open public issues for security vulnerabilities
- Email: security@vx0.network
- Use GPG key: [security-public-key.asc](security-public-key.asc)

### Security Guidelines
- Never commit secrets or private keys
- Use secure defaults in configurations
- Validate all inputs
- Follow cryptographic best practices

## 📋 Pull Request Process

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**:
   - Write clean, documented code
   - Add tests if applicable
   - Update documentation

3. **Test thoroughly**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ./scripts/docker-deploy.sh edge  # Test deployment
   ```

4. **Commit with clear messages**:
   ```bash
   git commit -m "feat: add auto-discovery for regional nodes"
   ```

5. **Push and create PR**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Fill out PR template** with:
   - Description of changes
   - Testing performed
   - Breaking changes (if any)
   - Related issues

## 🌍 Community Guidelines

### Code of Conduct
We follow the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).

### Communication
- **Discord**: [discord.gg/vx0network](https://discord.gg/vx0network)
- **GitHub Discussions**: For design discussions
- **Issues**: For bugs and feature requests
- **Email**: team@vx0.network

### Being Respectful
- Be welcoming to newcomers
- Respect different perspectives
- Provide constructive feedback
- Help others learn and grow

## 🏗️ Architecture Overview

```
VX0 Network Daemon
├── src/
│   ├── main.rs              # CLI interface
│   ├── lib.rs               # Core library
│   ├── config/              # Configuration management
│   ├── node/                # Node management & peering
│   ├── network/             # BGP, IKE, DNS protocols
│   └── services/            # Service discovery & monitoring
├── k8s/                     # Kubernetes deployment
├── docker-compose.yml       # Docker orchestration
├── install-vx0.sh          # One-command installer
└── scripts/                 # Management scripts
```

## 🎨 Areas Needing Help

### High Priority
- [ ] Complete BGP protocol implementation
- [ ] Full IKE v2 security implementation
- [ ] Network interface (TUN/TAP) integration
- [ ] Performance optimization
- [ ] Mobile app development

### Medium Priority
- [ ] Web-based management interface
- [ ] Monitoring and alerting improvements
- [ ] Additional platform support
- [ ] Load balancing improvements
- [ ] Documentation translations

### Low Priority
- [ ] Advanced routing features
- [ ] Custom protocol development
- [ ] Research and experimentation
- [ ] Community tools and utilities

## 🏷️ Commit Message Format

Use conventional commits format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

Examples:
```
feat(network): add auto-discovery for backbone nodes
fix(docker): resolve container startup race condition
docs(install): improve Windows installation guide
```

## 📦 Release Process

1. **Version Bump**: Update `Cargo.toml` version
2. **Changelog**: Update `CHANGELOG.md`
3. **Tag Release**: `git tag v1.2.3`
4. **GitHub Release**: Create release with binaries
5. **Docker Images**: Publish to container registry
6. **Documentation**: Update installation guides

## 🤝 Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file
- GitHub contributors page
- Annual community report
- Social media acknowledgments

## ❓ Questions?

- **General**: [GitHub Discussions](https://github.com/vx0net/daemon/discussions)
- **Technical**: [Discord #dev channel](https://discord.gg/vx0network)
- **Private**: team@vx0.network

---

Thank you for helping build a free and open internet! 🌐✨
