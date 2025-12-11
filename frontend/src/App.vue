<script setup lang="ts"></script>

<template>
  <div class="min-h-screen bg-background text-foreground">
    <main class="mx-auto flex max-w-3xl flex-col gap-6 px-4 py-12">
      <section class="space-y-2">
        <p class="text-sm font-medium text-muted-foreground">Tailwind v4 + shadcn-vue wiring</p>
        <h1 class="text-3xl font-semibold tracking-tight">
          Frontend tooling is ready
        </h1>
        <p class="text-sm leading-relaxed text-muted-foreground">
          Global styles are loaded from <code>src/styles/index.css</code>. Drop your views and
          components in <code>src/</code> and start building.
        </p>
      </section>

      <section class="rounded-xl border bg-card/60 p-6 shadow-sm">
        <h2 class="text-lg font-semibold">Next steps</h2>
        <ol class="mt-3 space-y-2 text-sm text-muted-foreground">
          <li>1) Add UI components from the shadcn-vue registry to <code>src/components/ui</code>.</li>
          <li>2) Import the pieces you need in your pages.</li>
          <li>3) Customize tokens in <code>src/styles/index.css</code> to match your brand.</li>
        </ol>
      </section>
    </main>
  </div>
</template>
