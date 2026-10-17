// This file was auto-generated from api/gameplay-v6.schema.json. Do not edit.

export type ProtocolVersionInput = number

export type GameplayVersion = 6

export type RulesVersion = 1 | 2 | 3 | 4

export type MapId = "fixed_v1" | "generated_v1"

export type PickupKind = "bombCapacity" | "blastRange" | "movementSpeed"

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
  "rematchReady": boolean;
  "connection": ConnectionStatus;
}

export type RoomPhase = WaitingPhase | CountdownPhase | RunningPhase | ResultPhase | MatchResultPhase | InterruptedPhase

export interface WaitingPhase {
  "type": "waiting";
}

export interface CountdownPhase {
  "type": "countdown";
  "kind": "initial" | "interRound";
  "endsAt": Tick;
}

export interface RunningPhase {
  "type": "running";
}

export interface ResultPhase {
  "type": "result";
  "holdUntil": Tick;
}

export interface InterruptedPhase {
  "type": "interrupted";
  "endedAt": Tick;
  "reason": MatchInterruptReason;
}

export interface RoomView {
  "roomId": RoomId;
  "roomCode": RoomCode;
  "roundId": Counter;
  "revision": Counter;
  "tick": Tick;
  "rulesVersion": RulesVersion;
  "mapId": MapId;
  "roundNumber": RoundCount;
  "scores": Scores;
  "lastRound": CompletedRound | null;
  "seats": SeatStatus[];
  "phase": RoomPhase;
  "settings": GameSettings;
  "settingsRevision": Counter;
  "settingsEditor": Seat | null;
}

export type Terrain = "floor" | "wall"

export interface GameState {
  "runningUpdates": number;
  "remainingTicks": number;
  "arena": ArenaView;
  "actors": ActorView[];
  "bombs": BombView[];
  "flames": FlameView[];
  "pickups": PickupView[];
}

export interface PickupView {
  "cell": Cell;
  "kind": PickupKind;
}

export interface ArenaView {
  "width": 15;
  "height": 13;
  "terrain": Terrain[];
  "blocks": Cell[];
}

export interface ActorView {
  "id": EntityId;
  "seat": Seat;
  "position": Position;
  "heading": Direction;
  "speedLevel": number;
  "alive": boolean;
  "bombCapacity": number;
  "bombRange": number;
  "liveBombCount": number;
}

export interface BombView {
  "id": EntityId;
  "cell": Cell;
  "owner": EntityId;
  "range": number;
  "placedAt": Tick;
  "detonatesAt": Tick;
}

export interface FlameView {
  "cell": Cell;
  "expiresAt": Tick;
}

export interface WireEvent {
  "tick": Tick;
  "index": number;
  "kind": EventKind;
}

export type EventKind = BombPlacedEvent | BombDetonatedEvent | ActorEliminatedEvent | RoundEndedEvent | PickupCollectedEventKind

export interface BombPlacedEvent {
  "type": "bombPlaced";
  "bomb": BombView;
}

export interface BombDetonatedEvent {
  "type": "bombDetonated";
  "bomb": BombView;
}

export interface ActorEliminatedEvent {
  "type": "actorEliminated";
  "id": EntityId;
  "seat": Seat;
}

export interface RoundEndedEvent {
  "type": "roundEnded";
  "result": RoundResult;
}

export interface PickupCollectedEventKind {
  "type": "pickupCollected";
  "id": EntityId;
  "seat": Seat;
  "cell": Cell;
  "kind": PickupKind;
}

export type GameClientMessage = Authenticate | CreateRoom | JoinRoom | ResumeRoom | Ready | Movement | PlaceBomb | LeaveRoom | Rematch | UpdateSettingsCommand

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
  "settingsRevision": Counter;
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

export type GameServerMessage = Authenticated | IncompatibleProtocol | Admitted | RoomState | Snapshot | LeftRoom | SeatReplaced | Error

export interface Authenticated {
  "type": "authenticated";
  "protocolVersion": GameplayVersion;
}

export interface IncompatibleProtocol {
  "type": "incompatibleProtocol";
  "supportedVersion": ProtocolVersionInput;
}

export interface RoomState {
  "type": "roomState";
  "room": RoomView;
}

export interface Snapshot {
  "type": "snapshot";
  "room": RoomView;
  "state": GameState;
  "events": WireEvent[];
}

export interface LeftRoom {
  "type": "leftRoom";
  "roomId": RoomId;
  "sequence": Counter;
}

export interface Error {
  "type": "error";
  "code": V6ErrorCode;
  "sequence": Counter | null;
}

export type RoundCount = number

export type Wins = number

export interface Scores {
  "one": Wins;
  "two": Wins;
}

export type MatchInterruptReason = "connectionLost" | "serverOverloaded" | "serverStopped" | "internalError"

export type MatchResult = MatchWinnerResult | AbandonedMatchResult | InterruptedMatchResult

export interface MatchWinnerResult {
  "type": "winner";
  "seat": Seat;
  "reason": "roundWins" | "forfeit";
}

export interface AbandonedMatchResult {
  "type": "abandoned";
}

export interface InterruptedMatchResult {
  "type": "interrupted";
  "reason": MatchInterruptReason;
}

export interface CompletedRound {
  "roundId": Counter;
  "roundNumber": Counter;
  "endedAt": Tick;
  "result": RoundResult;
}

export interface MatchResultPhase {
  "type": "matchResult";
  "endedAt": Tick;
  "result": MatchResult;
  "rematchAt": Tick;
}

export interface Rematch {
  "type": "rematch";
  "sequence": Counter;
  "roomId": RoomId;
  "roundId": Counter;
  "ready": boolean;
}

export type ResumeToken = string

export type ConnectionStatus = VacantConnection | ConnectedConnection | DisconnectedConnection | ExpiredConnection

export interface VacantConnection {
  "type": "vacant";
}

export interface ConnectedConnection {
  "type": "connected";
}

export interface DisconnectedConnection {
  "type": "disconnected";
  "expiresAt": Tick;
}

export interface ExpiredConnection {
  "type": "expired";
}

export interface ResumeRoom {
  "type": "resumeRoom";
  "sequence": Counter;
  "roomId": RoomId;
  "resumeToken": ResumeToken;
}

export interface Admitted {
  "type": "admitted";
  "sequence": Counter;
  "room": RoomView;
  "youSeat": Seat;
  "resumeToken": ResumeToken;
  "lastBombActionId": Counter | null;
  "state": GameState | null;
}

export interface SeatReplaced {
  "type": "seatReplaced";
  "roomId": RoomId;
}

export interface GameSettings {
  "startingSpeedLevel": number;
  "turnAssistUnits": 64 | 128 | 192 | 256;
  "winsToFinish": number;
  "bombFuseTicks": 60 | 90 | 120 | 150 | 180 | 210 | 240 | 270 | 300;
}

export interface UpdateSettingsCommand {
  "type": "updateSettings";
  "sequence": Counter;
  "roomId": RoomId;
  "roundId": Counter;
  "settingsRevision": Counter;
  "settings": GameSettings;
}

export type V6ErrorCode = "invalidMessage" | "authenticationFailed" | "connectionInUse" | "notAuthenticated" | "alreadyAuthenticated" | "alreadyInRoom" | "notInRoom" | "roomNotFound" | "roomFull" | "staleRoom" | "invalidPhase" | "staleSequence" | "duplicateAction" | "rateLimited" | "capacityReached" | "resumeRejected" | "staleSettings" | "settingsLocked"
