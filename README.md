# 🪴 Planty

A full-stack Progressive Web Application for tracking plant care, growth, and health. Built with SolidJS frontend and Rust backend.

![Planty](https://img.shields.io/badge/status-in%20development-yellow)
![Frontend](https://img.shields.io/badge/frontend-SolidJS-blue)
![Backend](https://img.shields.io/badge/backend-Rust-orange)
![Database](https://img.shields.io/badge/database-PostgreSQL-green)

## 🌟 Overview

Planty is a comprehensive plant care management system that helps users:
- Track watering and fertilizing schedules with automated reminders
- Monitor custom plant metrics (height, pH, leaf count, etc.)
- Document growth with photo galleries and progress tracking
- Manage multiple plants with personalized care routines
- Access via installable Progressive Web App on any device
- Control access with invite-based registration system

## ✨ Current Features

### 🔐 Authentication & Access Control
- **Invite System**: Registration requires invite codes with usage limits
- **Role-Based Access**: Admin, moderator, and user roles with permissions
- **Admin Dashboard**: User management, system statistics, and settings
- **Session Authentication**: Secure database-backed session management

### 🏠 Plant Management
- **Plant Gallery**: Grid view with care status indicators and search
- **Plant Profiles**: Detailed view with care history and photo timeline
- **Custom Schedules**: Personalized watering and fertilizing intervals
- **Custom Metrics**: Track any measurement (height, pH, etc.)

### 💧 Care Tracking
- **Quick Actions**: One-click watering/fertilizing with timestamps
- **Detailed Logging**: Custom entries with notes and photo documentation
- **Care Status**: Visual overdue/due indicators with smart scheduling
- **Activity Timeline**: Complete care history with filtering

### 📸 Photo Management
- **Growth Documentation**: Upload and organize plant photos with AVIF compression
- **Gallery View**: Chronological photo timeline with thumbnail previews
- **Progress Tracking**: Set preview photos for visual growth comparison

### 📅 Calendar & Reminders
- **Calendar View**: Interactive calendar showing care schedules
- **iCalendar Export**: Subscribe to care reminders in external calendar apps
- **Smart Filtering**: Filter by plant, care type, and date ranges

### 📱 Progressive Web App
- **Mobile-First**: Optimized responsive design for all devices
- **Installable**: Add to home screen with native app experience
- **Dark Mode**: System-adaptive dark/light theme support
- **Offline Support**: Basic offline functionality for essential features

## 📈 Development Timeline

### June 2025
- **June 19**: 🔐 Complete invite system with admin dashboard and role-based permissions
- **June 16**: 🎨 Major UI/UX improvements with mobile-first redesign
- **June 11**: 📅 Calendar functionality with iCalendar subscriptions
- **June 10**: 🔍 Search, sorting, and mobile camera integration
- **June 9**: 🔑 Session-based authentication and comprehensive testing

### Earlier Development
- **June 8**: 🏗️ Project foundation with error handling and deployment support
- **June 8**: 🚀 Initial project setup with SolidJS frontend and Rust backend

## 🚧 Upcoming Features (Priority Order)

### High Priority
1. **Google Tasks Integration** - Sync plant care reminders to Google Tasks
2. **Email Notifications** - Automated care reminder emails
3. **Plant Templates** - Pre-configured care schedules for common plants
4. **Data Export/Import** - Backup and restore plant data

### Medium Priority
5. **Mobile App** - Native iOS/Android apps using Capacitor
6. **Plant Sharing** - Share plant profiles and care tips with other users
7. **Advanced Analytics** - Growth charts, care pattern analysis
8. **Weather Integration** - Adjust care schedules based on local weather

### Low Priority
9. **Plant Disease Detection** - AI-powered plant health analysis
10. **Community Features** - Plant forums and expert advice
11. **IoT Integration** - Connect soil moisture sensors and smart watering systems
12. **Multi-language Support** - Internationalization for global users

## 🎯 RELEASE TARGET

**Version 1.0 Release: Q3 2025 (September 2025)**

The first stable release will include all High Priority features and core functionality for comprehensive plant care management. This target allows for proper testing, documentation, and refinement of the invite system and admin controls.

## 🏗️ Architecture

```
planty/
├── frontend/                 # SolidJS PWA frontend
│   ├── src/
│   │   ├── api/              # API client and OpenAPI spec
│   │   ├── components/       # Reusable UI components
│   │   ├── routes/           # Page components
│   │   ├── stores/           # State management
│   │   └── types/            # TypeScript definitions
│   └── vite.config.ts        # Vite configuration
│
├── backend/                  # Rust API backend
│   ├── src/
│   │   ├── handlers/         # HTTP request handlers
│   │   ├── models/           # Data models and authentication
│   │   ├── database/         # Database operations
│   │   └── middleware/       # Authentication and CORS
│   ├── migrations/           # Database schema migrations
│   └── tests/               # Comprehensive test suite
│
└── docker-compose.yml       # Local development setup
```

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/your-username/planty.git
cd planty

# Start development environment
just dev

# Or start services individually
just backend    # Start Rust API server
just frontend   # Start SolidJS dev server
just db         # Start PostgreSQL database
```

Visit `http://localhost:5173` and register with an invite code to start tracking your plants!