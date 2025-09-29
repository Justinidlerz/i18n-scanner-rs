import { defineConfig } from 'vitest/config';

console.log('system info', process.platform, process.arch)

export default defineConfig({
    test: {
        pool: 'forks',
        poolOptions: {
            forks: {
                singleFork: true,
            },
        },
    },
});