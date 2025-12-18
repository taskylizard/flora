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
  Code2,
  History,
  Settings,
  Shield,
  LogOut,
  Moon,
  Sun,
  Menu,
  ChevronRight,
  Play,
  Terminal,
  Clock,
  CheckCircle2,
  XCircle,
  Plus,
  Trash2,
  Copy,
  Server,
  Activity,
  Box
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import Monaco from "@uwu/monaco-react"
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

type Tab = "overview" | "editor" | "deployments" | "settings"

export function Dashboard() {
  const [session, setSession] = useState<AuthUser | null>(null)
  const [sessionError, setSessionError] = useState<string | null>(null)

  const [guilds, setGuilds] = useState<LoadState<Guild[]>>({ ...initialState })
  const [deployments, setDeployments] = useState<LoadState<Deployment[]>>({ ...initialState })
  const [tokens, setTokens] = useState<LoadState<Token[]>>({ ...initialState })

  const [selectedGuild, setSelectedGuild] = useState<string>("")
  const [activeTab, setActiveTab] = useState<Tab>("overview")
  const [sidebarOpen, setSidebarOpen] = useState(false)
  
  const [language, setLanguage] = useState<Language>("typescript")
  const [code, setCode] = useState<string>("")
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "saved" | "error">("idle")
  const [saveError, setSaveError] = useState<string | null>(null)
  const [isCodeLoading, setIsCodeLoading] = useState(false)

  const [newToken, setNewToken] = useState<string | null>(null)
  const [tokenLabel, setTokenLabel] = useState<string>("")

  const { theme, toggleTheme } = useTheme()

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

  useEffect(() => {
    if (!selectedGuild) return

    setIsCodeLoading(true)
    fetchDeployment(selectedGuild)
      .then((dep) => {
        if (dep.language) setLanguage(dep.language as Language)
        if (dep.source) {
          setCode(dep.source)
        } else {
          setCode(getDefaultCode())
        }
      })
      .catch(() => {
        setCode(getDefaultCode())
      })
      .finally(() => setIsCodeLoading(false))
  }, [selectedGuild])

  const getDefaultCode = () => 
    "// Write your guild bot here\nexport default async function main(ctx) {\n  ctx.reply('Hello from Oakmoss!')\n}\n"

  const handleSave = async () => {
    if (!selectedGuild) return
    setSaveStatus("saving")
    setSaveError(null)
    try {
      await saveDeployment(selectedGuild, { code, language })
      setSaveStatus("saved")
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

  const selectedGuildInfo = useMemo(
    () => guilds.data?.find((g) => g.id === selectedGuild),
    [guilds.data, selectedGuild]
  )

  if (sessionError) {
    return (
      <FullScreenMessage
        icon={XCircle}
        title="Connection Failed"
        description={sessionError}
        actionLabel="Retry"
        onAction={() => window.location.reload()}
      />
    )
  }

  if (!session) {
    return (
      <FullScreenMessage
        icon={LayoutDashboard}
        title="Welcome to Oakmoss"
        description="Sign in with Discord to manage your guild deployments."
        actionLabel="Sign in with Discord"
        onAction={redirectToLogin}
      />
    )
  }

  return (
    <div className="flex h-screen w-full bg-background text-foreground overflow-hidden font-sans">
      {sidebarOpen && (
        <div 
          className="fixed inset-0 z-40 bg-black/50 lg:hidden backdrop-blur-sm"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      <aside className={cn(
        "fixed inset-y-0 left-0 z-50 flex w-72 flex-col border-r bg-sidebar text-sidebar-foreground transition-transform duration-300 lg:relative lg:translate-x-0",
        sidebarOpen ? "translate-x-0" : "-translate-x-full"
      )}>
        <div className="flex h-16 items-center gap-2 border-b px-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-primary-foreground font-bold">
            OM
          </div>
          <div className="font-semibold tracking-tight">Oakmoss</div>
        </div>

        <div className="flex-1 overflow-y-auto py-4 px-3 space-y-1">
          <div className="px-3 mb-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
            Your Guilds
          </div>
          
          {guilds.loading && (
            <div className="px-3 space-y-2">
              <Skeleton className="h-9 w-full" />
              <Skeleton className="h-9 w-full" />
              <Skeleton className="h-9 w-full" />
            </div>
          )}

          {!guilds.loading && guilds.data?.length === 0 && (
             <div className="px-3 py-4 text-center border-2 border-dashed rounded-lg">
                <Shield className="h-5 w-5 mx-auto text-muted-foreground mb-2" />
                <p className="text-xs text-muted-foreground">No admin guilds found</p>
             </div>
          )}

          {guilds.data?.map((guild) => (
            <button
              key={guild.id}
              onClick={() => {
                setSelectedGuild(guild.id)
                if (window.innerWidth < 1024) setSidebarOpen(false)
              }}
              className={cn(
                "group flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                selectedGuild === guild.id 
                  ? "bg-sidebar-primary text-sidebar-primary-foreground shadow-sm" 
                  : "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground text-muted-foreground"
              )}
            >
              <Avatar 
                name={guild.name} 
                guildId={guild.id}
                iconHash={guild.icon}
                className={cn("h-6 w-6 text-[10px]", selectedGuild === guild.id ? "bg-sidebar-primary-foreground/20 text-sidebar-primary-foreground" : "bg-muted")} 
              />
              <span className="truncate">{guild.name}</span>
              {selectedGuild === guild.id && <ChevronRight className="ml-auto h-4 w-4 opacity-50" />}
            </button>
          ))}
        </div>

        <div className="border-t p-4">
          <div className="flex items-center gap-3 rounded-lg border bg-sidebar-accent/50 p-3 shadow-sm">
            <Avatar 
              name={session.global_name || session.username} 
              userId={session.id}
              avatarHash={session.avatar}
              className="h-8 w-8" 
            />
            <div className="flex-1 min-w-0">
              <p className="truncate text-sm font-medium">{session.global_name || session.username}</p>
              <button onClick={redirectToLogin} className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors">
                <LogOut className="h-3 w-3" />
                Sign out
              </button>
            </div>
          </div>
        </div>
      </aside>

      <main className="flex min-w-0 flex-1 flex-col bg-background">
        <header className="sticky top-0 z-30 flex h-16 items-center gap-4 border-b bg-background/95 px-6 backdrop-blur supports-[backdrop-filter]:bg-background/60">
          <Button variant="ghost" size="icon" className="lg:hidden -ml-2" onClick={() => setSidebarOpen(true)}>
            <Menu className="h-5 w-5" />
          </Button>

          <div className="flex items-center gap-2">
            <span className="text-muted-foreground hidden sm:inline-block">/</span>
            <span className="font-medium">
              {selectedGuildInfo ? selectedGuildInfo.name : "Select a Guild"}
            </span>
          </div>

          <div className="ml-auto flex items-center gap-2">
            <Button variant="ghost" size="icon" onClick={toggleTheme}>
              {theme === "dark" ? <Moon className="h-4 w-4" /> : <Sun className="h-4 w-4" />}
            </Button>
          </div>
        </header>

        {selectedGuild && (
            <div className="border-b px-6 bg-muted/20">
              <nav className="flex items-center gap-6 overflow-x-auto">
                <TabButton 
                  active={activeTab === 'overview'} 
                  onClick={() => setActiveTab('overview')} 
                  icon={LayoutDashboard} 
                  label="Overview" 
                />
                <TabButton 
                  active={activeTab === 'editor'} 
                  onClick={() => setActiveTab('editor')} 
                  icon={Code2} 
                  label="Editor" 
                />
                <TabButton 
                  active={activeTab === 'deployments'} 
                  onClick={() => setActiveTab('deployments')} 
                  icon={History} 
                  label="Deployments" 
                />
                <TabButton 
                  active={activeTab === 'settings'} 
                  onClick={() => setActiveTab('settings')} 
                  icon={Settings} 
                  label="Settings" 
                />
              </nav>
            </div>
        )}

        <div className={cn(
          "flex-1 p-4 md:p-6 lg:p-8",
          activeTab === 'editor' ? "flex flex-col overflow-hidden" : "overflow-y-auto"
        )}>
          {!selectedGuild ? (
            <div className="flex h-full flex-col items-center justify-center text-center animate-in fade-in zoom-in-95 duration-500">
              <div className="rounded-full bg-primary/10 p-6 mb-4">
                <Server className="h-10 w-10 text-primary" />
              </div>
              <h2 className="text-2xl font-bold tracking-tight">No Guild Selected</h2>
              <p className="text-muted-foreground mt-2 max-w-sm">
                Select a guild from the sidebar to manage its bot deployment, view history, or configure settings.
              </p>
            </div>
          ) : (
            <div className={cn(
              "mx-auto w-full animate-in fade-in slide-in-from-bottom-4 duration-500",
              activeTab === 'editor' ? "flex-1 flex flex-col min-h-0 max-w-6xl" : "max-w-6xl space-y-6"
            )}>
              
              {activeTab === 'overview' && (
                <div className="grid gap-6">
                  <div className="grid gap-4 md:grid-cols-3">
                    <MetricCard 
                       icon={Activity} 
                       label="Total Deployments" 
                       value={deployments.data?.length.toString() || "0"} 
                       description="Across all guilds"
                    />
                    <MetricCard 
                       icon={Box} 
                       label="Current Language" 
                       value={language === 'typescript' ? 'TS' : 'JS'} 
                       description={language}
                    />
                    <MetricCard 
                       icon={Clock} 
                       label="Last Updated" 
                       value={deployments.data?.find(d => d.guild_id === selectedGuild)?.updated_at ? formatTimeAgo(deployments.data.find(d => d.guild_id === selectedGuild)?.updated_at) : "Never"} 
                       description="Deployment age"
                    />
                  </div>

                  <div className="grid gap-6 md:grid-cols-2">
                    <Card>
                        <CardHeader>
                            <CardTitle>Quick Actions</CardTitle>
                            <CardDescription>Common tasks for {selectedGuildInfo?.name}</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-2">
                            <Button className="w-full justify-start" onClick={() => setActiveTab('editor')}>
                                <Code2 className="mr-2 h-4 w-4" />
                                Edit & Deploy Code
                            </Button>
                            <Button variant="outline" className="w-full justify-start" onClick={() => setActiveTab('deployments')}>
                                <History className="mr-2 h-4 w-4" />
                                View Deployment History
                            </Button>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle>System Status</CardTitle>
                            <CardDescription>Runtime Environment</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                             <div className="flex items-center justify-between">
                                <span className="text-sm text-muted-foreground">API Connection</span>
                                <Badge variant="outline" className="text-green-600 border-green-200 bg-green-50 dark:bg-green-900/20 dark:border-green-900">Online</Badge>
                             </div>
                             <div className="flex items-center justify-between">
                                <span className="text-sm text-muted-foreground">Runtime Version</span>
                                <span className="text-sm font-mono">v1.0.0-alpha</span>
                             </div>
                        </CardContent>
                    </Card>
                  </div>
                </div>
              )}

              {activeTab === 'editor' && (
                <div className="flex flex-col flex-1 min-h-0 gap-6">
                   <Card className="flex-1 flex flex-col min-h-0 overflow-hidden border-border/60 shadow-md">
                      <div className="flex items-center justify-between border-b px-4 py-2 bg-muted/30">
                         <div className="flex items-center gap-3">
                            <Terminal className="h-4 w-4 text-muted-foreground" />
                            <Select value={language} onValueChange={(v) => setLanguage(v as Language)}>
                                <SelectTrigger className="h-8 w-[140px] bg-background border-border/50">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="typescript">TypeScript</SelectItem>
                                    <SelectItem value="javascript">JavaScript</SelectItem>
                                </SelectContent>
                            </Select>
                         </div>
                         <div className="flex items-center gap-2">
                            {saveStatus === "error" && saveError && (
                                <span className="text-xs text-destructive flex items-center gap-1 animate-in fade-in">
                                    <XCircle className="h-3 w-3" /> {saveError}
                                </span>
                            )}
                            {saveStatus === "saved" && (
                                <span className="text-xs text-green-600 flex items-center gap-1 animate-in fade-in">
                                    <CheckCircle2 className="h-3 w-3" /> Saved
                                </span>
                            )}
                            <Button 
                                size="sm" 
                                onClick={handleSave} 
                                disabled={saveStatus === "saving" || isCodeLoading}
                                className={cn("transition-all", saveStatus === "saved" ? "bg-green-600 hover:bg-green-700" : "")}
                            >
                                {saveStatus === "saving" ? (
                                    <Clock className="mr-2 h-3 w-3 animate-spin" />
                                ) : (
                                    <Play className="mr-2 h-3 w-3 fill-current" />
                                )}
                                {saveStatus === "saved" ? "Deployed" : "Deploy"}
                            </Button>
                         </div>
                      </div>
                      <div className="relative flex-1 bg-zinc-950">
                          {isCodeLoading && (
                             <div className="absolute inset-0 z-10 flex items-center justify-center bg-background/50 backdrop-blur-sm">
                                 <Clock className="h-8 w-8 animate-spin text-primary" />
                             </div>
                          )}
                          <Monaco
                              value={code}
                              valOut={setCode}
                              lang={language}
                              theme="vs-dark"
                              height="100%"
                              width="100%"
                              readonly={isCodeLoading}
                              otherCfg={{
                                minimap: { enabled: false },
                                fontSize: 14,
                                lineNumbers: "on",
                                scrollBeyondLastLine: false,
                                automaticLayout: true,
                                padding: { top: 16, bottom: 16 }
                              }}
                          />
                      </div>
                   </Card>
                   <div className="text-xs text-muted-foreground px-1">
                      <span className="font-semibold">Tip:</span> The <code>main</code> function is the entry point for your bot. It receives a context object.
                   </div>
                </div>
              )}

              {activeTab === 'deployments' && (
                <Card>
                    <CardHeader>
                        <CardTitle>Deployment History</CardTitle>
                        <CardDescription>Recent updates to your guild bots.</CardDescription>
                    </CardHeader>
                    <CardContent>
                        {!deployments.data?.length ? (
                            <EmptyState 
                                icon={History}
                                title="No deployments"
                                description="You haven't deployed any code yet."
                            />
                        ) : (
                            <div className="space-y-4">
                                {deployments.data.filter(d => d.guild_id === selectedGuild || !selectedGuild).map((dep) => (
                                    <div key={dep.guild_id} className="flex items-center justify-between rounded-lg border p-4 transition-colors hover:bg-muted/50">
                                        <div className="flex items-center gap-4">
                                            <div className="rounded-full bg-primary/10 p-2 text-primary">
                                                <Code2 className="h-4 w-4" />
                                            </div>
                                            <div>
                                                <p className="font-medium text-sm">
                                                    {guilds.data?.find(g => g.id === dep.guild_id)?.name || dep.guild_id}
                                                </p>
                                                <p className="text-xs text-muted-foreground">
                                                    Deployed {formatTimeAgo(dep.updated_at)}
                                                </p>
                                            </div>
                                        </div>
                                        <div className="flex items-center gap-3">
                                            <Badge variant="secondary" className="font-mono text-xs">
                                                {dep.language}
                                            </Badge>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </CardContent>
                </Card>
              )}

              {activeTab === 'settings' && (
                <div className="space-y-6">
                    <Card>
                        <CardHeader>
                            <CardTitle>API Tokens</CardTitle>
                            <CardDescription>Manage access tokens for the CLI and external tools.</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-6">
                            <div className="flex flex-col gap-4 sm:flex-row">
                                <div className="flex-1">
                                    <Input 
                                        placeholder="Token Label (e.g. CI/CD Pipeline)" 
                                        value={tokenLabel}
                                        onChange={(e) => setTokenLabel(e.target.value)}
                                    />
                                </div>
                                <Button onClick={handleCreateToken}>
                                    <Plus className="mr-2 h-4 w-4" />
                                    Generate Token
                                </Button>
                            </div>

                            {newToken && (
                                <div className="rounded-md border border-green-200 bg-green-50 p-4 dark:bg-green-900/20 dark:border-green-900">
                                    <div className="flex items-center justify-between mb-2">
                                        <h4 className="text-sm font-semibold text-green-800 dark:text-green-300">Token Generated</h4>
                                        <Button 
                                            variant="ghost" 
                                            size="sm" 
                                            className="h-6 text-green-700 hover:text-green-800 hover:bg-green-100 dark:text-green-400 dark:hover:bg-green-900/40"
                                            onClick={() => navigator.clipboard.writeText(newToken)}
                                        >
                                            <Copy className="h-3 w-3 mr-1" /> Copy
                                        </Button>
                                    </div>
                                    <code className="block w-full overflow-x-auto rounded bg-white p-2 font-mono text-xs text-green-900 border border-green-100 dark:bg-black/40 dark:text-green-100 dark:border-green-900/50">
                                        {newToken}
                                    </code>
                                    <p className="mt-2 text-xs text-green-700 dark:text-green-400">
                                        Make sure to copy this token now. You won't be able to see it again.
                                    </p>
                                </div>
                            )}

                            <div className="space-y-1">
                                <h4 className="text-sm font-medium text-muted-foreground mb-3">Active Tokens</h4>
                                {tokens.loading && <Skeleton className="h-12 w-full" />}
                                {!tokens.loading && !tokens.data?.length && (
                                    <EmptyState 
                                        icon={Shield}
                                        title="No active tokens"
                                        description="Generate a token to get started with the CLI."
                                    />
                                )}
                                {tokens.data?.map((token) => (
                                    <div key={token.token_id} className="flex items-center justify-between rounded-md border p-3">
                                        <div>
                                            <div className="font-medium text-sm">{token.label || "Untitled Token"}</div>
                                            <div className="text-xs text-muted-foreground">
                                                Created {formatTimeAgo(token.created_at)} • Last used {formatTimeAgo(token.last_used_at)}
                                            </div>
                                        </div>
                                        <Button 
                                            variant="ghost" 
                                            size="sm" 
                                            onClick={() => handleDeleteToken(token.token_id)}
                                            className="text-destructive hover:text-destructive hover:bg-destructive/10"
                                        >
                                            <Trash2 className="h-4 w-4" />
                                        </Button>
                                    </div>
                                ))}
                            </div>
                        </CardContent>
                    </Card>
                </div>
              )}
            </div>
          )}
        </div>
      </main>
    </div>
  )
}

function TabButton({ 
  active, 
  onClick, 
  icon: Icon, 
  label 
}: { 
  active: boolean, 
  onClick: () => void, 
  icon: React.ComponentType<{ className?: string }>, 
  label: string 
}) {
    return (
        <button
            onClick={onClick}
            className={cn(
                "flex items-center gap-2 border-b-2 py-4 text-sm font-medium transition-colors hover:text-primary",
                active 
                  ? "border-primary text-primary" 
                  : "border-transparent text-muted-foreground"
            )}
        >
            <Icon className="h-4 w-4" />
            {label}
        </button>
    )
}

function MetricCard({ 
  icon: Icon, 
  label, 
  value, 
  description 
}: { 
  icon: React.ComponentType<{ className?: string }>, 
  label: string, 
  value: string, 
  description: string 
}) {
    return (
        <Card className="overflow-hidden">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium text-muted-foreground">
                    {label}
                </CardTitle>
                <Icon className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
                <div className="text-2xl font-bold">{value}</div>
                <p className="text-xs text-muted-foreground mt-1">
                    {description}
                </p>
            </CardContent>
        </Card>
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
    <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed py-8 text-center animate-in fade-in zoom-in-95">
      <div className="rounded-full bg-muted p-3">
        <Icon className="h-6 w-6 text-muted-foreground" />
      </div>
      <div className="font-medium mt-2">{title}</div>
      <div className="text-muted-foreground text-sm max-w-xs">{description}</div>
    </div>
  )
}

function FullScreenMessage({
  icon: Icon,
  title,
  description,
  actionLabel,
  onAction,
}: {
  icon: React.ComponentType<{ className?: string }>
  title: string
  description: string
  actionLabel: string
  onAction: () => void
}) {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background p-4 text-foreground">
      <div className="mx-auto flex max-w-[400px] flex-col items-center justify-center text-center space-y-6">
        <div className="flex h-20 w-20 items-center justify-center rounded-3xl bg-primary/10 text-primary ring-8 ring-primary/5">
          <Icon className="h-10 w-10" />
        </div>
        <div className="space-y-2">
            <h1 className="text-2xl font-bold tracking-tight">{title}</h1>
            <p className="text-muted-foreground">{description}</p>
        </div>
        <Button onClick={onAction} size="lg" className="w-full">
          {actionLabel}
        </Button>
      </div>
    </div>
  )
}
