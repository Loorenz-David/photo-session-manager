export const ftp = (host: string, port: number, user: string) => [
  'MENU › Network › [FTP Transfer] › [FTP Transfer Func.]; first enable [Wi-Fi Connect] and join the same network as this Mac.',
  'MENU › Network › [FTP Transfer] › [FTP Transfer Func.] › [Server Setting] › [Server 1].',
  `[Destination Settings] › [Host Name] = ${host}; [Port] = ${port}; [Secure Protocol] = Off.`,
  '[Directory Settings] › [Directory Hierarchy] = [Standard]; [Same File Name] = [Does Not Overwrite].',
  `[User Info Settings] › [User] = ${user}; [Password] = the password shown above.`,
  '[FTP Transfer Func.] › [Server Setting] › select Server 1 as the active server.',
  '[FTP Transfer Func.] › [FTP Function] = [On].',
  'MENU › Network › [FTP Transfer] › [FTP Transfer Func.] › [Auto FTP Trnsfr] = [On]; [Auto FTP Target] Still Image = [All].',
  '[FTP Transfer Func.] › [RAW+J/H Transfer Target] = [RAW] when shooting RAW+JPEG.',
  '[FTP Transfer Func.] › [FTP Power Save] = [Off].',
  'MENU › Shooting › [Image Quality] › [File Format] = [RAW].',
];
export const accessAuthentication = [
  'MENU › Network › [Wi-Fi] › [Wi-Fi Connect] › [On] (same access point as this Mac).',
  'MENU › Network › [Network Option] › [Access Authen. Settings] › [Access Authen.] = [On].',
  'Use the camera’s [User] and [Password] in the fields above.',
  'MENU › Network › [Cnct./Remote Sht.] › [Remote Shoot Function] › [Remote Shooting] = [On].',
  'Make sure no smartphone app is connected.',
  'Compare the fingerprint at MENU › Network › Network Option › Access Authen. Info.',
];
export const storeDestination = [
  'MENU › Network › [Cnct./Remote Sht.] › [Remote Shoot Setting] › [Still Img. Save Dest.] = [Camera Only] for Wi-Fi.',
  'On the cable, Session Manager sets the camera to save to the card and computer.',
];
