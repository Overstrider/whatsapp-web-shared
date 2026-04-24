import type { Config } from 'tailwindcss';

const config: Config = {
  content: ['./app/**/*.{ts,tsx}', './components/**/*.{ts,tsx}', './lib/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        wa: {
          primary: '#00a884',
          primaryDark: '#008069',
          bg: '#efeae2',
          panel: '#ffffff',
          panelAlt: '#f0f2f5',
          ink: '#111b21',
          muted: '#667781',
          bubbleOut: '#d9fdd3',
          bubbleIn: '#ffffff',
          active: '#f0f2f5',
          border: '#e9edef',
        },
      },
      fontFamily: {
        sans: ['"Segoe UI"', 'Roboto', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [],
};

export default config;
