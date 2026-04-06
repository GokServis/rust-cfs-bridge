import type { ReactNode } from 'react'

import './Panel.css'

export function Panel({
  title,
  children,
  className = '',
}: {
  title?: string
  children: ReactNode
  className?: string
}) {
  return (
    <section className={`ui-panel ${className}`.trim()}>
      {title ? <h3 className="ui-panel__title">{title}</h3> : null}
      {children}
    </section>
  )
}
