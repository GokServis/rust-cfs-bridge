import { observer } from 'mobx-react-lite'

import type { TelemetryStore } from '../../stores/telemetryStore'
import { apidOf, summaryLine } from '../../telemetryFiltering'
import { Button } from '../ui/Button'
import { Panel } from '../ui/Panel'

import './TelemetryLogTable.css'

export const TelemetryLogTable = observer(function TelemetryLogTable({
  store,
}: {
  store: TelemetryStore
}) {
  const {
    kindFilter,
    apidFilter,
    searchText,
    pageSize,
    effectivePageIndex,
    totalPages,
    filteredCount,
    entries,
    pagedEntries,
  } = store

  return (
    <Panel title="Telemetry log" className="telemetry-log">
      <div className="telemetry-log__filters" role="search">
        <label className="telemetry-log__field">
          <span className="telemetry-log__label">Kind</span>
          <select
            value={kindFilter}
            onChange={(e) =>
              store.setKindFilter(
                e.target.value as 'all' | 'es_hk_v1' | 'to_lab_hk_v1' | 'evs_long_event_v1' | 'parse_error',
              )
            }
            aria-label="Filter by packet kind"
          >
            <option value="all">All</option>
            <option value="es_hk_v1">es_hk_v1</option>
            <option value="to_lab_hk_v1">to_lab_hk_v1</option>
            <option value="evs_long_event_v1">evs_long_event_v1</option>
            <option value="parse_error">parse_error</option>
          </select>
        </label>
        <label className="telemetry-log__field">
          <span className="telemetry-log__label">APID</span>
          <input
            type="text"
            inputMode="numeric"
            placeholder="Any"
            value={apidFilter}
            onChange={(e) => store.setApidFilter(e.target.value)}
            aria-label="Filter by CCSDS APID"
          />
        </label>
        <label className="telemetry-log__field telemetry-log__field--grow">
          <span className="telemetry-log__label">Search</span>
          <input
            type="search"
            placeholder="Substring in JSON"
            value={searchText}
            onChange={(e) => store.setSearchText(e.target.value)}
            aria-label="Search in telemetry JSON"
          />
        </label>
        <label className="telemetry-log__field">
          <span className="telemetry-log__label">Parse errors</span>
          <select
            value={store.hideParseError ? 'hide' : 'show'}
            onChange={(e) => store.setHideParseError(e.target.value === 'hide')}
            aria-label="Parse error visibility"
          >
            <option value="show">Show</option>
            <option value="hide">Hide</option>
          </select>
        </label>
        <label className="telemetry-log__field">
          <span className="telemetry-log__label">Rows / page</span>
          <select
            value={pageSize}
            onChange={(e) => store.setPageSize(Number(e.target.value))}
            aria-label="Rows per page"
          >
            {[10, 25, 50, 100].map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
        </label>
        <div className="telemetry-log__actions">
          <Button type="button" variant="ghost" onClick={() => store.clearBuffer()}>
            Clear buffer
          </Button>
        </div>
      </div>

      <p className="telemetry-log__meta" aria-live="polite">
        Buffered {entries.length} · matching {filteredCount} · page {effectivePageIndex + 1} of{' '}
        {totalPages}
      </p>

      <div className="telemetry-log__table-wrap">
        <table className="telemetry-log__table">
          <thead>
            <tr>
              <th scope="col">#</th>
              <th scope="col">Server time</th>
              <th scope="col">Kind</th>
              <th scope="col">APID</th>
              <th scope="col">Len</th>
              <th scope="col">Summary</th>
            </tr>
          </thead>
          <tbody>
            {pagedEntries.length === 0 ? (
              <tr>
                <td colSpan={6} className="telemetry-log__empty">
                  No rows match the current filters.
                </td>
              </tr>
            ) : (
              pagedEntries.map((row) => {
                const m = row.message
                const apid = apidOf(m)
                return (
                  <tr key={row.seq}>
                    <td>{row.seq}</td>
                    <td className="telemetry-log__mono">{m.received_at}</td>
                    <td>
                      <code>{m.kind}</code>
                    </td>
                    <td>{apid === null ? '—' : apid}</td>
                    <td>{m.raw_len}</td>
                    <td className="telemetry-log__summary">{summaryLine(m)}</td>
                  </tr>
                )
              })
            )}
          </tbody>
        </table>
      </div>

      <div className="telemetry-log__pager">
        <Button type="button" variant="ghost" disabled={effectivePageIndex <= 0} onClick={() => store.prevPage()}>
          Previous
        </Button>
        <Button
          type="button"
          variant="ghost"
          disabled={effectivePageIndex >= totalPages - 1}
          onClick={() => store.nextPage()}
        >
          Next
        </Button>
      </div>
    </Panel>
  )
})
