import { commands } from './bindings';

export async function getHealth() {
  const result = await commands.healthGet();
  if (result.status === 'error') throw new Error(result.error);
  return result.data;
}

export async function getAppPaths() {
  const result = await commands.appPathsGet();
  if (result.status === 'error') throw new Error(result.error);
  return result.data;
}

