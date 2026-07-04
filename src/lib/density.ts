// Spacing presets driven by the "Layout density" setting. Returns a CSS
// custom-property string to apply on a page's root element.
import type { Density } from './types';

const DENSITY_VARS: Record<string, Record<string, string>> = {
  compact: {
    '--sp-main-pad': '10px',
    '--sp-card-gap': '6px',
    '--sp-sidebar-pad': '8px',
    '--sp-topcat-pad': '6px 10px',
    '--sp-card-pad': '8px 10px',
    '--sp-card-gap-inner': '4px',
    '--sp-panel-pad': '14px',
    '--sp-panel-gap': '12px',
    '--sp-field-gap': '4px',
  },
  normal: {
    '--sp-main-pad': '24px',
    '--sp-card-gap': '12px',
    '--sp-sidebar-pad': '16px',
    '--sp-topcat-pad': '10px 16px',
    '--sp-card-pad': '14px 16px',
    '--sp-card-gap-inner': '8px',
    '--sp-panel-pad': '24px',
    '--sp-panel-gap': '16px',
    '--sp-field-gap': '5px',
  },
  comfortable: {
    '--sp-main-pad': '32px',
    '--sp-card-gap': '18px',
    '--sp-sidebar-pad': '22px',
    '--sp-topcat-pad': '16px 22px',
    '--sp-card-pad': '20px 22px',
    '--sp-card-gap-inner': '12px',
    '--sp-panel-pad': '32px',
    '--sp-panel-gap': '22px',
    '--sp-field-gap': '8px',
  },
};

export function densityStyle(density: Density | string): string {
  return Object.entries(DENSITY_VARS[density] ?? DENSITY_VARS.normal)
    .map(([k, v]) => `${k}:${v}`).join(';');
}
