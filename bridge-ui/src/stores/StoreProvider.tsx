import type { ReactNode } from 'react'

import type { RootStore } from './rootStore'
import { StoreContext } from './context'

export function StoreProvider({ store, children }: { store: RootStore; children: ReactNode }) {
  return <StoreContext.Provider value={store}>{children}</StoreContext.Provider>
}
