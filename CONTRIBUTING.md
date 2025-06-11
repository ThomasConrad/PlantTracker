# Contributing to Planty

Thank you for your interest in contributing to Planty! This document provides guidelines and information for contributors.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Issue Reporting](#issue-reporting)

## 🤝 Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. Please treat all community members with respect and create a welcoming environment for everyone.

## 🚀 Getting Started

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- PostgreSQL 14+
- Git

### Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/plant-tracker.git
   cd plant-tracker
   ```
3. Set up the development environment following the [README](README.md#quick-start)

## 🔄 Development Workflow

### Branch Naming

Use descriptive branch names:
- `feature/add-plant-search`
- `fix/photo-upload-bug`
- `docs/update-api-documentation`
- `refactor/improve-auth-flow`

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions/changes
- `chore`: Maintenance tasks

**Examples:**
```
feat(frontend): add plant search functionality
fix(backend): resolve photo upload timeout issue
docs(api): update authentication endpoints
```

## 🎨 Code Style

### Frontend (TypeScript/SolidJS)

- Use TypeScript strict mode
- Follow ESLint configuration
- Use Prettier for formatting
- Prefer functional components
- Use proper TypeScript types (avoid `any`)

```typescript
// Good
interface PlantProps {
  plant: Plant;
  onUpdate: (plant: Plant) => void;
}

const PlantCard: Component<PlantProps> = (props) => {
  // Component implementation
};

// Avoid
const PlantCard = (props: any) => {
  // Implementation
};
```

### Backend (Rust)

- Use `cargo fmt` for formatting
- Follow `cargo clippy` suggestions
- Use proper error handling with `Result<T, E>`
- Write comprehensive documentation

```rust
// Good
/// Creates a new plant for the authenticated user
pub async fn create_plant(
    State(state): State<AppState>,
    Json(request): Json<CreatePlantRequest>,
) -> Result<Json<PlantResponse>, ApiError> {
    // Implementation
}

// Avoid
pub async fn create_plant(data: Json<CreatePlantRequest>) -> Json<PlantResponse> {
    // Implementation without proper error handling
}
```

## 🧪 Testing

### Frontend Testing

```bash
cd frontend
npm run test           # Unit tests
npm run test:e2e      # E2E tests
npm run test:coverage # Coverage report
```

### Backend Testing

```bash
cd backend
cargo test            # All tests
cargo test --release  # Optimized tests
```

### Test Requirements

- Write tests for new features
- Maintain or improve test coverage
- Include both unit and integration tests
- Test error conditions

## 📤 Submitting Changes

### Pull Request Process

1. **Create a feature branch** from `main`
2. **Make your changes** following the code style guidelines
3. **Add tests** for new functionality
4. **Update documentation** if needed
5. **Run the test suite** to ensure everything passes
6. **Submit a pull request** with a clear description

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Tests added/updated
- [ ] All tests pass
- [ ] Manual testing completed

## Screenshots (if applicable)
[Add screenshots for UI changes]

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

### Review Process

1. Automated checks must pass
2. At least one maintainer review required
3. All feedback addressed
4. Final approval and merge

## 🐛 Issue Reporting

### Bug Reports

Use the bug report template and include:

- **Environment** (OS, browser, versions)
- **Steps to reproduce**
- **Expected behavior**
- **Actual behavior**
- **Screenshots** (if applicable)
- **Error messages** or logs

### Feature Requests

Use the feature request template and include:

- **Problem statement**
- **Proposed solution**
- **Alternative solutions considered**
- **Additional context**

### Security Issues

**Do not** open public issues for security vulnerabilities.
Email security@planttracker.com instead.

## 📝 Documentation

### When to Update Documentation

- New features or API changes
- Configuration changes
- Installation/setup changes
- Breaking changes

### Documentation Types

- **README.md** - Overview and quick start
- **API documentation** - OpenAPI spec
- **Code comments** - Complex logic explanation
- **Examples** - Usage examples

## 🏗️ Architecture Guidelines

### Frontend Architecture

- **Components** should be reusable and focused
- **Stores** manage application state
- **API client** handles all backend communication
- **Types** ensure type safety throughout

### Backend Architecture

- **Handlers** process HTTP requests
- **Models** define data structures
- **Services** contain business logic
- **Middleware** handles cross-cutting concerns

### Database

- Use migrations for schema changes
- Follow naming conventions
- Index performance-critical queries
- Maintain referential integrity

## 🚀 Release Process

1. Update version numbers
2. Update CHANGELOG.md
3. Create release branch
4. Final testing
5. Tag release
6. Deploy to staging
7. Deploy to production
8. Announce release

## 📞 Getting Help

- **Discord**: [Join our community](https://discord.gg/planttracker)
- **Discussions**: [GitHub Discussions](https://github.com/username/plant-tracker/discussions)
- **Email**: contribute@planttracker.com

## 🙏 Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- Discord announcements

Thank you for contributing to Plant Tracker! 🌱