/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'bs': {
          // 背景色阶
          bg: {
            primary: '#0d0d12',
            secondary: '#15151c',
            tertiary: '#1e1e28',
            hover: '#252533',
          },
          // 文字色阶
          text: {
            primary: '#f0f0f5',
            secondary: '#a0a0b0',
            muted: '#6b7280',
          },
          // 强调色
          accent: {
            primary: '#ff6b6b',
            secondary: '#4ecdc4',
            purple: '#9b59b6',
            blue: '#3498db',
            green: '#2ecc71',
            yellow: '#f1c40f',
          },
          // 边框
          border: {
            DEFAULT: '#2a2a3a',
            hover: '#3a3a4f',
          },
          // 状态色
          status: {
            success: '#22c55e',
            warning: '#f59e0b',
            error: '#ef4444',
            info: '#3b82f6',
          },
        },
        // 难度颜色（用于 badge）
        difficulty: {
          easy: { bg: 'rgba(34, 197, 94, 0.15)', text: '#4ade80', border: 'rgba(34, 197, 94, 0.3)' },
          normal: { bg: 'rgba(59, 130, 246, 0.15)', text: '#60a5fa', border: 'rgba(59, 130, 246, 0.3)' },
          hard: { bg: 'rgba(245, 158, 11, 0.15)', text: '#fbbf24', border: 'rgba(245, 158, 11, 0.3)' },
          expert: { bg: 'rgba(239, 68, 68, 0.15)', text: '#f87171', border: 'rgba(239, 68, 68, 0.3)' },
          expertplus: { bg: 'rgba(155, 89, 182, 0.15)', text: '#c084fc', border: 'rgba(155, 89, 182, 0.3)' },
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
      },
      boxShadow: {
        'card': '0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -1px rgba(0, 0, 0, 0.2)',
        'card-hover': '0 10px 15px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -2px rgba(0, 0, 0, 0.3)',
        'glow': '0 0 20px rgba(255, 107, 107, 0.3)',
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'fade-in': 'fadeIn 0.3s ease-out',
        'slide-up': 'slideUp 0.3s ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
      },
    },
  },
  plugins: [],
}