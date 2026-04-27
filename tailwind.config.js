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
          // 背景色阶 - BeatSaver 风格深灰系
          bg: {
            primary: '#1a1a1a',      // 主背景
            secondary: '#232323',    // 卡片/侧边栏背景
            tertiary: '#2d2d2d',     // 输入框/按钮背景
            hover: '#3a3a3a',        // 悬停状态
          },
          // 文字色阶
          text: {
            primary: '#ffffff',
            secondary: '#b0b0b0',
            muted: '#808080',
          },
          // 强调色 - BeatSaver 绿色系
          accent: {
            primary: '#3ecf8e',      // 主强调色（绿色）
            secondary: '#2eb37a',    // 深绿色（悬停）
            purple: '#9b59b6',
            blue: '#3498db',
            green: '#3ecf8e',
            yellow: '#f1c40f',
          },
          // 边框
          border: {
            DEFAULT: '#3a3a3a',
            hover: '#4a4a4a',
          },
          // 状态色
          status: {
            success: '#3ecf8e',
            warning: '#f59e0b',
            error: '#e74c3c',
            info: '#3498db',
          },
        },
        // 难度颜色（BeatSaver 风格）
        difficulty: {
          easy: { bg: 'rgba(62, 207, 142, 0.2)', text: '#3ecf8e', border: 'rgba(62, 207, 142, 0.4)' },
          normal: { bg: 'rgba(52, 152, 219, 0.2)', text: '#3498db', border: 'rgba(52, 152, 219, 0.4)' },
          hard: { bg: 'rgba(230, 126, 34, 0.2)', text: '#e67e22', border: 'rgba(230, 126, 34, 0.4)' },
          expert: { bg: 'rgba(231, 76, 60, 0.2)', text: '#e74c3c', border: 'rgba(231, 76, 60, 0.4)' },
          expertplus: { bg: 'rgba(155, 89, 182, 0.2)', text: '#9b59b6', border: 'rgba(155, 89, 182, 0.4)' },
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
      },
      boxShadow: {
        'card': '0 2px 8px rgba(0, 0, 0, 0.4)',
        'card-hover': '0 4px 12px rgba(0, 0, 0, 0.5)',
        'glow': '0 0 20px rgba(62, 207, 142, 0.3)',
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