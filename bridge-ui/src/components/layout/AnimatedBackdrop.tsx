import './AnimatedBackdrop.css'

export function AnimatedBackdrop() {
  return (
    <div className="animated-backdrop" aria-hidden>
      <div className="animated-backdrop__grid" />
      <div className="animated-backdrop__glow" />
      <div className="animated-backdrop__scan" />
    </div>
  )
}
