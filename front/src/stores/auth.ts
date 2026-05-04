import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { UserInfo } from '@/types/api'
import { exchangeGithubCode, getAnonymousToken, getGithubLoginUrl } from '@/api/auth'

const OAUTH_KEY = 'oauth_token'
const ANON_KEY = 'anon_token'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<UserInfo | null>(null)
  const token = ref<string | null>(null)
  const tokenType = ref<'oauth' | 'anonymous' | null>(null)

  const isLoggedIn = computed(() => token.value !== null)

  function loadFromStorage() {
    const oauthToken = localStorage.getItem(OAUTH_KEY)
    if (oauthToken) {
      token.value = oauthToken
      tokenType.value = 'oauth'
      try {
        const stored = localStorage.getItem('oauth_user')
        if (stored) user.value = JSON.parse(stored)
      } catch {
        // ignore parse errors
      }
      return
    }

    const anonToken = sessionStorage.getItem(ANON_KEY)
    if (anonToken) {
      token.value = anonToken
      tokenType.value = 'anonymous'
    }
  }

  function loginWithGithub() {
    sessionStorage.setItem('login_redirect', window.location.pathname)
    window.location.href = getGithubLoginUrl()
  }

  async function handleCallback(code: string): Promise<void> {
    const resp = await exchangeGithubCode(code)
    token.value = resp.token
    tokenType.value = 'oauth'
    user.value = resp.user
    localStorage.setItem(OAUTH_KEY, resp.token)
    if (resp.user) {
      localStorage.setItem('oauth_user', JSON.stringify(resp.user))
    }
  }

  async function handleAnonymousAuth(turnstileToken: string): Promise<void> {
    const resp = await getAnonymousToken(turnstileToken)
    token.value = resp.token
    tokenType.value = 'anonymous'
    sessionStorage.setItem(ANON_KEY, resp.token)
  }

  function logout() {
    token.value = null
    user.value = null
    tokenType.value = null
    localStorage.removeItem(OAUTH_KEY)
    localStorage.removeItem('oauth_user')
    sessionStorage.removeItem(ANON_KEY)
  }

  function getLoginRedirect(): string {
    const redirect = sessionStorage.getItem('login_redirect')
    sessionStorage.removeItem('login_redirect')
    return redirect || '/'
  }

  return {
    user,
    token,
    tokenType,
    isLoggedIn,
    loadFromStorage,
    loginWithGithub,
    handleCallback,
    handleAnonymousAuth,
    logout,
    getLoginRedirect,
  }
})
