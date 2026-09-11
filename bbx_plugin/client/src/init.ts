/**
 * Side-effect entry that wires the `window.__BBX__` API a JUCE WebView backend
 * pushes visualization data through. Import once, before mounting the UI:
 *
 * ```ts
 * import '@bbx-audio/plugin/init'
 * ```
 */

import '@bbx-audio/nectar/init'

import { sampleData, spectrumData, vuData } from '@bbx-audio/honey'
import type { IBbxGlobal, ISampleData, ISpectrumData, IVuData } from '@bbx-audio/nectar'

declare global {
    interface Window {
        __BBX__: IBbxGlobal
    }
}

window.__BBX__ = {
    updateSampleData(data: ISampleData): void {
        sampleData.set(data)
    },
    updateSpectrumData(data: ISpectrumData): void {
        spectrumData.set(data)
    },
    updateVuData(data: IVuData): void {
        vuData.set(data)
    },
}

// Gate on process.env.NODE_ENV rather than import.meta.env.PROD: Vite lib mode
// statically replaces import.meta.env.* when *this package* is built, whereas
// process.env.NODE_ENV is left for the consuming plugin's bundler to substitute.
if (process.env.NODE_ENV === 'production') {
    document.body.oncontextmenu = () => false
}
