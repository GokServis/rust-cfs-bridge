import type { ButtonHTMLAttributes, ReactNode } from 'react'

import './Button.css'

type Variant = 'primary' | 'ghost'

export function Button({
  children,
  variant = 'primary',
  className = '',
  type = 'button',
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  children: ReactNode
  variant?: Variant
}) {
  const v = variant === 'ghost' ? 'ui-btn--ghost' : ''
  return (
    <button type={type} className={`ui-btn ${v} ${className}`.trim()} {...rest}>
      {children}
    </button>
  )
}
