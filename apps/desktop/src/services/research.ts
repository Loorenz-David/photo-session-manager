import { commands, type ExportSelectionDto, type ResearchStatusDto_Serialize, type SecretsInputDto, type SecretsStateDto, type StepViewDto_Serialize } from './bindings';
import { unwrap } from './sessions';

export type ResearchStatus = ResearchStatusDto_Serialize;
export type ResearchStep = StepViewDto_Serialize;
export type ResearchSecretsState = SecretsStateDto;

/** The code `research_*` commands answer when the app could not build the research engine. */
export const RESEARCH_UNAVAILABLE = 'research_unavailable';

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
/** Restores operational research defaults while retaining the user's personal categories. */
export async function resetResearchSettings(): Promise<string> {
  return unwrap(await commands.researchSettingsReset());
}
/** The trusted sites of a `research` value the site commands return (the new value, saved by the command). */
function sitesOf(text: string): string[] {
  const value = JSON.parse(text) as { trusted_sites?: unknown };
  return Array.isArray(value.trusted_sites) ? value.trusted_sites.filter((s): s is string => typeof s === 'string') : [];
}
/** Adds a trusted site (MC-A3 Trusted entry); resolves to the new list, or rejects with the refusal's text. */
export async function addResearchSite(site: string): Promise<string[]> {
  return sitesOf(unwrap(await commands.researchSiteAdd(site)));
}
export async function removeResearchSite(site: string): Promise<string[]> {
  return sitesOf(unwrap(await commands.researchSiteRemove(site)));
}
export async function reorderResearchSites(sites: string[]): Promise<string[]> {
  return sitesOf(unwrap(await commands.researchSitesReorder(sites)));
}
/** Export research (MC-A11): writes the selected runs to `dest` as one zipped folder; resolves to the zip's path. */
export async function exportResearch(selection: ExportSelectionDto, dest: string): Promise<string> {
  return unwrap(await commands.researchExport(selection, dest));
}
