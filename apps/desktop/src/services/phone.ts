import { commands, type PairingCreatedDto, type PairingDto_Serialize, type PhoneServerStatusDto_Serialize } from './bindings';
import { unwrap } from './sessions';

/** A pairing just made: its token is only in the two URLs and QR codes (shown once). */
export type PairingCreated = PairingCreatedDto;
/** One paired phone (never its token). */
export type Pairing = PairingDto_Serialize;
/** The phone server: switched on, listening, its port, why it could not start. */
export type PhoneServerStatus = PhoneServerStatusDto_Serialize;

/** Pairs a new phone (named, or "Phone <n>"); resolves to the URLs and QR codes that open the phone app. */
export async function phonePairingCreate(name: string | null): Promise<PairingCreated> {
  return unwrap(await commands.phonePairingCreate(name));
}

export async function phonePairingsList(): Promise<Pairing[]> {
  return unwrap(await commands.phonePairingsList());
}

export async function phonePairingRename(id: number, name: string): Promise<void> {
  unwrap(await commands.phonePairingRename(id, name));
}

/** Removes a pairing: that phone's token stops working. */
export async function phonePairingRevoke(id: number): Promise<void> {
  unwrap(await commands.phonePairingRevoke(id));
}

export async function phoneServerStatus(): Promise<PhoneServerStatus> {
  return unwrap(await commands.phoneServerStatus());
}
