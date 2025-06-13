/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#f0fdf4',
          100: '#dcfce7',
          200: '#bbf7d0',
          300: '#86efac',
          400: '#4ade80',
          500: '#22c55e',
          600: '#16a34a',
          700: '#15803d',
          800: '#166534',
          900: '#14532d',
        }
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [
    function({ addComponents }) {
      addComponents({
        '.calendar-cell': {
          '@apply h-24 bg-white p-2 cursor-pointer hover:bg-gray-50 border-b border-r border-gray-100 overflow-hidden': {},
        },
        '.calendar-cell-disabled': {
          '@apply bg-gray-50 text-gray-400': {},
        },
        '.calendar-date': {
          '@apply text-sm font-medium mb-1': {},
        },
        '.calendar-date-today': {
          '@apply bg-blue-600 text-white rounded-full w-6 h-6 flex items-center justify-center': {},
        },
        '.calendar-event': {
          '@apply text-xs p-1 rounded border truncate cursor-pointer hover:opacity-80 transition-opacity': {},
        },
        '.activity-dot': {
          '@apply w-1.5 h-1.5 rounded-full': {},
        },
        '.nav-button': {
          '@apply p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-full transition-colors': {},
        },
        '.nav-button-compact': {
          '@apply p-1.5 text-gray-400 hover:text-gray-600 rounded-full transition-colors': {},
        },
        '.modal-overlay': {
          '@apply fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50': {},
        },
        '.modal-content': {
          '@apply bg-white rounded-lg shadow-xl max-w-xs w-full transform transition-all duration-200 ease-out': {},
        },
      })
    }
  ],
};