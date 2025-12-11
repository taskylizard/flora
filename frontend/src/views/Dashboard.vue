<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import {
  AlertCircleIcon,
  CheckIcon,
  CopyIcon,
  Loader2Icon,
  PlusIcon,
  MoonIcon,
  SunIcon,
  ShieldAlertIcon,
  Trash2Icon,
  UploadCloudIcon,
  User2Icon,
} from 'lucide-vue-next'
import { format } from 'date-fns'
import { toast } from 'vue-sonner'

import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'
import { client } from '@/lib/client'
import type { components } from '@/lib/types'

type User = components['schemas']['AuthUser']
type Guild = components['schemas']['GuildResponse']
type Deployment = components['schemas']['DeploymentResponse']
type Token = components['schemas']['TokenResponse']

const loading = ref(true)
const me = ref<User | null>(null)
const guilds = ref<Guild[]>([])
const deployments = ref<Deployment[]>([])
const tokens = ref<Token[]>([])
const creatingToken = ref(false)
const newTokenLabel = ref('')
const isDark = ref(false)
const THEME_KEY = 'oakmoss_theme'

const guildMap = computed(() => {
  const map = new Map<string, Guild>()
  guilds.value.forEach((g) => map.set(g.id, g))
  return map
})

const goToLogin = () => {
  window.location.assign('/api/auth/login')
}

const loadAll = async () => {
  loading.value = true
  try {
    const [{ data: user, error: meErr, response: meRes }] = await Promise.all([
      client.GET('/auth/me' as any),
    ])
    if (meRes && meRes.status === 401) {
      goToLogin()
      return
    }
    if (meErr) throw meErr
    me.value = user?.user ?? null

    const [guildRes, deployRes, tokenRes] = await Promise.all([
      client.GET('/guilds/guilds' as any),
      client.GET('/deployments/deployments' as any),
      client.GET('/tokens/tokens' as any),
    ])

    if (guildRes.data) guilds.value = guildRes.data
    if (deployRes.data) deployments.value = deployRes.data
    if (tokenRes.data) tokens.value = tokenRes.data
  } catch (error) {
    console.error(error)
    toast.error('Unable to load dashboard', { description: 'Redirecting to login…' })
    goToLogin()
  } finally {
    loading.value = false
  }
}

const createToken = async () => {
  try {
    creatingToken.value = true
    const { data, error } = await client.POST('/tokens/tokens' as any, {
      body: { label: newTokenLabel.value || null },
    })
    if (error) throw error
    const token = data?.token
    toast.success('Token created', {
      description: token ?? 'Copy the token from the response.',
    })
    if (token) {
      await navigator.clipboard?.writeText(token)
    }
    await refreshTokens()
    newTokenLabel.value = ''
  } catch (err) {
    console.error(err)
    toast.error('Failed to create token')
  } finally {
    creatingToken.value = false
  }
}

const deleteToken = async (tokenId: string) => {
  try {
    const { error } = await client.DELETE('/tokens/tokens/{token_id}' as any, {
      params: { path: { token_id: tokenId } },
    })
    if (error) throw error
    toast.success('Token deleted')
    await refreshTokens()
  } catch (err) {
    console.error(err)
    toast.error('Failed to delete token')
  }
}

const refreshTokens = async () => {
  const res = await client.GET('/tokens/tokens' as any)
  if (res.data) tokens.value = res.data
}

const formatDate = (value?: string | null) => {
  if (!value) return '—'
  const d = new Date(value)
  if (Number.isNaN(d.getTime())) return '—'
  return format(d, 'PPP p')
}

const applyTheme = () => {
  document.documentElement.classList.toggle('dark', isDark.value)
}

onMounted(() => {
  const saved = localStorage.getItem(THEME_KEY)
  if (saved === 'dark' || saved === 'light') {
    isDark.value = saved === 'dark'
  } else {
    isDark.value = window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false
  }
  applyTheme()
})

watch(isDark, (val) => {
  localStorage.setItem(THEME_KEY, val ? 'dark' : 'light')
  applyTheme()
})

onMounted(loadAll)
</script>

<template>
  <div class="min-h-screen bg-background text-foreground">
    <header class="border-b bg-card/60 backdrop-blur supports-[backdrop-filter]:backdrop-blur">
      <div class="mx-auto flex max-w-6xl flex-col gap-3 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
        <div class="flex items-center gap-3">
          <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary text-primary-foreground">
            <UploadCloudIcon class="h-4 w-4" aria-hidden="true" />
          </div>
          <div>
            <p class="text-sm font-semibold leading-tight">Oakmoss</p>
            <p class="text-xs text-muted-foreground">Guild deployments & tokens</p>
          </div>
        </div>
        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:gap-4">
          <div v-if="me" class="flex items-center gap-2">
            <Avatar class="h-9 w-9">
              <AvatarImage :src="me.avatar ?? undefined" alt="User avatar" />
              <AvatarFallback>
                {{ me.username?.slice(0, 2)?.toUpperCase() ?? 'U' }}
              </AvatarFallback>
            </Avatar>
            <div class="leading-tight">
              <p class="text-sm font-medium truncate max-w-[12rem] sm:max-w-[16rem]">
                {{ me.global_name ?? me.username }}
              </p>
              <p class="text-xs text-muted-foreground truncate max-w-[12rem] sm:max-w-[16rem]">{{ me.id }}</p>
            </div>
          </div>
          <div class="flex items-center gap-2 sm:gap-3">
            <Separator orientation="vertical" class="hidden h-6 sm:block" />
            <Button
              variant="ghost"
              size="icon-sm"
              class="border border-border"
              :aria-pressed="isDark"
              aria-label="Toggle theme"
              @click="isDark = !isDark"
            >
              <SunIcon v-if="!isDark" class="h-4 w-4" />
              <MoonIcon v-else class="h-4 w-4" />
            </Button>
            <Button variant="outline" size="sm" class="w-full sm:w-auto" @click="goToLogin">
              Re-auth
            </Button>
          </div>
        </div>
      </div>
    </header>

    <main class="mx-auto flex max-w-6xl flex-col gap-6 px-4 py-6">
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2 text-lg">
            <User2Icon class="h-4 w-4 text-muted-foreground" />
            Account
          </CardTitle>
          <CardDescription>Your Discord profile and active session.</CardDescription>
        </CardHeader>
        <CardContent>
          <div v-if="loading" class="flex items-center gap-2 text-sm text-muted-foreground">
            <Loader2Icon class="h-4 w-4 animate-spin" />
            Loading profile…
          </div>
          <div v-else-if="me" class="flex flex-wrap items-center gap-4">
            <Avatar class="h-12 w-12">
              <AvatarImage :src="me.avatar ?? undefined" alt="User avatar" />
              <AvatarFallback>{{ me.username?.slice(0, 2)?.toUpperCase() ?? 'U' }}</AvatarFallback>
            </Avatar>
            <div class="space-y-1">
              <p class="text-base font-semibold">{{ me.global_name ?? me.username }}</p>
              <p class="text-sm text-muted-foreground">{{ me.username }}</p>
              <Badge variant="secondary" class="w-full max-w-xs truncate">ID: {{ me.id }}</Badge>
            </div>
          </div>
          <div v-else class="flex items-center gap-2 text-sm text-destructive">
            <AlertCircleIcon class="h-4 w-4" />
            Not authenticated. Redirecting…
          </div>
        </CardContent>
      </Card>

      <div class="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <ShieldAlertIcon class="h-4 w-4 text-muted-foreground" />
              User Tokens
            </CardTitle>
            <CardDescription>Manage API tokens for CLI or CI usage.</CardDescription>
          </CardHeader>
          <CardContent class="space-y-4">
            <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
              <Input
                v-model="newTokenLabel"
                placeholder="Label (optional)"
                class="w-full min-w-0 sm:w-56"
              />
              <Button size="sm" class="w-full sm:w-auto" :loading="creatingToken" @click="createToken">
                <PlusIcon class="h-4 w-4" />
                Create token
              </Button>
            </div>
            <div class="space-y-2">
              <div v-if="tokens.length === 0" class="text-sm text-muted-foreground">
                No tokens yet.
              </div>
              <div
                v-for="token in tokens"
                :key="token.token_id"
                class="flex flex-col gap-2 rounded-lg border px-3 py-2 text-sm sm:flex-row sm:items-center sm:justify-between"
              >
                <div class="space-y-0.5">
                  <p class="font-medium">{{ token.label ?? 'Untitled token' }}</p>
                  <p class="text-xs text-muted-foreground">
                    Created {{ formatDate(token.created_at) }} · Last used {{ formatDate(token.last_used_at) }}
                  </p>
                </div>
                <div class="flex items-center gap-1 sm:gap-2 self-start sm:self-auto">
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    class="text-muted-foreground"
                    @click="navigator.clipboard?.writeText(token.token_id)"
                    :aria-label="`Copy ${token.label ?? 'token'}`"
                  >
                    <CopyIcon class="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    class="text-destructive"
                    @click="deleteToken(token.token_id)"
                    aria-label="Delete token"
                  >
                    <Trash2Icon class="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <UploadCloudIcon class="h-4 w-4 text-muted-foreground" />
              Deployments
            </CardTitle>
            <CardDescription>Guild scripts currently stored.</CardDescription>
          </CardHeader>
          <CardContent class="space-y-3">
            <div v-if="loading" class="flex items-center gap-2 text-sm text-muted-foreground">
              <Loader2Icon class="h-4 w-4 animate-spin" />
              Loading deployments…
            </div>
            <div
              v-for="deployment in deployments"
              :key="deployment.guild_id"
              class="flex flex-col gap-2 rounded-lg border px-3 py-2 text-sm sm:flex-row sm:items-center sm:justify-between"
            >
              <div class="space-y-0.5">
                <p class="font-medium">
                  {{ guildMap.get(deployment.guild_id)?.name ?? 'Unknown guild' }}
                </p>
                <p class="text-xs text-muted-foreground">
                  {{ deployment.guild_id }} · {{ deployment.language }}
                </p>
              </div>
              <Badge variant="outline">
                Updated {{ formatDate(deployment.updated_at) }}
              </Badge>
            </div>
            <div v-if="!loading && deployments.length === 0" class="text-sm text-muted-foreground">
              No deployments found.
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <CheckIcon class="h-4 w-4 text-muted-foreground" />
            Manageable Servers
          </CardTitle>
          <CardDescription>Servers where you are admin and the bot is installed.</CardDescription>
        </CardHeader>
        <CardContent class="space-y-2">
          <div v-if="loading" class="flex items-center gap-2 text-sm text-muted-foreground">
            <Loader2Icon class="h-4 w-4 animate-spin" />
            Loading servers…
          </div>
          <div
            v-for="guild in guilds"
            :key="guild.id"
            class="flex flex-col gap-2 rounded-lg border px-3 py-2 sm:flex-row sm:items-center sm:justify-between"
          >
            <div class="flex items-center gap-3">
              <div
                class="flex h-9 w-9 items-center justify-center rounded-lg bg-muted text-sm font-semibold uppercase"
              >
                {{ guild.name.slice(0, 2) }}
              </div>
              <div>
                <p class="text-sm font-medium">{{ guild.name }}</p>
                <p class="text-xs text-muted-foreground">{{ guild.id }}</p>
              </div>
            </div>
            <Badge variant="secondary">Admin</Badge>
          </div>
          <div v-if="!loading && guilds.length === 0" class="text-sm text-muted-foreground">
            No manageable servers.
          </div>
        </CardContent>
      </Card>
    </main>
  </div>
</template>
