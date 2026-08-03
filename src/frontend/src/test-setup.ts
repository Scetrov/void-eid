if (typeof globalThis.window === 'undefined') {
  Object.assign(globalThis, { window: globalThis })
}
