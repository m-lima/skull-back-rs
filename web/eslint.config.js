import javascript from '@eslint/js';
import prettier from 'eslint-config-prettier';
import typescript from 'typescript-eslint';

export default typescript.config(
  {
    ignores: ['**/dist/', '**/dev-dist/', 'vite.config.ts'],
  },
  javascript.configs.recommended,
  {
    files: ['**/*.{ts,tsx}'],

    extends: typescript.configs.strictTypeChecked,
    rules: {
      '@typescript-eslint/restrict-template-expressions': [
        'error',
        {
          allowNumber: true,
        },
      ],
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          args: 'all',
          argsIgnorePattern: '^_',
          caughtErrors: 'all',
          caughtErrorsIgnorePattern: '^_',
          destructuredArrayIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          ignoreRestSiblings: true,
        },
      ],
    },

    languageOptions: {
      parserOptions: {
        tsconfigRootDir: import.meta.dirname,
        project: true,
      },
    },
  },
  prettier,
);
