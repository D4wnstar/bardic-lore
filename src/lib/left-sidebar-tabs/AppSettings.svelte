<script lang="ts">
    import { TrackSet } from '$lib/state/trackset.svelte'
    import {
        appTags,
        AUTOCONNECT_SETTING,
        AUTOHIDE_SIDEBARS_SETTING,
        DARK_MODE,
        HIDE_OST,
        settings,
        SETTINGS_FILENAME,
        SHOW_ALBUM_TAGS,
        SHOW_ARTIST_TAGS,
        SHOW_COVERS_SETTING,
        TAGS_FILENAME,
        TAGS_SETTING
    } from '$lib/stores.svelte'
    import type { CachedTrack } from '$lib/types'
    import ButtonSetting from '$lib/utils/settings/ButtonSetting.svelte'
    import SwitchSetting from '$lib/utils/settings/SwitchSetting.svelte'
    import type { ToastContext } from '@skeletonlabs/skeleton-svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { emit } from '@tauri-apps/api/event'
    import { open, save } from '@tauri-apps/plugin-dialog'
    import { load, Store } from '@tauri-apps/plugin-store'
    import { getContext, onMount } from 'svelte'

    interface Props {
        getCachedTracks: () => Promise<void>
        tracks: CachedTrack[]
    }

    let { getCachedTracks, tracks }: Props = $props()
    let settingsStore: Store
    let tagsStore: Store
    const toast: ToastContext = getContext('toast')

    async function darkMode(newState: boolean) {
        if (newState) {
            document.documentElement.classList.add('dark')
        } else {
            document.documentElement.classList.remove('dark')
        }

        settingsStore.set(DARK_MODE, newState)
    }

    async function showCovers(newState: boolean) {
        settingsStore.set(SHOW_COVERS_SETTING, newState)
        // Send an event to tell SongBoxes to reload their cover
        await emit('reload-cover')
    }

    async function connectOnLaunch(newState: boolean) {
        settingsStore.set(AUTOCONNECT_SETTING, newState)
    }

    async function autohideSidebars(newState: boolean) {
        settingsStore.set(AUTOHIDE_SIDEBARS_SETTING, newState)
    }

    async function hideOst(newState: boolean) {
        settingsStore.set(HIDE_OST, newState)
        await getCachedTracks()
    }

    async function showAlbumTags(newState: boolean) {
        settingsStore.set(SHOW_ALBUM_TAGS, newState)
        await getCachedTracks()
    }

    async function showArtistTags(newState: boolean) {
        settingsStore.set(SHOW_ARTIST_TAGS, newState)
        await getCachedTracks()
    }

    async function importTags() {
        const path = await open({
            title: 'Import tags',
            directory: false,
            multiple: false,
            filters: [
                {
                    name: 'JSON',
                    extensions: ['json']
                }
            ]
        })

        if (!path) return

        const tagsJson = await invoke<string>('load_tags', { path }).catch(
            (e) => {
                toast.create({
                    title: 'Error',
                    description: e,
                    type: 'error'
                })
                console.error(e)
            }
        )

        if (!tagsJson) {
            const e = `Could not read tags from ${path}`
            toast.create({
                title: 'Error',
                description: e,
                type: 'error'
            })
            console.error(e)
            return
        }

        try {
            appTags.import(tagsJson, new TrackSet(tracks))
            await getCachedTracks()
            await tagsStore.set(TAGS_SETTING, appTags)
        } catch (e) {
            toast.create({
                title: 'Error',
                //@ts-ignore
                description: e,
                type: 'error'
            })
            console.error(e)
        }
    }

    async function exportTags() {
        let path = await save({
            title: 'Export tags',
            filters: [
                {
                    name: 'JSON',
                    extensions: ['json']
                }
            ]
        })

        if (path === null) return

        // Make sure we have a JSON file
        if (!path.toLowerCase().endsWith('.json')) {
            path += '.json'
        }

        await invoke('save_tags', { tags: appTags.export(), path })
            .catch((e) => {
                toast.create({
                    title: 'Error',
                    description: e,
                    type: 'error'
                })
                console.error(e)
            })
            .then(() => {
                toast.create({
                    title: 'Exported tags',
                    description: 'Successfully exported tags.',
                    type: 'info'
                })
                console.log(`Tags exported to ${path}`)
            })

        // The fs plugin does not seem to work (all commands infinitely await
        // and never complete) so until the cause is found, we save from the
        // backend. (It is not a permission problem)
        // await writeTextFile('tags.txt', 'hello world', {
        //     baseDir: BaseDirectory.Document
        // })
    }

    onMount(async () => {
        settingsStore = await load(SETTINGS_FILENAME)
        tagsStore = await load(TAGS_FILENAME)
    })
</script>

<div id="settings-sidebar" class="flex flex-col h-full min-h-0 px-2">
    <h3 class="type-scale-7 heading-font-weight text-primary-900-100 pb-2">
        Settings
    </h3>
    <div class="space-y-6 pb-5">
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
                description="If on, embedded covers will be shown behind the tracks. May reduce performance."
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
                description="If on, albums are shown as selectable tags."
                switchName="album-tags"
                bind:checked={settings.showAlbumTags}
                onCheckedChange={showAlbumTags}
            />
            <SwitchSetting
                name="Show artist tags"
                description="If on, artists are shown as selectable tags."
                switchName="artist-tags"
                bind:checked={settings.showArtistTags}
                onCheckedChange={showArtistTags}
            />
            <ButtonSetting
                name="Import tags"
                description="Import tags from a JSON file exported from Bardic Lore. This process is irreversible! It is recommended you export your tags first as a precaution."
                buttonName="Import"
                onClick={importTags}
            />
            <ButtonSetting
                name="Export tags"
                description="Export tags to a file. This will export all your tags, tag groups and information about the tracks that have them. This includes the tracks' filenames."
                buttonName="Export"
                onClick={exportTags}
            />
        </section>
    </div>
</div>
