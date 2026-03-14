import { defineConfig } from 'vite';
import { VitePWA } from 'vite-plugin-pwa';

export default defineConfig({
  plugins: [
    VitePWA({
      registerType: 'autoUpdate',
      devOptions: {
        enabled: true,
      },
      manifest: {
        short_name: 'Skull',
        name: 'Skull',
        start_url: '.',
        display: 'standalone',
        theme_color: "#282c34",
        background_color: "#282c34",
        icons: [
          {
            src: 'favicon.png',
            sizes: '64x64 32x32 24x24 16x16',
            type: 'image/x-icon'
          },
        ],
      },
    }),
  ],
});
