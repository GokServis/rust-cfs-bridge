import { observer } from 'mobx-react-lite'
import { NavLink } from 'react-router-dom'

import { useStore } from '../../stores/useStore'

import './TopNav.css'

export const TopNav = observer(function TopNav() {
  const { theme } = useStore()
  const navClass = ({ isActive }: { isActive: boolean }) =>
    `top-nav__link ${isActive ? 'top-nav__link--active' : ''}`.trim()

  return (
    <header className="top-nav">
      <div className="top-nav__brand">
        <h1 className="top-nav__title">cFS bridge</h1>
        <p className="top-nav__subtitle">Ground data link</p>
      </div>
      <nav className="top-nav__links" aria-label="Primary">
        <NavLink to="/" end className={navClass}>
          Commands
        </NavLink>
        <NavLink to="/telemetry" className={navClass}>
          Telemetry
        </NavLink>
      </nav>
      <div className="top-nav__actions">
        <button
          type="button"
          className="top-nav__theme-btn"
          onClick={() => theme.toggle()}
          aria-label={theme.theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
        >
          {theme.theme === 'dark' ? 'Light' : 'Dark'}
        </button>
      </div>
    </header>
  )
})
