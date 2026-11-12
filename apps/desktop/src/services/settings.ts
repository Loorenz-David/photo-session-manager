import { open } from '@tauri-apps/plugin-dialog';
import {
  commands,
  type IntegrationSettingsDto,
  type ResearchSettingsDto_Serialize,
  type SettingsDto_Serialize,
  type SettingsPatchDto_Deserialize,
} from './bindings';
import { unwrap } from './sessions';

export type Settings = SettingsDto_Serialize;
export type SettingsPatch = SettingsPatchDto_Deserialize;
export type IntegrationSettings = IntegrationSettingsDto;
/** Settings → Research: the `research` block, every field present on read (master_plan_research §6.6). */
export type ResearchSettings = ResearchSettingsDto_Serialize;

/** `settings.json` as shown: the client secret masked, the preview delay as stored plus its effective (clamped) value. */
export async function getSettings(): Promise<Settings> {
  return unwrap(await commands.settingsGet());
}

/** Saves a partial update; sending the masked secret back keeps the stored one. Resolves to the settings as shown after the save. */
export async function saveSettings(patch: SettingsPatch): Promise<Settings> {
  return unwrap(await commands.settingsSet(patch));
}

/**
 * Saves the whole `research` block. The research engine validates it first (MC-A8 amounts, MC-A11 `record_days` ≥ 2,
 * whole seconds, positive long edges): a refusal rejects with its text. Resolves to the block as saved.
 */
export async function saveResearchSettings(research: ResearchSettings): Promise<ResearchSettings | null> {
  const saved = await saveSettings({ research });
  return saved.research ?? null;
}

/** Opens the macOS folder picker; resolves to the chosen folder or null when cancelled. */
export async function pickFolder(title: string): Promise<string | null> {
  const chosen = await open({ multiple: false, directory: true, title });
  return typeof chosen === 'string' ? chosen : null;
}

export async function pickSessionFile(title: string): Promise<string | null> {
  const chosen = await open({ multiple: false, directory: false,
    filters: [{ name: 'Capture One session', extensions: ['cosessiondb'] }], title });
  return typeof chosen === 'string' ? chosen : null;
}

export async function pickArchive(): Promise<string | null> {
  const chosen = await open({ multiple: false, directory: false,
    filters: [{ name: 'ZIP archive', extensions: ['zip'] }], title: 'Add camera support → Sony' });
  return typeof chosen === 'string' ? chosen : null;
}
