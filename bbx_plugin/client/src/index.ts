/**
 * Node-safe root entry: the parameter codegen API only. The browser-facing
 * entries (`@bbx-audio/plugin/init`, `@bbx-audio/plugin/presets`) and the Vite
 * plugin (`@bbx-audio/plugin/vite`) are deliberately not re-exported here so
 * importing the root never pulls in the optional nectar/honey/vite peers.
 */

export * from './params'
