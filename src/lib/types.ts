export type AudioSource = {
    path: string
    recursive: boolean
    active: boolean
}

export type Track = {
    title: string
    album: string
    artist: string
    path: string
    extension: string
}

export type GuildSlug = {
    id: number
    name: string
    voice_channels: VoiceChannelSlug[]
}

export type VoiceChannelSlug = {
    id: number
    name: string
    active: boolean
}
