<script setup lang="ts">
import { RouterView } from 'vue-router'

import { Toaster } from '@/components/ui/sonner'
</script>

<template>
  <div class="min-h-screen bg-background text-foreground">
    <RouterView />
    <Toaster position="top-center" rich-colors close-button />
  </div>
</template>
