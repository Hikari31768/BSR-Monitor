/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'bs-dark': {
          900: '#0a0a0f',
          800: '#1a1a2e',
          700: '#16213e',
          600: '#0f3460',
        },
        'bs-accent': {
          primary: '#e94560',
          secondary: '#533483',
          neon: '#00ff88',
        },
      },
    },
  },
  plugins: [],
}