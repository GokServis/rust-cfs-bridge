import { makeAutoObservable } from 'mobx'

const STORAGE_KEY = 'bridge-ui-theme'

export type ThemeMode = 'dark' | 'light'

export class ThemeStore {
  theme: ThemeMode = 'dark'

  constructor() {
    makeAutoObservable(this)
    this.hydrate()
  }

  hydrate(): void {
    if (typeof window === 'undefined') return
    const saved = localStorage.getItem(STORAGE_KEY) as ThemeMode | null
    if (saved === 'dark' || saved === 'light') {
      this.theme = saved
    } else if (window.matchMedia('(prefers-color-scheme: light)').matches) {
      this.theme = 'light'
    } else {
      this.theme = 'dark'
    }
    this.applyDom()
  }

  toggle(): void {
    this.theme = this.theme === 'dark' ? 'light' : 'dark'
    this.persist()
  }

  setTheme(mode: ThemeMode): void {
    this.theme = mode
    this.persist()
  }

  private persist(): void {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, this.theme)
    }
    this.applyDom()
  }

  private applyDom(): void {
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', this.theme)
    }
  }
}
