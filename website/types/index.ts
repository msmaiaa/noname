export type User = {
  personaname: String;
  avatar: String;
  is_admin: boolean;
};

export type ServerStatus =
  | "Idle"
  | "WaitingForPlayers"
  | "Starting"
  | "KnifeRound"
  | "Live"
  | "Ending";

export type ServerWithStatus = {
  id: number;
  ip: string;
  port: string;
  status: ServerStatus;
  online: boolean;
};
