import {
    SETTINGS_FILENAME,
    appState,
    LOOP_SETTING,
    SHUFFLE_SETTING,
    MUTE_SETTING,
    settings,
    SHOW_COVERS_SETTING,
    AUTOCONNECT_SETTING
} from '$lib/stores.svelte'
import { load as tauriLoad } from '@tauri-apps/plugin-store'
import type { LayoutLoad } from './$types'

export const load = (async () => {
    // Load defaults and/or persisted states before the page loads
    const settingsStore = await tauriLoad(SETTINGS_FILENAME)

    // Player state
    appState.player.loopState =
        (await settingsStore.get(LOOP_SETTING)) ?? appState.player.loopState
    appState.player.shuffle =
        (await settingsStore.get(SHUFFLE_SETTING)) ?? appState.player.shuffle
    appState.player.mute =
        (await settingsStore.get(MUTE_SETTING)) ?? appState.player.mute

    // Settings
    settings.showCovers =
        (await settingsStore.get(SHOW_COVERS_SETTING)) ?? settings.showCovers
    settings.autoconnect =
        (await settingsStore.get(AUTOCONNECT_SETTING)) ?? settings.autoconnect

    return {}
}) satisfies LayoutLoad

// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true
export const ssr = false
