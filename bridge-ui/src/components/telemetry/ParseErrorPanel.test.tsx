import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { ParseErrorPanel } from './ParseErrorPanel'

describe('ParseErrorPanel', () => {
  it('shows message and hex', () => {
    render(
      <ParseErrorPanel
        msg={{
          kind: 'parse_error',
          received_at: '2026-01-01T00:00:00Z',
          raw_len: 3,
          primary: null,
          message: 'bad packet',
          hex_preview: 'de ad',
        }}
      />,
    )
    expect(screen.getByText('bad packet')).toBeInTheDocument()
    expect(screen.getByText('de ad')).toBeInTheDocument()
  })
})
