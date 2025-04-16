"use client";
import { useRivaWs } from "@/hooks/socket-io";
import { ServerMessage } from "@/types/ServerMessage";
import { ServerEvent } from "@/types/ServerEvent";
import { useParams } from "next/navigation";
import { useEffect } from "react";
import { RoomTable } from "./rooms-table";
import { match } from "ts-pattern";
import { useQueryClient } from "@tanstack/react-query";

const Page = () => {
  const queryClient = useQueryClient();
  
  const handleEvent = async (event: ServerMessage<ServerEvent>): Promise<void> => {
    console.log("Received event:", event);
    match(event.payload)
      .with({ type: "PresentationJoined" }, async () => {
        console.log('Refetching rooms after PresentationJoined event');
        await queryClient.invalidateQueries({ queryKey: ['rooms'] });
      })
      .with({ type: "PresentationLeft" }, async () => {
        console.log('Refetching rooms after PresentationLeft event');
        await queryClient.invalidateQueries({ queryKey: ['rooms'] });
      })
      .with({ type: "SlideChanged" }, () => {
        // No refetch needed for slide changes
      })
      .otherwise((payload) => {
        console.warn("Unhandled event type:", payload);
      });
  };

  const { emitCommand } = useRivaWs(handleEvent);

  return (
    <RoomTable emitCommand={emitCommand} />
  );
};

export default Page;