import { Component } from 'solid-js';
import { A } from '@solidjs/router';

export const NotFoundPage: Component = () => {
  return (
    <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
      <div class="max-w-md w-full space-y-8 text-center">
        <div>
          <div class="mx-auto h-24 w-24 rounded-full bg-green-100 dark:bg-green-900/40 flex items-center justify-center">
            <svg class="h-16 w-16" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M32 30V20" stroke="#2F855A" stroke-width="3" stroke-linecap="round" />
              <path d="M32 20C32 14 38 11 43 12C43 18 38 23 32 23" fill="#68D391" />
              <path d="M32 20C32 14 26 11 21 12C21 18 26 23 32 23" fill="#48BB78" />
              <path d="M18 34H46L43 50H21L18 34Z" fill="#B7791F" />
              <path d="M18 34H46" stroke="#975A16" stroke-width="2" />
              <circle cx="28" cy="40" r="1.8" fill="#2D3748" />
              <circle cx="36" cy="40" r="1.8" fill="#2D3748" />
              <path d="M28 45C29.2 46.6 30.8 47.4 32 47.4C33.2 47.4 34.8 46.6 36 45" stroke="#2D3748" stroke-width="1.8" stroke-linecap="round" />
            </svg>
          </div>
          <h2 class="mt-6 text-3xl font-extrabold text-gray-900 dark:text-white">
            404 - Page Not Found
          </h2>
          <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
            Sorry, we couldn't find the page you're looking for.
          </p>
        </div>
        
        <div class="mt-8 space-y-4">
          <A
            href="/"
            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
          >
            Go Home
          </A>
          
          <button
            onClick={() => window.history.back()}
            class="group relative w-full flex justify-center py-2 px-4 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 dark:border-gray-600 dark:text-gray-300 dark:bg-gray-800 dark:hover:bg-gray-700"
          >
            Go Back
          </button>
        </div>
      </div>
    </div>
  );
};
