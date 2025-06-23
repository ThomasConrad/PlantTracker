Backend Functionality Not Yet Implemented

  Based on my analysis of the frontend code and backend handlers, here are the key functionalities that are not yet implemented in the backend:

  1. User Profile Management

  - Update user profile (PUT /auth/profile or similar)
    - Change name, email
    - Currently only placeholders in UserSettingsPage
  - Change password (POST /auth/change-password)
    - Current password validation + new password update
  - User preferences/settings storage
    - Notification preferences
    - Theme preferences
    - Language settings

  2. Data Export & Privacy

  - Export user data (GET /auth/export or /users/export)
    - Complete data dump (plants, photos, tracking entries)
    - GDPR compliance feature
  - Delete user account (DELETE /auth/account)
    - Complete account deletion with data cleanup
    - Currently placeholder in UserSettingsPage

  3. User Management (Admin)

  - Edit user functionality in AdminUsersPage
    - Update user roles, permissions, invite limits
    - The UI exists but edit modal is just a placeholder
  - Bulk user operations
    - Backend has bulk_user_action but frontend doesn't use it yet

  4. Notification System

  - Email notifications for plant care reminders
  - Push notifications (if PWA notifications are desired)
  - Notification preferences storage and handling
  - Weekly digest emails

  5. Waitlist Management

  - Homepage waitlist signup (POST /waitlist/join)
    - Currently just simulated in HomePage
    - Backend has /invites/waitlist but may need different structure
  - Admin waitlist management
    - View and manage waitlist entries
    - Convert waitlist to invites

  6. Advanced Plant Features

  - Plant care recommendations based on plant type
  - Plant identification features
  - Care history analytics
  - Plant health tracking trends

  7. Calendar Features

  - Calendar notifications/reminders
  - Recurring care schedules automation
  - Care predictions based on history

  8. File Management

  - Photo compression/optimization on upload
  - Photo metadata (EXIF data handling)
  - Bulk photo operations
  - Photo storage optimization

  9. Search & Filtering

  - Advanced plant search with filters
    - By care needs, plant type, health status
  - Global search across all user data
  - Search suggestions/autocomplete

  10. System Configuration

  - Email service configuration for notifications
  - File storage configuration (local vs cloud)
  - Backup and restore functionality
  - System maintenance tools

  11. Reporting & Analytics

  - User activity analytics (for admin)
  - System usage statistics
  - Plant care success metrics
  - Export system logs

  12. Authentication Enhancements

  - Password reset flow (forgot password)
  - Email verification for new accounts
  - Session management (view/revoke active sessions)
  - Two-factor authentication (optional security enhancement)

  Most Critical Missing Features:

  1. User profile updates (high priority - basic functionality)
  2. Password change (high priority - security)
  3. Data export (medium priority - privacy compliance)
  4. Waitlist signup (medium priority - marketing)
  5. User editing in admin panel (medium priority - admin functionality)
  6. Email notifications (low priority - enhancement)

  The admin pages are well-structured and most backend endpoints exist, but the user settings functionality is almost entirely placeholder code that needs real API
  implementation.
