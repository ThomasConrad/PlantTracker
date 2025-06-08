# 🌱 Plant Tracker

A full-stack Progressive Web Application for tracking plant care, growth, and health. Built with SolidJS frontend and Rust backend.

![Plant Tracker](https://img.shields.io/badge/status-in%20development-yellow)
![Frontend](https://img.shields.io/badge/frontend-SolidJS-blue)
![Backend](https://img.shields.io/badge/backend-Rust-orange)
![Database](https://img.shields.io/badge/database-PostgreSQL-green)

## 📋 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [Quick Start](#quick-start)
- [Development](#development)
- [API Documentation](#api-documentation)
- [Deployment](#deployment)
- [Contributing](#contributing)

## 🌟 Overview

Plant Tracker is a comprehensive plant care management system that helps users:
- Track watering and fertilizing schedules
- Monitor custom plant metrics (height, leaf count, etc.)
- Document growth with photo galleries
- Manage multiple plants with personalized care routines
- Access the app on any device (PWA support)

## ✨ Features

### 🏠 Plant Management
- **Plant Gallery**: Grid view with care status indicators
- **Add/Edit Plants**: Custom watering and fertilizing schedules
- **Custom Metrics**: Track any measurement (height, pH, etc.)
- **Plant Profiles**: Detailed view with care history

### 💧 Care Tracking
- **Quick Actions**: One-click watering/fertilizing
- **Custom Entries**: Detailed tracking with notes
- **Care Status**: Visual overdue/due indicators
- **History Timeline**: Complete care activity log

### 📸 Photo Management
- **Growth Documentation**: Upload and organize plant photos
- **Gallery View**: Chronological photo timeline
- **Progress Tracking**: Visual growth comparison

### 📱 Progressive Web App
- **Mobile-First**: Optimized for mobile devices
- **Installable**: Add to home screen
- **Offline Support**: Basic offline functionality
- **Responsive**: Works on all screen sizes

### 🔐 Authentication & Security
- **User Accounts**: Secure registration and login
- **JWT Authentication**: Stateless token-based auth
- **Data Privacy**: Each user's plants are private

## 🛠 Tech Stack

### Frontend
- **[SolidJS](https://www.solidjs.com/)** - Reactive JavaScript framework
- **[TypeScript](https://www.typescriptlang.org/)** - Type-safe JavaScript
- **[Tailwind CSS](https://tailwindcss.com/)** - Utility-first CSS framework
- **[Kobalte](https://kobalte.dev/)** - Accessible UI components
- **[Vite](https://vitejs.dev/)** - Fast build tool and dev server

### Backend
- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[Axum](https://github.com/tokio-rs/axum)** - Web application framework
- **[SQLx](https://github.com/launchbadge/sqlx)** - Async SQL toolkit
- **[PostgreSQL](https://www.postgresql.org/)** - Relational database
- **[JWT](https://jwt.io/)** - JSON Web Tokens for authentication

### DevOps & Tools
- **[Docker](https://www.docker.com/)** - Containerization
- **[Git](https://git-scm.com/)** - Version control
- **[GitHub Actions](https://github.com/features/actions)** - CI/CD

## 📁 Project Structure

```
plant-tracker/
├── frontend/                 # SolidJS PWA frontend
│   ├── src/
│   │   ├── api/              # API client and OpenAPI spec
│   │   ├── components/       # Reusable UI components
│   │   ├── routes/           # Page components
│   │   ├── stores/           # State management
│   │   ├── types/            # TypeScript definitions
│   │   └── utils/            # Utility functions
│   ├── public/               # Static assets
│   ├── package.json          # Frontend dependencies
│   └── vite.config.ts        # Vite configuration
│
├── backend/                  # Rust API backend
│   ├── src/
│   │   ├── handlers/         # HTTP request handlers
│   │   ├── models/           # Data models and types
│   │   ├── middleware/       # Custom middleware
│   │   └── utils/            # Helper functions
│   ├── migrations/           # Database migrations
│   ├── Cargo.toml           # Rust dependencies
│   └── .env.example         # Environment template
│
├── docs/                    # Documentation
├── docker-compose.yml       # Local development setup
└── README.md               # This file
```

## 🚀 Quick Start

### Prerequisites
- **Node.js** 18+ and npm
- **Rust** 1.70+ and Cargo
- **PostgreSQL** 14+
- **Git**

### 1. Clone the Repository
```bash
git clone https://github.com/yourusername/plant-tracker.git
cd plant-tracker
```

### 2. Set Up the Database
```bash
# Install PostgreSQL and create database
createdb plant_tracker

# Or use Docker
docker run --name plant-tracker-db \
  -e POSTGRES_DB=plant_tracker \
  -e POSTGRES_USER=username \
  -e POSTGRES_PASSWORD=password \
  -p 5432:5432 -d postgres:14
```

### 3. Configure Environment
```bash
# Backend configuration
cp backend/.env.example backend/.env
# Edit backend/.env with your database credentials

# Frontend configuration
cp frontend/.env.example frontend/.env
# Edit frontend/.env with your API URL
```

### 4. Start Development (Option 1: Separate servers)
```bash
# Start backend API server
cd backend && cargo run

# In another terminal, start frontend dev server
cd frontend && npm run dev
```

### 5. Start Development (Option 2: Using Just)
```bash
# Install Just (if not already installed)
# macOS: brew install just
# Other platforms: https://github.com/casey/just

# Start both frontend and backend
just dev

# Or start them individually
just backend    # Start only backend
just frontend   # Start only frontend
```

### 6. Production Mode
```bash
# Build frontend and serve from backend
just build
just backend
# Full-stack app available at http://localhost:3000
```

### 7. Open the App
- **Development**: `http://localhost:5173` (frontend dev server)
- **Production**: `http://localhost:3000` (backend serves frontend)

## 🔧 Development

### Frontend Development
```bash
cd frontend

# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Run tests
npm run test

# Type checking
npm run typecheck

# Linting
npm run lint
```

### Backend Development
```bash
cd backend

# Run in development mode
cargo run

# Run with auto-reload
cargo watch -x run

# Run tests
cargo test

# Check code
cargo check

# Format code
cargo fmt

# Run clippy (linter)
cargo clippy
```

### Database Migrations
```bash
cd backend

# Install sqlx-cli
cargo install sqlx-cli

# Run migrations
sqlx migrate run

# Create new migration
sqlx migrate add create_plants_table
```

### Environment Variables

#### Backend (.env)
```bash
DATABASE_URL=postgresql://username:password@localhost/plant_tracker
PORT=3000
JWT_SECRET=your-super-secret-key
CORS_ORIGIN=http://localhost:5173
UPLOAD_DIR=./uploads
RUST_LOG=plant_tracker_api=debug
```

#### Frontend (.env)
```bash
VITE_API_URL=http://localhost:3000/v1
```

## 📚 API Documentation

The backend implements a RESTful API following the OpenAPI 3.0 specification. Key endpoints include:

### Authentication
- `POST /v1/auth/register` - User registration
- `POST /v1/auth/login` - User login
- `GET /v1/auth/me` - Get current user

### Plants
- `GET /v1/plants` - List user's plants
- `POST /v1/plants` - Create new plant
- `GET /v1/plants/{id}` - Get plant details
- `PUT /v1/plants/{id}` - Update plant
- `DELETE /v1/plants/{id}` - Delete plant

### Photos
- `GET /v1/plants/{id}/photos` - List plant photos
- `POST /v1/plants/{id}/photos` - Upload photo
- `DELETE /v1/plants/{id}/photos/{photoId}` - Delete photo

### Tracking
- `GET /v1/plants/{id}/entries` - List tracking entries
- `POST /v1/plants/{id}/entries` - Create tracking entry
- `PUT /v1/plants/{id}/entries/{entryId}` - Update entry
- `DELETE /v1/plants/{id}/entries/{entryId}` - Delete entry

Complete API documentation is available in `frontend/src/api/openapi.yaml`.

## 🐳 Docker Development

Use Docker Compose for easy local development:

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

## 🚀 Deployment

### Frontend Deployment
The frontend can be deployed to any static hosting service:

```bash
cd frontend
npm run build
# Deploy the dist/ folder
```

**Recommended platforms:**
- [Vercel](https://vercel.com/) - Zero config deployment
- [Netlify](https://netlify.com/) - Great for PWAs
- [GitHub Pages](https://pages.github.com/) - Free static hosting

### Backend Deployment
The Rust backend can be deployed to various platforms:

**Recommended platforms:**
- [Railway](https://railway.app/) - Easy Rust deployment
- [Fly.io](https://fly.io/) - Global edge deployment
- [Heroku](https://heroku.com/) - Simple platform-as-a-service
- [DigitalOcean App Platform](https://www.digitalocean.com/products/app-platform/) - Managed containers

### Database
For production, use a managed PostgreSQL service:
- [Railway PostgreSQL](https://railway.app/)
- [Supabase](https://supabase.com/)
- [PlanetScale](https://planetscale.com/)
- [Neon](https://neon.tech/)

## 🧪 Testing

### Frontend Testing
```bash
cd frontend
npm run test           # Run unit tests
npm run test:e2e      # Run end-to-end tests
npm run test:coverage # Generate coverage report
```

### Backend Testing
```bash
cd backend
cargo test            # Run all tests
cargo test --release  # Run optimized tests
cargo test -- --nocapture  # Show println! output
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Workflow
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`npm test` and `cargo test`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Code Style
- **Frontend**: Prettier + ESLint configuration
- **Backend**: `cargo fmt` + `cargo clippy`
- **Commits**: Follow [Conventional Commits](https://www.conventionalcommits.org/)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [SolidJS](https://www.solidjs.com/) for the reactive frontend framework
- [Axum](https://github.com/tokio-rs/axum) for the excellent Rust web framework
- [Tailwind CSS](https://tailwindcss.com/) for the utility-first CSS framework
- [Kobalte](https://kobalte.dev/) for accessible UI components

## 📞 Support

If you have any questions or need help:

- 📧 Email: support@planttracker.com
- 🐛 Issues: [GitHub Issues](https://github.com/yourusername/plant-tracker/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/yourusername/plant-tracker/discussions)

---

**Happy plant tracking! 🌱🪴🌿**