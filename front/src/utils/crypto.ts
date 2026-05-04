export async function hashEndpoint(endpoint: string): Promise<string> {
  const trimmed = endpoint.trim()
  if (!trimmed) return ''

  const encoder = new TextEncoder()
  const data = encoder.encode(trimmed)
  const hashBuffer = await crypto.subtle.digest('SHA-256', data)
  const hashArray = Array.from(new Uint8Array(hashBuffer))
  return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('')
}
