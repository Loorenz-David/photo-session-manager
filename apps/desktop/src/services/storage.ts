import { commands, type StorageDto_Serialize } from './bindings';
import { unwrap } from './sessions';

/** Settings → Storage (10F §4): free space, space used per kind of file, the minimum free space, the warning. */
export type Storage = StorageDto_Serialize;

export async function getStorage(): Promise<Storage> {
  return unwrap(await commands.storageGet());
}

/** Saves the minimum free space in GB; the background clean-up applies it within seconds. */
export async function setMinFreeGb(gb: number): Promise<Storage> {
  return unwrap(await commands.storageMinFreeSet(gb));
}
