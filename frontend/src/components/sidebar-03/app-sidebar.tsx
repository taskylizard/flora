"use client";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarTrigger,
  useSidebar,
} from "@/components/ui/sidebar";
import { cn } from "@/lib/utils";
import { motion } from "framer-motion";
import { useApp } from "@/contexts/AppContext";
import { redirectToLogin } from "@/lib/api";
import { NavUser } from "./nav-user";
import DashboardNavigation, { type Route } from "./nav-main";
import { Avatar } from "@/components/ui/avatar";
import { Skeleton } from "@/components/ui/skeleton";
import { Shield } from "lucide-react";
import { Separator } from "../ui/separator";

export function DashboardSidebar() {
  const { state, isMobile, setOpenMobile } = useSidebar();
  const isCollapsed = state === "collapsed";
  const { session, guilds, selectedGuild, setSelectedGuild, view, setView } = useApp();

  const handleGuildClick = (guildId: string) => {
    setSelectedGuild(guildId);
    setView("guild");
    if (isMobile) setOpenMobile(false);
  };

  const routes: Route[] =
    guilds.data?.map((guild) => ({
      id: guild.id,
      title: guild.name,
      icon: (
        <Avatar
          name={guild.name}
          guildId={guild.id}
          iconHash={guild.icon}
          className="h-6 w-6 text-[10px]"
        />
      ),
      isActive: view === "guild" && selectedGuild === guild.id,
      onClick: () => handleGuildClick(guild.id),
    })) || [];

  return (
    <Sidebar variant="floating" collapsible="icon">
      <SidebarHeader
        className={cn(
          "flex md:pt-3.5",
          isCollapsed
            ? "flex-row items-center justify-between gap-y-4 md:flex-col md:items-start md:justify-start"
            : "flex-row items-center justify-between",
        )}
      >
        <div className="flex cursor-pointer items-center gap-2" onClick={() => setView("guild")}>
          <img src="/logo.png" alt="logo" className="h-8 w-8 rounded-lg object-cover" />
          {!isCollapsed && <span className="font-semibold text-black dark:text-white">flora</span>}
        </div>

        <motion.div
          key={isCollapsed ? "header-collapsed" : "header-expanded"}
          className={cn(
            "flex items-center gap-2",
            isCollapsed ? "flex-row md:flex-col-reverse" : "flex-row",
          )}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.8 }}
        >
          <SidebarTrigger />
        </motion.div>
      </SidebarHeader>
      <SidebarContent className={cn("gap-4 px-2", !isCollapsed && "py-4")}>
        {!isCollapsed ? (
          <div className="px-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Your Guilds
          </div>
        ) : (
          <Separator />
        )}
        {guilds.loading ? (
          <div className="space-y-2 px-2">
            <Skeleton className="h-8 w-full" />
            <Skeleton className="h-8 w-full" />
            <Skeleton className="h-8 w-full" />
          </div>
        ) : routes.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed p-4 text-center">
            <Shield className="h-5 w-5 text-muted-foreground" />
            {!isCollapsed && <p className="text-xs text-muted-foreground"> No guilds found </p>}
          </div>
        ) : (
          <DashboardNavigation routes={routes} />
        )}
      </SidebarContent>
      <SidebarFooter className="px-2">
        {session && (
          <NavUser
            user={session}
            onSettingsClick={() => setView("user-settings")}
            onLogoutClick={redirectToLogin}
          />
        )}
      </SidebarFooter>
    </Sidebar>
  );
}
