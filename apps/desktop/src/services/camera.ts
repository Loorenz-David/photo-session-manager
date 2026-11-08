import { commands, type CameraStatusResponse, type CameraSupportDto, type TestConnectionResponse } from './bindings';
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

export const cameraReconnect = async () => { unwrap(await commands.cameraReconnect()); };
export const cameraFilesNotReceivedDismiss = async () => { unwrap(await commands.cameraFilesNotReceivedDismiss()); };
export const cameraTakeBack = async () => unwrap(await commands.cameraTakeBack());
export const cameraTakeBackAvailability = async (): Promise<string | null> => unwrap(await commands.cameraTakeBackAvailability()).reason;
export const cameraTestConnection = async (): Promise<TestConnectionResponse> => unwrap(await commands.cameraTestConnection());
export const cameraSupportGet = async (): Promise<CameraSupportDto | null> => unwrap(await commands.cameraSupportGet());
export const cameraSupportInstall = async (path: string): Promise<CameraSupportDto> => unwrap(await commands.cameraSupportInstall(path));
export const cameraSecretsSet = async (value: {access_auth_user: string; access_auth_password: string}) => {
  unwrap(await commands.cameraSecretsSet(value.access_auth_user, value.access_auth_password));
};
export const cameraFtpPasswordGenerate = async () => unwrap(await commands.cameraFtpPasswordGenerate());
export const cameraFingerprintConfirm = async (fingerprint: string) => { unwrap(await commands.cameraFingerprintConfirm(fingerprint)); };
export const cameraFingerprintReject = async () => { unwrap(await commands.cameraFingerprintReject()); };
export const cameraHelperLog = async (): Promise<string[]> => unwrap(await commands.cameraHelperLog());
