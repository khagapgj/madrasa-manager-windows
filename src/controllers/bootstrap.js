import { InvokeTransport, ServiceError } from '../services/core.js';
import { createServices } from '../services/facades.js';

function resolveInvoke() {
  const invoke = globalThis.__TAURI__?.core?.invoke;
  if (typeof invoke !== 'function') {
    return async () => { throw new ServiceError({ code: 'TAURI_UNAVAILABLE', message: 'Desktop database service পাওয়া যায়নি।' }); };
  }
  return invoke;
}

const services = createServices(new InvokeTransport(resolveInvoke()));
Object.defineProperty(globalThis, 'AppServices', { value: services, writable: false, configurable: false });

// Foundation connection point. Existing visual handlers remain untouched in Phase 2A;
// Phase 2B adapters will call these facades instead of legacy storage.
globalThis.MadrasaServiceBridge = Object.freeze({
  ready: services.app.initialize().then(result => {
    document.dispatchEvent(new CustomEvent('madrasa:services-ready', { detail: result }));
    return result;
  }).catch(error => {
    document.dispatchEvent(new CustomEvent('madrasa:services-error', { detail: error }));
    console.error('[Foundation] database initialization failed', error);
    return null;
  }),
  services
});
