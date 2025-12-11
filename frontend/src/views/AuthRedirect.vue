<script setup lang="ts">
import { onMounted } from 'vue'
import { Loader2Icon } from 'lucide-vue-next'

import { Button } from '@/components/ui/button'

const loginPath = '/api/auth/login'

const goToAuth = () => {
  window.location.assign(loginPath)
}

onMounted(goToAuth)
</script>

<template>
  <div class="flex min-h-screen items-center justify-center bg-background px-4 text-foreground">
    <div class="flex flex-col items-center gap-4 text-center">
      <Loader2Icon class="h-6 w-6 animate-spin text-muted-foreground" aria-hidden="true" />
      <p class="text-sm text-muted-foreground">Redirecting to Discord…</p>
      <Button variant="outline" size="sm" @click="goToAuth">
        Continue
      </Button>
    </div>
  </div>
</template>
