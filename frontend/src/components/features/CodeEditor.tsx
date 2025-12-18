import { useEffect, useState } from "react"
import { CheckCircle2, Clock, Play, Terminal, XCircle } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Textarea } from "@/components/ui/textarea"
import { cn } from "@/lib/utils"
import { useApp } from "@/contexts/AppContext"
import { fetchDeployment, saveDeployment, type Language } from "@/lib/api"

export function CodeEditor() {
  const { selectedGuild, refreshDeployments } = useApp()
  
  const [language, setLanguage] = useState<Language>("typescript")
  const [code, setCode] = useState<string>("")
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "saved" | "error">("idle")
  const [saveError, setSaveError] = useState<string | null>(null)
  const [isCodeLoading, setIsCodeLoading] = useState(false)

  const getDefaultCode = () => 
    "// Write your guild bot here\nexport default async function main(ctx) {\n  ctx.reply('Hello from Oakmoss!')\n}\n"

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

  const handleSave = async () => {
    if (!selectedGuild) return
    setSaveStatus("saving")
    setSaveError(null)
    try {
      await saveDeployment(selectedGuild, { code, language })
      setSaveStatus("saved")
      await refreshDeployments()
      setTimeout(() => setSaveStatus("idle"), 2000)
    } catch (err: any) {
      setSaveStatus("error")
      setSaveError(err.message || "Failed to save")
    }
  }

  return (
    <div className="grid gap-6 h-[calc(100vh-14rem)] grid-rows-[auto_1fr]">
       <Card className="flex flex-col h-full overflow-hidden border-border/60 shadow-md">
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
              <Textarea
                value={code}
                onChange={(e) => setCode(e.target.value)}
                className="h-full w-full resize-none rounded-none border-0 bg-transparent p-4 font-mono text-sm leading-relaxed text-zinc-50 focus-visible:ring-0"
                spellCheck={false}
                placeholder="// Start coding..."
              />
          </div>
       </Card>
       <div className="text-xs text-muted-foreground px-1">
          <span className="font-semibold">Tip:</span> The <code>main</code> function is the entry point for your bot. It receives a context object.
       </div>
    </div>
  )
}
