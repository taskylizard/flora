import { useEffect, useMemo, useState } from "react"
import {
  createToken,
  deleteToken,
  fetchDeployment,
  fetchDeployments,
  fetchGuilds,
  fetchSession,
  fetchTokens,
  redirectToLogin,
  saveDeployment,
  type AuthUser,
  type Deployment,
  type Guild,
  type Language,
  type Token,
} from "@/lib/api"
import { formatDistanceToNow } from "date-fns"
import {
  LayoutDashboard,
  Braces,
  ShieldCheck,
  Shield,
  UploadCloud,
  LogOut,
  RefreshCw,
  Moon,
  Sun,
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Textarea } from "@/components/ui/textarea"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { Avatar } from "@/components/ui/avatar"
import { cn } from "@/lib/utils"
import { useTheme } from "@/lib/theme"

function formatTimeAgo(value?: string | null) {
  if (!value) return "never"
  return formatDistanceToNow(new Date(value), { addSuffix: true })
}

type LoadState<T> = {
  data: T | null
  loading: boolean
  error: string | null
}

const initialState = { data: null, loading: true, error: null }

export function Dashboard() {
  const [session, setSession] = useState<AuthUser | null>(null)
  const [sessionError, setSessionError] = useState<string | null>(null)

  const [guilds, setGuilds] = useState<LoadState<Guild[]>>({ ...initialState })
  const [deployments, setDeployments] = useState<LoadState<Deployment[]>>({ ...initialState })
  const [tokens, setTokens] = useState<LoadState<Token[]>>({ ...initialState })

  const [selectedGuild, setSelectedGuild] = useState<string>("")
  const [language, setLanguage] = useState<Language>("typescript")
  const [code, setCode] = useState<string>("// Write your guild bot here\nexport default async function main(ctx) {\n  ctx.reply('Hello from Oakmoss!')\n}\n")
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "saved" | "error">(
    "idle"
  )
  const [saveError, setSaveError] = useState<string | null>(null)
  const [isCodeLoading, setIsCodeLoading] = useState(false)
  const [newToken, setNewToken] = useState<string | null>(null)
  const [tokenLabel, setTokenLabel] = useState<string>("")
  const { theme, toggleTheme } = useTheme()

  // bootstrap session
  useEffect(() => {
    let mounted = true
    fetchSession()
      .then((res) => {
        if (!mounted) return
        setSession(res.user)
        setSessionError(null)
      })
      .catch((err) => {
        if (!mounted) return
        if (err.status === 401) {
          setSession(null)
        } else {
          setSessionError(err.message || "Failed to load session")
        }
      })
    return () => {
      mounted = false
    }
  }, [])

  // load lists once session is ready
  useEffect(() => {
    if (!session) return

    setGuilds({ ...initialState })
    fetchGuilds()
      .then((res) => setGuilds({ data: res, loading: false, error: null }))
      .catch((err) => setGuilds({ data: null, loading: false, error: err.message }))

    setDeployments({ ...initialState })
    fetchDeployments()
      .then((res) => {
        setDeployments({ data: res, loading: false, error: null })
        if (!selectedGuild && res.length > 0) {
          setSelectedGuild(res[0].guild_id)
          setLanguage(res[0].language as Language)
        }
      })
      .catch((err) => setDeployments({ data: null, loading: false, error: err.message }))

    setTokens({ ...initialState })
    fetchTokens()
      .then((res) => setTokens({ data: res, loading: false, error: null }))
      .catch((err) => setTokens({ data: null, loading: false, error: err.message }))
  }, [session])

  // when user switches guild, load latest deployment code and meta
  useEffect(() => {
    if (!selectedGuild) return

    setIsCodeLoading(true)
    fetchDeployment(selectedGuild)
      .then((dep) => {
        if (dep.language) setLanguage(dep.language as Language)
        if (dep.source) {
          setCode(dep.source)
        } else {
          setCode(
            "// Write your guild bot here\nexport default async function main(ctx) {\n  ctx.reply('Hello from Oakmoss!')\n}\n"
          )
        }
      })
      .catch(() => {
        setCode(
          "// Write your guild bot here\nexport default async function main(ctx) {\n  ctx.reply('Hello from Oakmoss!')\n}\n"
        )
      })
      .finally(() => setIsCodeLoading(false))
  }, [selectedGuild])

  const selectedGuildInfo = useMemo(
    () => guilds.data?.find((g) => g.id === selectedGuild),
    [guilds.data, selectedGuild]
  )

  const deploymentRows = deployments.data ?? []

  const handleSave = async () => {
    if (!selectedGuild) return
    setSaveStatus("saving")
    setSaveError(null)
    try {
      await saveDeployment(selectedGuild, { code, language })
      setSaveStatus("saved")
      // refresh deployments list for updated timestamps
      const refreshed = await fetchDeployments()
      setDeployments({ data: refreshed, loading: false, error: null })
      setTimeout(() => setSaveStatus("idle"), 2000)
    } catch (err: any) {
      setSaveStatus("error")
      setSaveError(err.message || "Failed to save")
    }
  }

  const handleCreateToken = async () => {
    try {
      const res = await createToken(tokenLabel || undefined)
      setNewToken(res.token)
      setTokenLabel("")
      const refreshed = await fetchTokens()
      setTokens({ data: refreshed, loading: false, error: null })
    } catch (err: any) {
      setNewToken(null)
      setTokens((prev) => ({ ...prev, error: err.message }))
    }
  }

  const handleDeleteToken = async (tokenId: string) => {
    await deleteToken(tokenId)
    const refreshed = await fetchTokens()
    setTokens({ data: refreshed, loading: false, error: null })
  }

  if (sessionError) {
    return (
      <FullScreenMessage
        title="API unreachable"
        description={sessionError}
        actionLabel="Retry"
        onAction={() => window.location.reload()}
      />
    )
  }

  if (!session) {
    return (
      <FullScreenMessage
        title="Welcome to Oakmoss"
        description="Sign in with Discord to manage guild deployments and tokens."
        actionLabel="Sign in with Discord"
        onAction={redirectToLogin}
      />
    )
  }

  return (
    <AppShell
      user={session}
      onLogout={redirectToLogin}
      theme={theme}
      onToggleTheme={toggleTheme}
    >
      <div className="grid gap-6 lg:grid-cols-[320px_1fr] xl:grid-cols-[360px_1fr]">
        <Panel>
          <CardHeader className="pb-2">
            <CardTitle className="text-lg">Your guilds</CardTitle>
            <CardDescription>Pick a guild to deploy your bot</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {guilds.loading && <Skeleton className="h-10 w-full" />}
            {guilds.error && <InlineAlert message={guilds.error} />}
            {!guilds.loading && !guilds.data?.length && (
              <EmptyState
                icon={Shield}
                title="No admin guilds yet"
                description="Invite the bot to a guild where you have admin permissions."
              />
            )}
            {guilds.data?.map((guild) => (
              <button
                key={guild.id}
                onClick={() => setSelectedGuild(guild.id)}
                className={cn(
                  "w-full rounded-lg border p-3 text-left transition hover:border-primary hover:bg-primary/5",
                  selectedGuild === guild.id && "border-primary bg-primary/5"
                )}
              >
                <div className="flex items-center gap-3">
                  <Avatar name={guild.name} />
                  <div className="flex flex-col">
                    <span className="font-medium">{guild.name}</span>
                    <span className="text-muted-foreground text-sm">{guild.id}</span>
                  </div>
                </div>
              </button>
            ))}
          </CardContent>
        </Panel>

        <div className="space-y-6">
          <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
            <MetricCard
              icon={Braces}
              label="Deployments"
              value={deploymentRows.length.toString()}
              helper="Managed guilds"
            />
            <MetricCard
              icon={ShieldCheck}
              label="API tokens"
              value={tokens.data?.length?.toString() || "0"}
              helper="Use for CLI access"
            />
          </div>

          <Panel>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <CardTitle className="text-lg">Deploy code</CardTitle>
                  <CardDescription>
                    Ship your script to the selected guild. Existing code is not returned by the API,
                    so paste the full source you want to run.
                  </CardDescription>
                </div>
                <Badge variant="secondary" className="uppercase">
                  {selectedGuildInfo ? selectedGuildInfo.name : "Select a guild"}
                </Badge>
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="grid gap-3 sm:grid-cols-[180px_1fr]">
                <Select value={language} onValueChange={(v) => setLanguage(v as Language)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="typescript">TypeScript</SelectItem>
                    <SelectItem value="javascript">JavaScript</SelectItem>
                  </SelectContent>
                </Select>
                <Textarea
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  rows={12}
                  className={cn("font-mono text-sm", isCodeLoading && "opacity-50")}
                  placeholder={isCodeLoading ? "Loading..." : "// paste bot code"}
                  disabled={!selectedGuild || isCodeLoading}
                />
              </div>
              <div className="flex flex-wrap items-center gap-3">
                <Button onClick={handleSave} disabled={!selectedGuild || saveStatus === "saving"}>
                  {saveStatus === "saving" && (
                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                  )}
                  <UploadCloud className="mr-2 h-4 w-4" />
                  Deploy to guild
                </Button>
                {saveStatus === "saved" && <Badge variant="secondary">Saved</Badge>}
                {saveStatus === "error" && saveError && <InlineAlert message={saveError} />}
              </div>
            </CardContent>
          </Panel>

          <Panel>
            <CardHeader className="pb-3">
              <CardTitle className="text-lg">Deployment history</CardTitle>
              <CardDescription>Latest entries from the API</CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              {deployments.loading && <Skeleton className="h-10 w-full" />}
              {deployments.error && <InlineAlert message={deployments.error} />}
              {!deployments.loading && !deploymentRows.length && (
                <EmptyState
                  icon={Braces}
                  title="No deployments yet"
                  description="Save your first script to see it here."
                />
              )}
              {!!deploymentRows.length && (
                <div className="grid gap-2 text-sm">
                  {deploymentRows.map((dep) => (
                    <div
                      key={dep.guild_id}
                      className="flex flex-wrap items-center justify-between gap-3 rounded-lg border bg-card px-4 py-3"
                    >
                      <div>
                        <div className="font-medium">{dep.guild_id}</div>
                        <div className="text-muted-foreground">Updated {formatTimeAgo(dep.updated_at)}</div>
                      </div>
                      <Badge>{dep.language}</Badge>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Panel>

          <Panel>
            <CardHeader className="pb-3">
              <CardTitle className="text-lg">API tokens</CardTitle>
              <CardDescription>Mint tokens for CLI or automation</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
                <Input
                  placeholder="Label (optional)"
                  value={tokenLabel}
                  onChange={(e) => setTokenLabel(e.target.value)}
                  className="sm:max-w-xs"
                />
                <Button onClick={handleCreateToken} className="w-full sm:w-auto">
                  <ShieldCheck className="mr-2 h-4 w-4" />
                  Create token
                </Button>
              </div>
              {newToken && (
                <div className="rounded-lg border bg-muted/40 p-3 text-sm font-mono">
                  {newToken}
                  <div className="text-muted-foreground mt-1 text-xs">Copy now; it will not be shown again.</div>
                </div>
              )}
              {tokens.loading && <Skeleton className="h-10 w-full" />}
              {tokens.error && <InlineAlert message={tokens.error} />}
              {!tokens.loading && !tokens.data?.length && (
                <EmptyState
                  icon={Shield}
                  title="No tokens yet"
                  description="Create a token to authenticate external tools."
                />
              )}
              {!!tokens.data?.length && (
                <div className="space-y-2 text-sm">
                  {tokens.data.map((tok) => (
                    <div
                      key={tok.token_id}
                      className="flex flex-wrap items-center justify-between gap-3 rounded-lg border bg-card px-4 py-3"
                    >
                      <div>
                        <div className="font-medium">{tok.label || "Unlabeled"}</div>
                        <div className="text-muted-foreground">
                          Created {formatTimeAgo(tok.created_at)} • Last used {formatTimeAgo(tok.last_used_at)}
                        </div>
                      </div>
                      <Button variant="ghost" size="sm" onClick={() => handleDeleteToken(tok.token_id)}>
                        Revoke
                      </Button>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Panel>
        </div>
      </div>
    </AppShell>
  )
}

function AppShell({
  user,
  onLogout,
  children,
  theme,
  onToggleTheme,
}: {
  user: AuthUser
  onLogout: () => void
  children: React.ReactNode
  theme: "light" | "dark"
  onToggleTheme: () => void
}) {
  return (
    <div className="bg-background text-foreground min-h-screen">
      <header className="border-border/60 bg-card/60 sticky top-0 z-20 border-b backdrop-blur">
        <div className="mx-auto flex max-w-6xl items-center justify-between gap-4 px-4 py-3">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
              <LayoutDashboard className="h-5 w-5" />
            </div>
            <div>
              <div className="font-semibold">Oakmoss Console</div>
              <div className="text-muted-foreground text-sm">Guild runtime & tokens</div>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="icon" onClick={onToggleTheme}>
              <span className="sr-only">Toggle theme</span>
              {theme === "dark" ? <Moon className="h-4 w-4" /> : <Sun className="h-4 w-4" />}
            </Button>
            <div className="flex items-center gap-2 rounded-full border px-3 py-1">
              <Avatar name={user.global_name || user.username} />
              <div className="leading-tight">
                <div className="text-sm font-medium">{user.global_name || user.username}</div>
                <div className="text-muted-foreground text-xs">{user.id}</div>
              </div>
            </div>
            <Button variant="ghost" size="icon" onClick={onLogout}>
              <LogOut className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </header>
      <main className="mx-auto max-w-6xl px-4 py-8">{children}</main>
    </div>
  )
}

function Panel({ children }: { children: React.ReactNode }) {
  return (
    <Card className="border-border/80 shadow-sm">
      {children}
    </Card>
  )
}

function InlineAlert({ message }: { message: string }) {
  return (
    <div className="text-destructive/90 rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2 text-sm">
      {message}
    </div>
  )
}

function EmptyState({
  icon: Icon,
  title,
  description,
}: {
  icon: React.ComponentType<{ className?: string }>
  title: string
  description: string
}) {
  return (
    <div className="flex flex-col items-start gap-2 rounded-lg border border-dashed px-4 py-6">
      <Icon className="text-muted-foreground h-6 w-6" />
      <div className="font-medium">{title}</div>
      <div className="text-muted-foreground text-sm">{description}</div>
    </div>
  )
}

function MetricCard({ icon: Icon, label, value, helper }: { icon: React.ComponentType<{ className?: string }>; label: string; value: string; helper: string }) {
  return (
    <div className="rounded-xl border bg-card p-4 shadow-sm">
      <div className="flex items-center justify-between">
        <div className="text-muted-foreground text-sm">{label}</div>
        <div className="rounded-lg bg-primary/10 p-2 text-primary">
          <Icon className="h-4 w-4" />
        </div>
      </div>
      <div className="mt-2 text-2xl font-semibold">{value}</div>
      <div className="text-muted-foreground text-xs">{helper}</div>
    </div>
  )
}

function FullScreenMessage({
  title,
  description,
  actionLabel,
  onAction,
}: {
  title: string
  description: string
  actionLabel: string
  onAction: () => void
}) {
  return (
    <div className="bg-background text-foreground flex min-h-screen items-center justify-center px-6">
      <div className="max-w-md space-y-4 text-center">
        <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-2xl bg-primary/10 text-primary">
          <LayoutDashboard className="h-6 w-6" />
        </div>
        <div className="text-2xl font-semibold">{title}</div>
        <div className="text-muted-foreground">{description}</div>
        <div className="flex justify-center">
          <Button onClick={onAction}>{actionLabel}</Button>
        </div>
      </div>
    </div>
  )
}
