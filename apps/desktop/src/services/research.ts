import { commands, type ExportSelectionDto, type ResearchStatusDto, type SecretsInputDto, type SecretsStateDto } from './bindings';
import { unwrap } from './sessions';

export type ResearchStatus = ResearchStatusDto;
export type ResearchSecretsState = SecretsStateDto;

export async function researchStatus(sessionId: number): Promise<ResearchStatus> {
  return unwrap(await commands.researchStatus(sessionId));
}
export async function runResearch(sessionId: number): Promise<void> {
  unwrap(await commands.researchRun(sessionId));
}
export async function setResearchSecrets(input: SecretsInputDto): Promise<void> {
  unwrap(await commands.researchSecretsSet(input));
}
export async function researchSecretsState(): Promise<ResearchSecretsState> {
  return unwrap(await commands.researchSecretsState());
}
export async function addResearchSite(site: string): Promise<unknown> {
  return JSON.parse(unwrap(await commands.researchSiteAdd(site)));
}
export async function removeResearchSite(site: string): Promise<unknown> {
  return JSON.parse(unwrap(await commands.researchSiteRemove(site)));
}
export async function reorderResearchSites(sites: string[]): Promise<unknown> {
  return JSON.parse(unwrap(await commands.researchSitesReorder(sites)));
}
export async function exportResearch(selection: ExportSelectionDto, dest: string): Promise<string> {
  return unwrap(await commands.researchExport(selection, dest));
}
