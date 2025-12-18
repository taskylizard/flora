import { cn } from "@/lib/utils"

function getInitials(name?: string | null) {
  if (!name) return "?"
  const parts = name.split(" ").filter(Boolean)
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase()
  return (parts[0][0] + parts[1][0]).toUpperCase()
}

export function Avatar({ name, className }: { name?: string | null; className?: string }) {
  return (
    <div
      className={cn(
        "flex h-9 w-9 items-center justify-center rounded-full bg-primary/10 text-sm font-semibold text-primary",
        className
      )}
    >
      {getInitials(name)}
    </div>
  )
}
