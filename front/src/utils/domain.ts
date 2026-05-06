export function extractDomain(endpoint: string): string {
  const trimmed = endpoint.trim()
  if (!trimmed) return ''

  try {
    const url = /^https?:\/\//i.test(trimmed)
      ? new URL(trimmed)
      : new URL(`https://${trimmed}`)
    return url.hostname
  } catch {
    return ''
  }
}

const LAN_HOSTNAMES = new Set(['localhost', 'localhost.localdomain'])

export function isLanIP(hostname: string): boolean {
  if (!hostname) return false

  const h = hostname.trim().toLowerCase()

  if (LAN_HOSTNAMES.has(h)) return true

  if (h === '::1' || h === '0.0.0.0') return true

  const parts = h.split('.').map(Number)
  if (parts.length === 4 && parts.every((n) => n >= 0 && n <= 255)) {
    const a = parts[0]!
    const b = parts[1]!
    if (a === 10) return true
    if (a === 127) return true
    if (a === 169 && b === 254) return true
    if (a === 172 && b >= 16 && b <= 31) return true
    if (a === 192 && b === 168) return true
  }

  return false
}
