import { Component } from 'solid-js';
import { A } from '@solidjs/router';

export const TermsPage: Component = () => {
  return (
    <div class="min-h-screen bg-gray-50 dark:bg-gray-900 py-8">
      <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="bg-white dark:bg-gray-800 shadow rounded-lg p-8">
          <div class="mb-8">
            <A href="/" class="text-green-600 hover:text-green-500 text-sm font-medium">
              ← Back to Home
            </A>
          </div>
          
          <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-8">Terms of Service</h1>
          
          <div class="prose prose-gray dark:prose-invert max-w-none">
            <p class="text-sm text-gray-600 dark:text-gray-400 mb-6">
              Last updated: {new Date().toLocaleDateString()}
            </p>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Acceptance of Terms</h2>
              <p class="mb-4">
                By accessing and using Planty, you accept and agree to be bound by the terms and 
                provision of this agreement. If you do not agree to abide by the above, please 
                do not use this service.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Service Description</h2>
              <p class="mb-4">
                Planty is a plant care tracking application that allows users to monitor their plants, 
                set care schedules, track activities, and manage plant-related data. The service is 
                provided "as is" without warranties of any kind.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">User Accounts</h2>
              <p class="mb-4">
                To use Planty, you must create an account by providing accurate and complete information. 
                You are responsible for:
              </p>
              <ul class="list-disc pl-6 mb-4">
                <li>Maintaining the security of your account and password</li>
                <li>All activities that occur under your account</li>
                <li>Immediately notifying us of any unauthorized use</li>
                <li>Ensuring your account information remains accurate and up-to-date</li>
              </ul>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Acceptable Use</h2>
              <p class="mb-4">You agree not to:</p>
              <ul class="list-disc pl-6 mb-4">
                <li>Use the service for any unlawful purpose or activity</li>
                <li>Upload malicious code or attempt to compromise system security</li>
                <li>Interfere with or disrupt the service or servers</li>
                <li>Attempt to gain unauthorized access to other user accounts</li>
                <li>Use the service to spam or send unsolicited communications</li>
                <li>Upload content that violates intellectual property rights</li>
              </ul>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Content and Data</h2>
              <p class="mb-4">
                You retain ownership of all content and data you upload to Planty. By using our service, 
                you grant us a limited license to store, process, and display your content solely for 
                the purpose of providing the service to you.
              </p>
              <p class="mb-4">
                You are responsible for backing up your data. While we strive to maintain data integrity, 
                we recommend regular exports of your plant data.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Plant Care Disclaimer</h2>
              <p class="mb-4">
                Planty provides tools for tracking plant care but does not provide professional 
                horticultural advice. The application's suggestions and reminders are for informational 
                purposes only. We are not responsible for plant health outcomes or any damage resulting 
                from following the application's suggestions.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Service Availability</h2>
              <p class="mb-4">
                We strive to maintain high service availability but cannot guarantee uninterrupted access. 
                The service may be temporarily unavailable due to maintenance, updates, or unforeseen issues.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Limitation of Liability</h2>
              <p class="mb-4">
                To the maximum extent permitted by law, Planty and its operators shall not be liable for 
                any indirect, incidental, special, consequential, or punitive damages, including but not 
                limited to loss of data, loss of revenue, or loss of plants.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Account Termination</h2>
              <p class="mb-4">
                You may terminate your account at any time. We may suspend or terminate accounts that 
                violate these terms. Upon termination, your access to the service will cease, and your 
                data may be deleted after a reasonable period.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Privacy Policy</h2>
              <p class="mb-4">
                Your privacy is important to us. Please review our{' '}
                <A href="/privacy" class="text-green-600 hover:text-green-500">Privacy Policy</A> to 
                understand how we collect, use, and protect your information.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Changes to Terms</h2>
              <p class="mb-4">
                We reserve the right to modify these terms at any time. Material changes will be 
                communicated to users. Continued use of the service after changes constitutes 
                acceptance of the new terms.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Governing Law</h2>
              <p class="mb-4">
                These terms shall be governed by and construed in accordance with applicable local laws. 
                Any disputes shall be resolved through appropriate legal channels.
              </p>
            </section>

            <section class="mb-8">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Contact Information</h2>
              <p>
                If you have any questions about these terms, please contact us through our{' '}
                <A href="/contact" class="text-green-600 hover:text-green-500">contact page</A>.
              </p>
            </section>
          </div>
        </div>
      </div>
    </div>
  );
};