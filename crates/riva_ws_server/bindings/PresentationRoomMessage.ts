// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ClientMessage } from "./ClientMessage";
import type { JoinRoomPayload } from "./JoinRoomPayload";
import type { LeaveRoomPayload } from "./LeaveRoomPayload";
import type { RequestSlideChangePayload } from "./RequestSlideChangePayload";
import type { RoomJoinedPayload } from "./RoomJoinedPayload";
import type { RoomLeftPayload } from "./RoomLeftPayload";
import type { ServerMessage } from "./ServerMessage";
import type { SlideChangedPayload } from "./SlideChangedPayload";

export type PresentationRoomMessage = { "type": "JoinRoom" } & ClientMessage<JoinRoomPayload> | { "type": "LeaveRoom" } & ClientMessage<LeaveRoomPayload> | { "type": "RequestSlideChange" } & ClientMessage<RequestSlideChangePayload> | { "type": "RoomJoined" } & ServerMessage<RoomJoinedPayload> | { "type": "RoomLeft" } & ServerMessage<RoomLeftPayload> | { "type": "SlideChanged" } & ServerMessage<SlideChangedPayload>;
