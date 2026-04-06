import { useContext } from 'react'

import { StoreContext } from './context'
import type { RootStore } from './rootStore'

export function useStore(): RootStore {
  const s = useContext(StoreContext)
  if (!s) {
    throw new Error('useStore must be used within StoreProvider')
  }
  return s
}
