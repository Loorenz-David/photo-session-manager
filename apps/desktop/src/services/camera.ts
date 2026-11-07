import { commands, type CameraStatusResponse } from './bindings';
import { unwrap } from './sessions';

export type CameraStatus = CameraStatusResponse;

export async function cameraStatus(): Promise<CameraStatus> {
  return unwrap(await commands.cameraStatus());
}

export async function cameraRetry(): Promise<CameraStatus> {
  return unwrap(await commands.cameraRetry());
}

/** Fires the shutter; answers the capture id (the exposure row carries it; callers may ignore it). */
export async function cameraCapture(): Promise<string> {
  return unwrap(await commands.cameraCapture()).capture_id;
}
