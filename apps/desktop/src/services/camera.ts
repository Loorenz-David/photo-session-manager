import { commands, type CameraStatusResponse } from './bindings';
import { unwrap } from './sessions';

export type CameraStatus = CameraStatusResponse;

export async function cameraStatus(): Promise<CameraStatus> {
  return unwrap(await commands.cameraStatus());
}

export async function cameraRetry(): Promise<CameraStatus> {
  return unwrap(await commands.cameraRetry());
}
