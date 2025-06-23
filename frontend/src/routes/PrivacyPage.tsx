import { Component } from 'solid-js';
import { A } from '@solidjs/router';

export const PrivacyPage: Component = () => {
  return (
    <div class="min-h-screen bg-gray-50 dark:bg-gray-900 py-8">
      <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="bg-white dark:bg-gray-800 shadow rounded-lg p-8">
          <div class="mb-8">
            <A href="/" class="text-green-600 hover:text-green-500 text-sm font-medium">
              ← Back to Home
            </A>
          </div>
          
          <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-8">Privacy Policy</h1>
          
          <div class="prose prose-gray dark:prose-invert max-w-none">
            <p class="text-sm text-gray-600 dark:text-gray-400 mb-6">
              Last updated: {new Date().toLocaleDateString()}
            </p>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Information We Collect</h2>
              <p class="mb-4">
                Planty collects minimal information necessary to provide our plant tracking service:
              </p>
              <ul class="list-disc pl-6 mb-4">
                <li>Account information: Email address, name, and password</li>
                <li>Plant data: Plant names, care schedules, photos, and tracking entries you create</li>
                <li>Usage data: Basic analytics about how you use the application</li>
              </ul>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">How We Use Your Information</h2>
              <p class="mb-4">We use your information to:</p>
              <ul class="list-disc pl-6 mb-4">
                <li>Provide and maintain the Planty service</li>
                <li>Store and sync your plant data across devices</li>
                <li>Send you care reminders and notifications</li>
                <li>Improve our service and user experience</li>
                <li>Communicate with you about your account</li>
              </ul>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Data Storage and Security</h2>
              <p class="mb-4">
                Your data is stored securely using industry-standard encryption. We implement appropriate 
                technical and organizational measures to protect your personal information against 
                unauthorized access, alteration, disclosure, or destruction.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Data Sharing</h2>
              <p class="mb-4">
                We do not sell, trade, or otherwise transfer your personal information to third parties. 
                Your plant data remains private and is only accessible to you through your account.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Data Retention</h2>
              <p class="mb-4">
                We retain your personal information for as long as your account is active or as needed 
                to provide you services. You may delete your account and all associated data at any time.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Your Rights</h2>
              <p class="mb-4">You have the right to:</p>
              <ul class="list-disc pl-6 mb-4">
                <li>Access your personal data</li>
                <li>Correct inaccurate data</li>
                <li>Delete your account and data</li>
                <li>Export your data</li>
                <li>Opt out of communications</li>
              </ul>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Cookies</h2>
              <p class="mb-4">
                We use essential cookies to maintain your login session and provide basic functionality. 
                No tracking or advertising cookies are used.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Children's Privacy</h2>
              <p class="mb-4">
                Our service is not intended for children under 13. We do not knowingly collect 
                personal information from children under 13.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Changes to This Policy</h2>
              <p class="mb-4">
                We may update this privacy policy from time to time. We will notify you of any 
                significant changes by posting the new policy on this page.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Contact Us</h2>
              <p>
                If you have any questions about this privacy policy, please contact us through 
                our <A href="/contact" class="text-green-600 hover:text-green-500">contact page</A>.
              </p>
            </section>
          </div>
        </div>
      </div>
    </div>
  );
};