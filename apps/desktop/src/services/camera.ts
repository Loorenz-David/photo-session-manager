import { commands, type CameraStatusResponse, type CameraSupportDto, type FingerprintDto, type FtpAddressDto_Serialize as FtpAddressDto, type TestConnectionResponse } from './bindings';
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
/** "Show password" (P68): the FTP password from the Keychain, `null` when none was generated. */
export const cameraFtpPasswordGet = async (): Promise<string | null> => unwrap(await commands.cameraFtpPasswordGet())?.password ?? null;
/** This Mac's address for the camera's Host Name and, when it changed, the sentence to show (P68). */
export const cameraFtpAddress = async (): Promise<FtpAddressDto> => unwrap(await commands.cameraFtpAddress());
/** The fingerprint waiting for confirmation, if any (P68). */
export const cameraFingerprintPending = async (): Promise<FingerprintDto | null> => unwrap(await commands.cameraFingerprintPending());
export const cameraFingerprintConfirm = async (fingerprint: string) => { unwrap(await commands.cameraFingerprintConfirm(fingerprint)); };
export const cameraFingerprintReject = async () => { unwrap(await commands.cameraFingerprintReject()); };
export const cameraHelperLog = async (): Promise<string[]> => unwrap(await commands.cameraHelperLog());
