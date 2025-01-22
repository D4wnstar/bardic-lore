<script lang="ts">
    import {
        AUTOCONNECT_SETTING,
        AUTOHIDE_SIDEBARS_SETTING,
        DARK_MODE,
        HIDE_OST,
        settings,
        SETTINGS_FILENAME,
        SHOW_ALBUM_TAGS,
        SHOW_ARTIST_TAGS,
        SHOW_COVERS_SETTING
    } from '$lib/stores.svelte'
    import SwitchSetting from '$lib/utils/settings/SwitchSetting.svelte'
    import { emit } from '@tauri-apps/api/event'
    import { load, Store } from '@tauri-apps/plugin-store'
    import { onMount } from 'svelte'

    interface Props {
        getCachedTracks: () => Promise<void>
    }

    let { getCachedTracks }: Props = $props()
    let store: Store

    async function darkMode(newState: boolean) {
        if (newState) {
            document.documentElement.classList.add('dark')
        } else {
            document.documentElement.classList.remove('dark')
        }

        store.set(DARK_MODE, newState)
    }

    async function showCovers(newState: boolean) {
        store.set(SHOW_COVERS_SETTING, newState)
        // Send an event to tell SongBoxes to reload their cover
        await emit('reload-cover')
    }

    async function connectOnLaunch(newState: boolean) {
        store.set(AUTOCONNECT_SETTING, newState)
    }

    async function autohideSidebars(newState: boolean) {
        store.set(AUTOHIDE_SIDEBARS_SETTING, newState)
    }

    async function hideOst(newState: boolean) {
        store.set(HIDE_OST, newState)
        await getCachedTracks()
    }

    async function showAlbumTags(newState: boolean) {
        store.set(SHOW_ALBUM_TAGS, newState)
        await getCachedTracks()
    }

    async function showArtistTags(newState: boolean) {
        store.set(SHOW_ARTIST_TAGS, newState)
        await getCachedTracks()
    }

    onMount(async () => {
        store = await load(SETTINGS_FILENAME)
    })
</script>

<div id="settings-sidebar" class="flex flex-col h-full min-h-0 pb-5 px-2">
    <h3 class="type-scale-7 heading-font-weight text-primary-900-100 pb-2">
        Settings
    </h3>
    <div class="space-y-6">
        <section class="space-y-4">
            <header class="type-scale-4">Appearance</header>
            <hr class="hr" />
            <SwitchSetting
                name="Light/dark mode"
                description="Switch between light and dark modes."
                switchName="dark-mode"
                bind:checked={settings.darkMode}
                onCheckedChange={darkMode}
            />
            <SwitchSetting
                name="Show cover images"
                description="If on, embedded covers will be shown behind the tracks. May reduce peformance."
                switchName="show-cover-images"
                bind:checked={settings.showCovers}
                onCheckedChange={showCovers}
            />
            <SwitchSetting
                name="Auto-hide sidebars"
                description="If on, the sidebars will automatically close and open when the window is resized."
                switchName="autohide-sidebars"
                bind:checked={settings.autohideSidebars}
                onCheckedChange={autohideSidebars}
            />
            <SwitchSetting
                name="Hide 'Original Soundtrack'"
                description="If on, 'Original Soundtrack' and similar expressions in albums will be hidden."
                switchName="hide-ost"
                bind:checked={settings.hideOst}
                onCheckedChange={hideOst}
            />
        </section>
        <section class="space-y-4">
            <header class="type-scale-4">Bot</header>
            <hr class="hr" />
            <SwitchSetting
                name="Connect on launch"
                description="If on, the bot will try to connect when the app is started."
                switchName="autoconnect"
                bind:checked={settings.autoconnect}
                onCheckedChange={connectOnLaunch}
            />
        </section>
        <section class="space-y-4">
            <header class="type-scale-4">Tags</header>
            <hr class="hr" />
            <SwitchSetting
                name="Show album tags"
                description="If on, albums become selectable tags."
                switchName="album-tags"
                bind:checked={settings.showAlbumTags}
                onCheckedChange={showAlbumTags}
            />
            <SwitchSetting
                name="Show artist tags"
                description="If on, shows artists become selectable tags."
                switchName="artist-tags"
                bind:checked={settings.showArtistTags}
                onCheckedChange={showArtistTags}
            />
        </section>
    </div>
</div>
