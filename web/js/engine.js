// The page's side of the worker protocol (see ../worker.js).

/** An error of the engine: a stable `code` the page translates, and an English message. */
export class EngineError extends Error {
  constructor(code, message) {
    super(message);
    this.code = code;
  }
}

/** Longest wait for the engine to start, in milliseconds (the package is ~1.5 MB gzip'd). */
const START_TIMEOUT_MS = 60_000;

export class Engine {
  /** @param {URL} workerUrl */
  constructor(workerUrl) {
    this.workerUrl = workerUrl;
    this.pending = new Map();
    this.slots = new Map();
    this.nextId = 1;
    this.worker = null;
    this.version = null;
  }

  /** Starts the worker; resolves with the package version. */
  start() {
    return new Promise((resolve, reject) => {
      let worker;
      try {
        worker = new Worker(this.workerUrl, { type: 'module' });
      } catch (error) {
        reject(new EngineError('no_worker', String(error && error.message || error)));
        return;
      }
      const timer = setTimeout(() => {
        worker.terminate();
        reject(new EngineError('start_timeout', 'the engine did not start in time'));
      }, START_TIMEOUT_MS);
      worker.onerror = (event) => {
        clearTimeout(timer);
        event.preventDefault();
        reject(new EngineError('no_worker', event.message || 'the worker could not be loaded'));
      };
      worker.onmessage = (event) => {
        const data = event.data || {};
        if (data.type === 'ready') {
          clearTimeout(timer);
          this.worker = worker;
          this.version = data.version;
          worker.onmessage = (e) => this.receive(e.data || {});
          worker.onerror = (e) => {
            e.preventDefault();
            this.restart(new EngineError('worker_error', e.message || 'worker error'));
          };
          resolve(data.version);
        } else if (data.type === 'init_error') {
          clearTimeout(timer);
          worker.terminate();
          reject(new EngineError('init_error', data.message));
        }
      };
    });
  }

  receive(data) {
    const entry = this.pending.get(data.id);
    if (!entry) return;
    this.pending.delete(data.id);
    if (data.ok) {
      entry.resolve(data.value);
    } else {
      const error = data.error || {};
      entry.reject(new EngineError(error.code || 'internal', error.message || ''));
      if (data.fatal) this.restart(new EngineError('panic', error.message || 'engine failure'));
    }
  }

  /** Replaces a worker whose instance can no longer be trusted, keeping the slots. */
  restart(reason) {
    if (this.worker) this.worker.terminate();
    this.worker = null;
    for (const entry of this.pending.values()) entry.reject(reason);
    this.pending.clear();
    this.restarting = this.start()
      .then(() => Promise.all([...this.slots].map(([slot, bytes]) => this.post('put', [slot, bytes]))))
      .catch(() => {});
  }

  post(fn, args, transfer = []) {
    return new Promise((resolve, reject) => {
      if (!this.worker) {
        reject(new EngineError('engine_unavailable', 'the engine is not running'));
        return;
      }
      const id = this.nextId++;
      this.pending.set(id, { resolve, reject });
      this.worker.postMessage({ id, fn, args }, transfer);
    });
  }

  /** Calls an exported function. Byte slots are passed as `{slot: name}`. */
  async call(fn, ...args) {
    if (!this.worker && this.restarting) await this.restarting;
    return this.post(fn, args);
  }

  /** Calls a function whose result is JSON. */
  async json(fn, ...args) {
    return JSON.parse(await this.call(fn, ...args));
  }

  /** Stores bytes in the worker under `slot` (`null` removes them). */
  async put(slot, bytes) {
    if (bytes) {
      this.slots.set(slot, bytes);
      return this.post('put', [slot, bytes]);
    }
    this.slots.delete(slot);
    return this.post('drop', [slot]);
  }
}
