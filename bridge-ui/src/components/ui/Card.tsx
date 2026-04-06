import type { ReactNode } from 'react'

import './Card.css'

export function Card({ children, className = '' }: { children: ReactNode; className?: string }) {
  return (
    <div className={`ui-card ${className}`.trim()}>
      <div className="ui-card__inner">{children}</div>
    </div>
  )
}
