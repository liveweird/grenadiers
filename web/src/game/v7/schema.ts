// This file was auto-generated from api/gameplay-v7.schema.json. Do not edit.

export type ProtocolVersionInput = number

export type GameplayVersion = 7

export type RulesVersion = 1 | 2 | 3 | 4

export type MapId = "fixed_v1" | "generated_v1"

export type PickupKind = "bombCapacity" | "blastRange" | "movementSpeed" | "kick" | "punch" | "bracelet" | "wallPass" | "heart" | "remoteBomb" | "piercingBomb" | "powerBomb" | "landmine" | "shield" | "stopwatch" | "invisibilityCloak" | "goldenFlame" | "shortFuse" | "diarrhea" | "slowFoot" | "shortFlame" | "longFuse" | "reverseControls" | "invisibleBombs" | "oilBarrel" | "teleporter"

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
  "availableItems": ItemId[];
}

export type Terrain = "floor" | "wall"

export interface GameState {
  "runningUpdates": number;
  "remainingTicks": number;
  "physicalTick": Tick;
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

export type ActorView = VisibleActorView | HiddenActorView

export interface BombView {
  "id": EntityId;
  "cell": Cell;
  "owner": EntityId;
  "range": number;
  "placedAt": Tick;
  "trigger": BombTrigger;
  "blast": "standard" | "piercing" | "power";
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

export type EventKind = BombPlacedEvent | BombDetonatedEvent | ActorEliminatedEvent | RoundEndedEvent | PickupCollectedEventKind | DamagePreventedEvent

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

export type GameClientMessage = Authenticate | CreateRoom | JoinRoom | ResumeRoom | Ready | Movement | ManualAction | LeaveRoom | Rematch | UpdateSettingsCommand

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
  "code": V7ErrorCode;
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
  "state": GameState | null;
  "lastActionId": Counter | null;
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
  "enabledItems": ItemId[];
}

export interface UpdateSettingsCommand {
  "type": "updateSettings";
  "sequence": Counter;
  "roomId": RoomId;
  "roundId": Counter;
  "settingsRevision": Counter;
  "settings": GameSettings;
}

export type V7ErrorCode = "invalidMessage" | "authenticationFailed" | "connectionInUse" | "notAuthenticated" | "alreadyAuthenticated" | "alreadyInRoom" | "notInRoom" | "roomNotFound" | "roomFull" | "staleRoom" | "invalidPhase" | "staleSequence" | "duplicateAction" | "rateLimited" | "capacityReached" | "resumeRejected" | "staleSettings" | "settingsLocked"

export type ItemId = "kick" | "punch" | "bracelet" | "wallPass" | "heart" | "remoteBomb" | "piercingBomb" | "powerBomb" | "landmine" | "shield" | "stopwatch" | "invisibilityCloak" | "goldenFlame" | "shortFuse" | "diarrhea" | "slowFoot" | "shortFlame" | "longFuse" | "reverseControls" | "invisibleBombs" | "oilBarrel" | "teleporter"

export type EquipmentId = "kick" | "punch" | "bracelet" | "wallPass" | "heart"

export type WeaponId = "remoteBomb" | "piercingBomb" | "powerBomb" | "landmine"

export interface InventoryView {
  "equipment": EquipmentId | null;
  "weapon": WeaponId | null;
}

export type EffectKind = "shield" | "stopwatch" | "invisibilityCloak" | "goldenFlame" | "shortFuse" | "diarrhea" | "slowFoot" | "shortFlame" | "longFuse" | "reverseControls" | "invisibleBombs"

export type EffectView = ShieldEffect | StopwatchEffect | InvisibilityCloakEffect | GoldenFlameEffect | ShortFuseEffect | DiarrheaEffect | SlowFootEffect | ShortFlameEffect | LongFuseEffect | ReverseControlsEffect | InvisibleBombsEffect

export interface ShieldEffect {
  "kind": "shield";
  "remainingTicks": number;
}

export interface StopwatchEffect {
  "kind": "stopwatch";
  "remainingTicks": number;
}

export interface InvisibilityCloakEffect {
  "kind": "invisibilityCloak";
  "remainingTicks": number;
}

export interface GoldenFlameEffect {
  "kind": "goldenFlame";
  "remainingTicks": number;
}

export interface ShortFuseEffect {
  "kind": "shortFuse";
  "remainingTicks": number;
}

export interface DiarrheaEffect {
  "kind": "diarrhea";
  "remainingTicks": number;
}

export interface SlowFootEffect {
  "kind": "slowFoot";
  "remainingTicks": number;
}

export interface ShortFlameEffect {
  "kind": "shortFlame";
  "remainingTicks": number;
}

export interface LongFuseEffect {
  "kind": "longFuse";
  "remainingTicks": number;
}

export interface ReverseControlsEffect {
  "kind": "reverseControls";
  "remainingTicks": number;
}

export interface InvisibleBombsEffect {
  "kind": "invisibleBombs";
  "remainingTicks": number;
}

export interface VisibleActorView {
  "type": "visible";
  "id": EntityId;
  "seat": Seat;
  "position": Position;
  "heading": Direction;
  "speedLevel": number;
  "alive": boolean;
  "bombCapacity": number;
  "bombRange": number;
  "liveBombCount": number;
  "inventory": InventoryView;
  "effects": EffectView[];
  "damageGraceRemainingTicks": number;
}

export interface HiddenActorView {
  "type": "hidden";
  "id": EntityId;
  "seat": Seat;
  "alive": boolean;
}

export type BombTrigger = TimedBombTrigger | RemoteBombTrigger | ContactBombTrigger

export interface TimedBombTrigger {
  "type": "timed";
  "remainingTicks": number;
  "paused": boolean;
}

export interface RemoteBombTrigger {
  "type": "remote";
}

export interface ContactBombTrigger {
  "type": "contact";
}

export interface DamagePreventedEvent {
  "type": "damagePrevented";
  "id": EntityId;
  "seat": Seat;
  "source": "heart" | "shield";
}

export interface ManualAction {
  "type": "manualAction";
  "sequence": Counter;
  "roomId": RoomId;
  "roundId": Counter;
  "actionId": Counter;
  "action": "bomb" | "equipment";
}
