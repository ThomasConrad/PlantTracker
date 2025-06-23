import { Component, createSignal } from 'solid-js';
import { A } from '@solidjs/router';

export const ContactPage: Component = () => {
  const [formSubmitted, setFormSubmitted] = createSignal(false);

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    // In a real app, you'd send this to a backend service
    setFormSubmitted(true);
    setTimeout(() => setFormSubmitted(false), 5000);
  };

  return (
    <div class="min-h-screen bg-gray-50 dark:bg-gray-900 py-8">
      <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="bg-white dark:bg-gray-800 shadow rounded-lg p-8">
          <div class="mb-8">
            <A href="/" class="text-green-600 hover:text-green-500 text-sm font-medium">
              ← Back to Home
            </A>
          </div>
          
          <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-8">Contact Us</h1>
          
          <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {/* Contact Information */}
            <div>
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Get in Touch</h2>
              
              <div class="space-y-6">
                <div>
                  <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">General Support</h3>
                  <p class="text-gray-600 dark:text-gray-400 mb-2">
                    For questions about using Planty, plant care tracking, or general support.
                  </p>
                  <a 
                    href="mailto:support@planty.app" 
                    class="text-green-600 hover:text-green-500 font-medium"
                  >
                    support@planty.app
                  </a>
                </div>

                <div>
                  <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">Technical Issues</h3>
                  <p class="text-gray-600 dark:text-gray-400 mb-2">
                    Report bugs, technical problems, or access issues.
                  </p>
                  <a 
                    href="mailto:tech@planty.app" 
                    class="text-green-600 hover:text-green-500 font-medium"
                  >
                    tech@planty.app
                  </a>
                </div>

                <div>
                  <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">Privacy & Data</h3>
                  <p class="text-gray-600 dark:text-gray-400 mb-2">
                    Questions about your data, privacy, or account deletion requests.
                  </p>
                  <a 
                    href="mailto:privacy@planty.app" 
                    class="text-green-600 hover:text-green-500 font-medium"
                  >
                    privacy@planty.app
                  </a>
                </div>

                <div>
                  <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">Business Inquiries</h3>
                  <p class="text-gray-600 dark:text-gray-400 mb-2">
                    Partnership opportunities, press inquiries, or business matters.
                  </p>
                  <a 
                    href="mailto:business@planty.app" 
                    class="text-green-600 hover:text-green-500 font-medium"
                  >
                    business@planty.app
                  </a>
                </div>
              </div>

              <div class="mt-8 p-4 bg-green-50 dark:bg-green-900/20 rounded-lg">
                <h3 class="text-lg font-medium text-green-900 dark:text-green-100 mb-2">Response Time</h3>
                <p class="text-green-800 dark:text-green-200 text-sm">
                  We aim to respond to all inquiries within 48 hours during business days. 
                  For urgent technical issues, please include "URGENT" in your subject line.
                </p>
              </div>
            </div>

            {/* Contact Form */}
            <div>
              <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Send us a Message</h2>
              
              {formSubmitted() ? (
                <div class="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                  <div class="flex">
                    <div class="flex-shrink-0">
                      <svg class="h-5 w-5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                      </svg>
                    </div>
                    <div class="ml-3">
                      <h3 class="text-sm font-medium text-green-800 dark:text-green-200">
                        Message sent successfully!
                      </h3>
                      <p class="mt-1 text-sm text-green-700 dark:text-green-300">
                        We'll get back to you as soon as possible.
                      </p>
                    </div>
                  </div>
                </div>
              ) : (
                <form onSubmit={handleSubmit} class="space-y-6">
                  <div>
                    <label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Name
                    </label>
                    <input
                      type="text"
                      id="name"
                      name="name"
                      required
                      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500 dark:bg-gray-700 dark:text-white"
                    />
                  </div>

                  <div>
                    <label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Email
                    </label>
                    <input
                      type="email"
                      id="email"
                      name="email"
                      required
                      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500 dark:bg-gray-700 dark:text-white"
                    />
                  </div>

                  <div>
                    <label for="subject" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Subject
                    </label>
                    <input
                      type="text"
                      id="subject"
                      name="subject"
                      required
                      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500 dark:bg-gray-700 dark:text-white"
                    />
                  </div>

                  <div>
                    <label for="category" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Category
                    </label>
                    <select
                      id="category"
                      name="category"
                      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500 dark:bg-gray-700 dark:text-white"
                    >
                      <option value="general">General Support</option>
                      <option value="technical">Technical Issue</option>
                      <option value="privacy">Privacy & Data</option>
                      <option value="business">Business Inquiry</option>
                      <option value="other">Other</option>
                    </select>
                  </div>

                  <div>
                    <label for="message" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Message
                    </label>
                    <textarea
                      id="message"
                      name="message"
                      rows={5}
                      required
                      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500 dark:bg-gray-700 dark:text-white"
                      placeholder="Please describe your question or issue in detail..."
                    ></textarea>
                  </div>

                  <button
                    type="submit"
                    class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                  >
                    Send Message
                  </button>
                </form>
              )}
            </div>
          </div>

          {/* Additional Information */}
          <div class="mt-12 pt-8 border-t border-gray-200 dark:border-gray-700">
            <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Before You Contact Us</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">Common Questions</h3>
                <ul class="text-sm text-gray-600 dark:text-gray-400 space-y-1">
                  <li>• How do I export my plant data?</li>
                  <li>• Can I sync data across multiple devices?</li>
                  <li>• How do I delete my account?</li>
                  <li>• What plant care information does Planty track?</li>
                </ul>
              </div>
              <div>
                <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">Quick Help</h3>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">
                  Many questions can be answered by:
                </p>
                <ul class="text-sm text-gray-600 dark:text-gray-400 space-y-1">
                  <li>• Checking your account settings</li>
                  <li>• Reviewing the app's help sections</li>
                  <li>• Trying to logout and login again</li>
                  <li>• Refreshing the page</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};