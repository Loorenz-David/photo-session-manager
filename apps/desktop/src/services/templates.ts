import { open } from '@tauri-apps/plugin-dialog';
import { commands, type TemplateDto_Serialize, type TemplateSummaryDto, type TemplateViewDto_Serialize } from './bindings';
import { unwrap } from './sessions';

export type Template = TemplateDto_Serialize;
export type TemplateSummary = TemplateSummaryDto;
export type Presets = { split: boolean; shape: string | null; zoom: number };
export type TemplateView = TemplateViewDto_Serialize;
export async function getTemplateView(sessionId: number): Promise<TemplateView> { return unwrap(await commands.templateSessionView(sessionId)); }

export async function listTemplates(): Promise<TemplateSummary[]> { return unwrap(await commands.templatesList()); }
export async function getTemplate(itemType: string): Promise<Template | null> { return unwrap(await commands.templateGet(itemType)); }
export async function createTemplate(itemType: string): Promise<Template> { return unwrap(await commands.templateCreate(itemType)); }
export async function deleteTemplate(id: number): Promise<void> { unwrap(await commands.templateDelete(id)); }
export async function addContainer(id: number, source: string | null): Promise<Template> { return unwrap(await commands.templateContainerAdd(id, source)); }
export async function removeContainer(id: number): Promise<Template> { return unwrap(await commands.templateContainerRemove(id)); }
export async function setExample(id: number, source: string): Promise<Template> { return unwrap(await commands.templateContainerExampleSet(id, source)); }
export async function reorderContainers(id: number, ids: number[]): Promise<Template> { return unwrap(await commands.templateContainersReorder(id, ids)); }
export async function setShootingOrder(id: number, ids: number[]): Promise<Template> { return unwrap(await commands.templateShootingOrderSet(id, ids)); }
export async function setPresets(id: number, presets: Presets): Promise<Template> { return unwrap(await commands.templateContainerPresetsSet(id, presets)); }
export async function pickExampleFiles(): Promise<string[]> {
  const chosen = await open({ multiple: true, directory: false, title: 'Add example photos', filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'tif', 'tiff', 'webp'] }] });
  if (!chosen) return [];
  return Array.isArray(chosen) ? chosen : [chosen];
}
