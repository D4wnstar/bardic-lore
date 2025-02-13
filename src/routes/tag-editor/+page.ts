import { appTracks } from '$lib/stores.svelte'
import { error } from '@sveltejs/kit'
import type { PageLoad } from './$types'

export const load = (async ({ url }) => {
    const trackFilename = url.searchParams.get('track')
    if (!trackFilename) {
        error(400, { message: 'Missing or malfored `track` URL parameter.' })
    }
    let track = appTracks.getByFilename(trackFilename)
    if (!track) {
        error(404, {
            message: `No track of filename ${trackFilename} found. Try going back and opening the tag editor again.`
        })
    }
    return { track }
}) satisfies PageLoad
