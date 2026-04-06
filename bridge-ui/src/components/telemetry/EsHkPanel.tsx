import type { TlmMessage } from '../../telemetryTypes'

function formatVersion(bytes: number[]): string {
  return bytes.map((b) => b.toString()).join('.')
}

export function EsHkPanel({ msg }: { msg: Extract<TlmMessage, { kind: 'es_hk_v1' }> }) {
  const h = msg.es_hk
  return (
    <dl className="telemetry-dl">
      <div className="telemetry-dl__row">
        <dt>CCSDS APID</dt>
        <dd>
          <code>0x{msg.primary.apid.toString(16).padStart(3, '0')}</code>
        </dd>
      </div>
      <div className="telemetry-dl__row">
        <dt>Command / error ctr</dt>
        <dd>
          {h.command_counter} / {h.command_error_counter}
        </dd>
      </div>
      <div className="telemetry-dl__row">
        <dt>cFE version</dt>
        <dd>{formatVersion(h.cfe_version)}</dd>
      </div>
      <div className="telemetry-dl__row">
        <dt>OSAL version</dt>
        <dd>{formatVersion(h.osal_version)}</dd>
      </div>
      <div className="telemetry-dl__row">
        <dt>PSP version</dt>
        <dd>{formatVersion(h.psp_version)}</dd>
      </div>
      <div className="telemetry-dl__row">
        <dt>Registered apps (core / ext)</dt>
        <dd>
          {h.registered_core_apps} / {h.registered_external_apps}
        </dd>
      </div>
      <div className="telemetry-dl__row">
        <dt>Tasks / libs</dt>
        <dd>
          {h.registered_tasks} / {h.registered_libs}
        </dd>
      </div>
      <div className="telemetry-dl__row">
        <dt>Heap bytes free / blocks / max blk</dt>
        <dd>
          {h.heap_bytes_free} / {h.heap_blocks_free} / {h.heap_max_block_size}
        </dd>
      </div>
      <div className="telemetry-dl__row">
        <dt>Processor resets / max</dt>
        <dd>
          {h.processor_resets} / {h.max_processor_resets}
        </dd>
      </div>
      <div className="telemetry-dl__row">
        <dt>Perf state / mode / triggers</dt>
        <dd>
          {h.perf_state} / {h.perf_mode} / {h.perf_trigger_count}
        </dd>
      </div>
      <div className="telemetry-dl__row">
        <dt>Raw length</dt>
        <dd>{msg.raw_len} bytes</dd>
      </div>
    </dl>
  )
}
