<script setup lang="ts">
import { ref, watchEffect } from 'vue'
import { toast } from 'vue-sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { Toaster } from '@/components/ui/sonner'

const botName = ref('Oakmoss bot')
const isDark = ref(false)

watchEffect(() => {
  document.documentElement.classList.toggle('dark', isDark.value)
})

const handleSave = () => {
  toast.success('Settings saved', {
    description: `Saved preferences for ${botName.value}`,
  })
}
</script>

<template>
  <div :class="[{ dark: isDark }, 'min-h-screen bg-background text-foreground transition-colors']">
    <main class="mx-auto flex max-w-4xl flex-col gap-8 px-4 py-10">
      <section class="flex flex-wrap items-center justify-between gap-4">
        <div class="space-y-2">
          <p class="text-sm font-medium text-muted-foreground">shadcn-vue + Tailwind v4</p>
          <h1 class="text-3xl font-semibold tracking-tight">
            UI toolkit is wired up
          </h1>
          <p class="text-sm text-muted-foreground">
            Components come from the local shadcn-vue registry and share the Tailwind theme in
            <code>src/styles/index.css</code>.
          </p>
        </div>
        <Badge variant="secondary" class="px-3 py-1 font-medium">Live preview</Badge>
      </section>

      <Card class="shadow-md">
        <CardHeader class="gap-2 pb-0">
          <CardTitle>Theme preview</CardTitle>
          <CardDescription>Toggle dark mode and try the base form controls.</CardDescription>
        </CardHeader>

        <CardContent class="space-y-6 pt-4">
          <div class="flex items-center justify-between gap-3">
            <div class="space-y-1">
              <p class="text-sm font-semibold">Dark mode</p>
              <p class="text-sm text-muted-foreground">
                Applies the <code>.dark</code> class on <code>&lt;html&gt;</code>.
              </p>
            </div>
            <Switch v-model:checked="isDark" aria-label="Toggle dark mode" />
          </div>

          <Separator />

          <div class="space-y-2">
            <Label for="botName">Bot name</Label>
            <Input id="botName" v-model="botName" placeholder="Give your bot a name" />
          </div>
        </CardContent>

        <CardFooter class="flex flex-wrap items-center gap-3">
          <Button @click="handleSave">
            Save & toast
          </Button>
          <Button variant="outline">
            Outline
          </Button>
          <Button variant="secondary">
            Secondary
          </Button>
          <Button variant="ghost">
            Ghost
          </Button>
          <Button variant="destructive">
            Destructive
          </Button>
        </CardFooter>
      </Card>
    </main>

    <Toaster position="top-center" rich-colors close-button />
  </div>
</template>
