import type { ReactNode } from 'react'

import { AnimatedBackdrop } from './AnimatedBackdrop'
import { TopNav } from './TopNav'

import './AppShell.css'

export function AppShell({ children }: { children: ReactNode }) {
  return (
    <>
      <AnimatedBackdrop />
      <div className="app-shell">
        <TopNav />
        <main className="app-shell__main">{children}</main>
      </div>
    </>
  )
}
