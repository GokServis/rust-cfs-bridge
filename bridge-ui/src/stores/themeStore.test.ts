import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { ThemeStore } from './themeStore'

beforeEach(() => {
  localStorage.clear()
  vi.stubGlobal(
    'matchMedia',
    vi.fn().mockImplementation(() => ({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  )
})

afterEach(() => {
  vi.unstubAllGlobals()
  document.documentElement.removeAttribute('data-theme')
})

describe('ThemeStore', () => {
  it('toggle switches theme and persists', () => {
    const store = new ThemeStore()
    const start = store.theme
    store.toggle()
    expect(store.theme).toBe(start === 'dark' ? 'light' : 'dark')
    expect(localStorage.getItem('bridge-ui-theme')).toBe(store.theme)
    expect(document.documentElement.getAttribute('data-theme')).toBe(store.theme)
  })

  it('setTheme updates mode', () => {
    const store = new ThemeStore()
    store.setTheme('light')
    expect(store.theme).toBe('light')
  })
})
