export type AuthUser = {
  id: string
  username: string
  global_name?: string | null
  avatar?: string | null
}

export type Guild = {
  id: string
  name: string
  icon?: string | null
  permissions: number
}

export type Deployment = {
  guild_id: string
  language: string
  created_at: string
  updated_at: string
  // server does not echo script for safety
}

export type DeploymentDetail = Deployment & {
  // included when fetching single deployment via GET /deployments/{guild}
  source?: string
}

export type Token = {
  token_id: string
  label?: string | null
  created_at: string
  last_used_at?: string | null
}

export type Language = "typescript" | "javascript"

const API_BASE = "/api"

export class ApiError extends Error {
  status: number
  data: unknown

  constructor(status: number, message: string, data?: unknown) {
    super(message)
    this.status = status
    this.data = data
  }
}

async function apiFetch<T>(path: string, init: RequestInit = {}): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: {
      "content-type": "application/json",
      ...(init.headers || {}),
    },
    credentials: "include",
  })

  const text = await res.text()
  let data: unknown
  try {
    data = text ? JSON.parse(text) : undefined
  } catch (err) {
    data = text
  }

  if (!res.ok) {
    throw new ApiError(res.status, (data as any)?.message || res.statusText, data)
  }

  return data as T
}

export async function fetchSession() {
  return apiFetch<{ user: AuthUser }>("/auth/me")
}

export function redirectToLogin() {
  window.location.href = `${API_BASE}/auth/login`
}

export function logoutClientSide() {
  // session cookie is httpOnly; simply bounce to login to refresh.
  window.location.href = `${API_BASE}/auth/login`
}

export async function fetchGuilds() {
  return apiFetch<Guild[]>("/guilds")
}

export async function fetchDeployments() {
  return apiFetch<Deployment[]>("/deployments")
}

export async function fetchDeployment(guildId: string) {
  return apiFetch<DeploymentDetail>(`/deployments/${guildId}`)
}

export async function saveDeployment(
  guildId: string,
  payload: { code: string }
) {
  return apiFetch<Deployment>(`/deployments/${guildId}`, {
    method: "POST",
    body: JSON.stringify(payload),
  })
}

export async function fetchTokens() {
  return apiFetch<Token[]>("/tokens")
}

export async function createToken(label?: string) {
  return apiFetch<{ token: string }>("/tokens", {
    method: "POST",
    body: JSON.stringify({ label }),
  })
}

export async function deleteToken(tokenId: string) {
  return apiFetch<void>(`/tokens/${tokenId}`, { method: "DELETE" })
}
