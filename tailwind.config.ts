import forms from '@tailwindcss/forms'
import typography from '@tailwindcss/typography'
import { skeleton, contentPath } from '@skeletonlabs/skeleton/plugin'
import * as themes from '@skeletonlabs/skeleton/themes'
import type { Config } from 'tailwindcss'
import Cosmos from './static/cosmos'

export default {
    content: [
        './index.html',
        './src/**/*.{js,ts,svelte}',
        contentPath(import.meta.url, 'svelte')
    ],

    darkMode: 'selector',

    theme: {
        extend: {}
    },

    plugins: [
        typography,
        forms,
        skeleton({
            // NOTE: each theme included will increase the size of your CSS bundle
            themes: [themes.wintry, Cosmos]
        })
    ]
} satisfies Config
