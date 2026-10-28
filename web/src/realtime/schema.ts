// This file was auto-generated from api/realtime.schema.json. Do not edit.

export type ClientMessage = Ping

export type ServerMessage = Welcome | Pong | Error

export interface Ping {
  "type": "ping";
  "sequence": number;
}

export interface Welcome {
  "type": "welcome";
  "protocolVersion": 2;
  "maxPlayers": 4;
}

export interface Pong {
  "type": "pong";
  "sequence": number;
}

export interface Error {
  "type": "error";
  "code": "invalidMessage";
  "message": "Invalid client message";
}
