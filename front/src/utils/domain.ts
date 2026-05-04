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
