import { observer } from 'mobx-react-lite'
import type { ReactNode } from 'react'

import { useStore } from '../../stores/useStore'
import { AlertBanner } from './AlertBanner'
import { AnimatedBackdrop } from './AnimatedBackdrop'
import { TopNav } from './TopNav'

import './AppShell.css'

export const AppShell = observer(function AppShell({ children }: { children: ReactNode }) {
  const { alerts } = useStore()
  return (
    <>
      <AnimatedBackdrop />
      <div className="app-shell">
        <TopNav />
        <AlertBanner store={alerts} />
        <main className="app-shell__main">{children}</main>
      </div>
    </>
  )
})
