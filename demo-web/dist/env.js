// Shim module for Wasm imports under the "env" module.
// Provide a high-resolution clock for crates that import `env.now`.
export function now() {
	// Prefer performance.now() (f64 milliseconds, monotonic-ish), fallback to Date.now().
	try {
			return globalThis.performance && typeof globalThis.performance.now === 'function'
				? globalThis.performance.now()
				: Date.now();
	} catch (_) {
			return Date.now();
	}
}

// You can add more shims here if other `env.*` symbols are requested.
