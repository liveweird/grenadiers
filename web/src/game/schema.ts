// This file was auto-generated from api/gameplay.schema.json. Do not edit.

export type ProtocolVersionInput = number

export type GameplayVersion = 1

export type RulesVersion = 1

export type MapId = "fixed_v1"

export type SessionToken = string

export type RoomId = string

export type RoomCode = string

export type Counter = number

export type EntityId = number

export type Tick = number

export type Seat = "one" | "two"

export type Direction = "north" | "east" | "south" | "west"

export interface Cell {
  "column": number;
  "row": number;
}

export interface Position {
  "x": number;
  "y": number;
}

export type ErrorCode = "invalidMessage" | "authenticationFailed" | "connectionInUse" | "notAuthenticated" | "alreadyAuthenticated" | "alreadyInRoom" | "notInRoom" | "roomNotFound" | "roomFull" | "staleRoom" | "invalidPhase" | "staleSequence" | "duplicateAction" | "rateLimited" | "capacityReached"

export type InterruptReason = "participantLeft" | "connectionLost" | "serverOverloaded" | "serverStopped" | "internalError"

export type RoundResult = WinnerResult | DrawResult

export interface WinnerResult {
  "type": "winner";
  "seat": Seat;
}

export interface DrawResult {
  "type": "draw";
  "reason": "mutualElimination" | "timeout";
}

export interface SeatStatus {
  "seat": Seat;
  "occupied": boolean;
  "ready": boolean;
}

export type RoomPhase = WaitingPhase | CountdownPhase | RunningPhase | ResultPhase | InterruptedPhase

export interface WaitingPhase {
  "type": "waiting";
}

export interface CountdownPhase {
  "type": "countdown";
  "endsAt": Tick;
}

export interface RunningPhase {
  "type": "running";
}

export interface ResultPhase {
  "type": "result";
  "endedAt": Tick;
  "result": RoundResult;
}

export interface InterruptedPhase {
  "type": "interrupted";
  "endedAt": Tick;
  "reason": InterruptReason;
}

export interface RoomView {
  "roomId": RoomId;
  "roomCode": RoomCode;
  "roundId": Counter;
  "revision": Counter;
  "tick": Tick;
  "rulesVersion": RulesVersion;
  "mapId": MapId;
  "seats": SeatStatus[];
  "phase": RoomPhase;
}

export type GameClientMessage = Authenticate | CreateRoom | JoinRoom | Ready | Movement | PlaceBomb | LeaveRoom

export interface Authenticate {
  "type": "authenticate";
  "protocolVersion": ProtocolVersionInput;
  "token": SessionToken;
}

export interface CreateRoom {
  "type": "createRoom";
  "sequence": Counter;
}

export interface JoinRoom {
  "type": "joinRoom";
  "sequence": Counter;
  "roomCode": RoomCode;
}

export interface Ready {
  "type": "ready";
  "sequence": Counter;
  "roomId": RoomId;
  "roundId": Counter;
  "ready": boolean;
}

export interface Movement {
  "type": "movement";
  "sequence": Counter;
  "roomId": RoomId;
  "roundId": Counter;
  "direction": Direction | null;
}

export interface PlaceBomb {
  "type": "placeBomb";
  "sequence": Counter;
  "roomId": RoomId;
  "roundId": Counter;
  "actionId": Counter;
}

export interface LeaveRoom {
  "type": "leaveRoom";
  "sequence": Counter;
  "roomId": RoomId;
  "roundId": Counter;
}

export type GameServerMessage = Authenticated | IncompatibleProtocol | RoomState | LeftRoom | Error

export interface Authenticated {
  "type": "authenticated";
  "protocolVersion": GameplayVersion;
}

export interface IncompatibleProtocol {
  "type": "incompatibleProtocol";
  "supportedVersion": GameplayVersion;
}

export interface RoomState {
  "type": "roomState";
  "room": RoomView;
  "youSeat": Seat;
}

export interface LeftRoom {
  "type": "leftRoom";
  "roomId": RoomId;
  "sequence": Counter;
}

export interface Error {
  "type": "error";
  "code": ErrorCode;
  "sequence": Counter | null;
}
