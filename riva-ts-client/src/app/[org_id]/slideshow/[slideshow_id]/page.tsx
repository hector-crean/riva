"use client";
import { handleEvent, useRivaWs } from "@/hooks/socket-io";
import { CommandMessage } from "@/types/CommandMessage";
import { CommandType } from "@/types/CommandType";
import { useParams } from "next/navigation";
import { useEffect } from "react";

type Params = {
  org_id: string;
  slideshow_id: string;
};

const Page = () => {
  const { org_id, slideshow_id } = useParams<Params>();

  const { emitCommand } = useRivaWs(handleEvent);

  useEffect(() => {
    const joinCmd: CommandMessage<CommandType> = {
      room_id: {
        room_name: "quarterly-review",
        organisation_id: "org_12345",
      },
      payload: {
        type: "JoinPresentation",
        client_id: "123",
        socket_id: "123",
        room_id: {
          room_name: "quarterly-review",
          organisation_id: "org_12345",
        },
      },
    };

    emitCommand(joinCmd);
  }, []);

  return (
    <div>
      <h1>Slideshow</h1>
      <h2>Slideshow ID: {slideshow_id}</h2>
    </div>
  );
};

export default Page;
