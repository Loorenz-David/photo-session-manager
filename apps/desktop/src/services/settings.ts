import { open } from '@tauri-apps/plugin-dialog';
import {
  commands,
  type IntegrationSettingsDto,
  type SettingsDto_Serialize,
  type SettingsPatchDto_Deserialize,
} from './bindings';
import { unwrap } from './sessions';

export type Settings = SettingsDto_Serialize;
export type SettingsPatch = SettingsPatchDto_Deserialize;
export type IntegrationSettings = IntegrationSettingsDto;

/** `settings.json` as shown: the client secret masked, the preview delay as stored plus its effective (clamped) value. */
export async function getSettings(): Promise<Settings> {
  return unwrap(await commands.settingsGet());
}

/** Saves a partial update; sending the masked secret back keeps the stored one. Resolves to the settings as shown after the save. */
export async function saveSettings(patch: SettingsPatch): Promise<Settings> {
  return unwrap(await commands.settingsSet(patch));
}

/** Opens the macOS folder picker; resolves to the chosen folder or null when cancelled. */
export async function pickFolder(title: string): Promise<string | null> {
  const chosen = await open({ multiple: false, directory: true, title });
  return typeof chosen === 'string' ? chosen : null;
}
