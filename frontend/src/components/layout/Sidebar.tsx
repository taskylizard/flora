import { LogOut } from "lucide-react"
import { Avatar } from "@/components/ui/avatar"
import { cn } from "@/lib/utils"
import { useApp } from "@/contexts/AppContext"
import { redirectToLogin } from "@/lib/api"
import { GuildList } from "@/components/features/GuildList"

export function Sidebar() {
  const { session, sidebarOpen } = useApp()

  if (!session) return null

  return (
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
          <GuildList />
        </div>

        <div className="border-t p-4">
          <div className="flex items-center gap-3 rounded-lg border bg-sidebar-accent/50 p-3 shadow-sm">
            <Avatar name={session.global_name || session.username} className="h-8 w-8" />
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
  )
}
