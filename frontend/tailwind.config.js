/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Bee theme colors
        'bee-yellow': '#FFC800',
        'bee-orange': '#FF8C00',
        'honey-gold': '#DAA520',
        'bee-dark': '#1E1E1E',
        'bee-darker': '#141414',
        'bee-light': '#2A2A2A',
        // UI colors
        'panel': 'rgba(30, 30, 30, 0.7)',
        'panel-border': 'rgba(255, 200, 0, 0.1)',
      },
      animation: {
        'buzz': 'buzz 0.3s ease-in-out',
        'pulse-bee': 'pulse-bee 2s ease-in-out infinite',
        'slide-in': 'slide-in 0.3s ease-out',
        'slide-up': 'slide-up 0.3s ease-out',
        'fade-in': 'fade-in 0.2s ease-in',
        'spin-slow': 'spin 2s linear infinite',
        'bounce-subtle': 'bounce-subtle 1s ease-in-out infinite',
        'glow': 'glow 2s ease-in-out infinite',
        'typing': 'typing 1s steps(3) infinite',
      },
      keyframes: {
        buzz: {
          '0%, 100%': { transform: 'translateX(0)' },
          '25%': { transform: 'translateX(-2px)' },
          '75%': { transform: 'translateX(2px)' },
        },
        'pulse-bee': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.6' },
        },
        'slide-in': {
          from: { transform: 'translateX(-100%)', opacity: '0' },
          to: { transform: 'translateX(0)', opacity: '1' },
        },
        'slide-up': {
          from: { transform: 'translateY(20px)', opacity: '0' },
          to: { transform: 'translateY(0)', opacity: '1' },
        },
        'fade-in': {
          from: { opacity: '0' },
          to: { opacity: '1' },
        },
        'bounce-subtle': {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-5px)' },
        },
        'glow': {
          '0%, 100%': { boxShadow: '0 0 20px rgba(255, 200, 0, 0.1)' },
          '50%': { boxShadow: '0 0 30px rgba(255, 200, 0, 0.3)' },
        },
        'typing': {
          '0%': { content: '.' },
          '33%': { content: '..' },
          '66%': { content: '...' },
        },
      },
      backdropBlur: {
        xs: '2px',
      },
      boxShadow: {
        'bee': '0 4px 20px rgba(255, 200, 0, 0.15)',
        'bee-lg': '0 8px 30px rgba(255, 200, 0, 0.2)',
        'inner-bee': 'inset 0 0 20px rgba(255, 200, 0, 0.05)',
      },
    },
  },
  plugins: [],
};
