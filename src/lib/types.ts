export type AudioSource = {
    path: string
    recursive: boolean
    active: boolean
}

export type Track = {
    title: string
    album: string | undefined
    artist: string | undefined
    duration: number | undefined
    path: string
    extension: string
}

// serde_json can't know what's inside of a JSON value
// so we send numbers as string to be deserialized into a
// HashMap<String, String>
export type GuildSlug = {
    id: number
    name: string
    voice_channels: VoiceChannelSlug[]
    offline: boolean
}

export type VoiceChannelSlug = {
    id: number
    name: string
    active: boolean
}
