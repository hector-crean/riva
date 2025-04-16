"use client";
import {  useRivaWs } from "@/hooks/socket-io";
import { CommandMessage } from "@/types/CommandMessage";
import { Room } from "@/types/Room";
import { CommandType } from "@/types/CommandType";
import { useParams } from "next/navigation";
import { useEffect, useState } from "react";
import CreateRoomForm from "./create-room-form";
import { QueryObserverResult, RefetchOptions, useQuery } from "@tanstack/react-query";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { BASE_URL } from "@/const";
import { GetRoomsResponse } from "@/types/GetRoomsResponse";
import { match } from "ts-pattern";
import { Presentation } from "@/types/Presentation";
import { toast } from "sonner";
import { EventMessage } from "@/types/EventMessage";
import { EventType } from "@/types/EventType";


type RefetchRoomFn = (options?: RefetchOptions) => Promise<QueryObserverResult<GetRoomsResponse, Error>>

const RoomRow = ({
  room,
  refetchRoom,
  emitCommand,
}: {
  room: Room;
  refetchRoom: RefetchRoomFn
  emitCommand: (command: CommandMessage<CommandType>) => void
}) => {
  return match(room)
  .with({ type: "Presentation" }, (room) => (
    <PresentationRow
      presentation={room.payload}
      id={room.payload.id.room_name}
      refetchRoom={refetchRoom}
      emitCommand={emitCommand}
    />
  ))
  .exhaustive();
}

const PresentationRow = ({
  presentation,
  id,
  refetchRoom,
  emitCommand,
}: {
  presentation: Presentation;
  id: string;
  refetchRoom: RefetchRoomFn;
  emitCommand: (command: CommandMessage<CommandType>) => void
}) => {
 




  const handleJoinRoom = () => {
    emitCommand({
      room_id: presentation.id,
      payload: {
        type: "JoinPresentation",
      },
    });
    refetchRoom();
    toast.success("Joined room");
  };
  const handleLeaveRoom = () => {
    emitCommand({
      room_id: presentation.id,
      payload: {
        type: "LeavePresentation",
      },
    });
    refetchRoom();
    toast.success("Left room");
  };

  return (
    <TableRow key={id}>
      <TableCell className="font-medium">
        {presentation.id.organisation_id}
      </TableCell>
      <TableCell className="font-medium">{presentation.id.room_name}</TableCell>
      <TableCell>
        {new Date(presentation.created_at).toLocaleString()}
      </TableCell>
      <TableCell>{presentation.clients.length}</TableCell>
      <TableCell>Active?</TableCell>
      <TableCell>
        <Button variant="outline" size="sm" onClick={handleJoinRoom}>
          Join
        </Button>
        <Button variant="outline" size="sm" onClick={handleLeaveRoom}>
          Leave
        </Button>
      </TableCell>
    </TableRow>
  );
};


interface RoomTableProps {
  emitCommand: (command: CommandMessage<CommandType>) => void
}
const RoomTable = ({emitCommand}: RoomTableProps) => {
  const fetchRooms = async (): Promise<GetRoomsResponse> => {
    const response = await fetch(`${BASE_URL}/room`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response.json();
  };

  const { data, isLoading, isError, error, refetch } = useQuery({
    queryKey: ['rooms'],
    queryFn: fetchRooms,
  });

  return (
    <div className="container mx-auto py-6">
      <h1 className="text-2xl font-bold mb-6">Rooms</h1>

      {isLoading && <p>Loading rooms...</p>}
      {isError && (
        <p className="text-red-500">Error loading rooms: {error?.toString()}</p>
      )}

      {data && data.rooms.length > 0 ? (
        <Table>
          <TableCaption>List of available rooms</TableCaption>
          <TableHeader>
            <TableRow>
              <TableHead>Organisation</TableHead>
              <TableHead>Room Name</TableHead>
              <TableHead>Created At</TableHead>
              <TableHead>Participants</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data.rooms.map(([id, room]) => (
              <RoomRow
                key={`${id.organisation_id}-${id.room_name}`}
                room={room}
                refetchRoom={refetch}
                emitCommand={emitCommand}
              />
            ))}
          </TableBody>
        </Table>
      ) : (
        <div className="text-center py-8">
          <p className="mb-4">No rooms available</p>
        </div>
      )}

      <div className="mt-8">
        <h2 className="text-xl font-semibold mb-4">Create New Room</h2>
        <CreateRoomForm onRoomCreated={refetch} />
      </div>
    </div>
  );
};

export { RoomTable };
