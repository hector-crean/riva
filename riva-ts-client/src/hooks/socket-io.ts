"use client";

import { CommandMessage } from "@/types/CommandMessage";
import { CommandType } from "@/types/CommandType";
import { EventMessage } from "@/types/EventMessage";
import { EventType } from "@/types/EventType";
import {} from "@/types/RoomEvent";
import { useEffect, useRef } from "react";
import { io, Socket } from "socket.io-client";

import { match } from "ts-pattern";

// Define a custom interface for your socket with proper typing
interface CustomSocket extends Socket {
  on(event: "connect", listener: () => void): this;
  on(event: "disconnect", listener: () => void): this;
  on(event: "connect_error", listener: (err: Error) => void): this;
  on(event: "message", listener: (data: EventMessage<EventType>) => void): this;
}

// export const handleEvent = (event: EventMessage<EventType>): void => {
//   console.log("Received event:", event);
//   match(event.payload)
//     .with({ type: 'PresentationJoined' }, (event) => {
//       console.log("PresentationJoined", event);
//     })
//     .with({ type: "PresentationLeft" }, (event) => {
//       console.log("PresentationLeft", event);
//     })
//     .with({ type: "SlideChanged" }, (event) => {
//       console.log("SlideChanged", event);
//     })
//     .otherwise((payload) => {
//       console.warn("Unhandled event type:", payload);
//     });
// };

const useRivaWs = (
  cb: (msg: EventMessage<EventType>) => Promise<void>
) => {
  const socketRef = useRef<CustomSocket | null>(null);

  // Function to emit events to the server
  const emitCommand = (cmd: CommandMessage<CommandType>) => {
    console.log("Emitting command:", cmd);
    if (socketRef.current && socketRef.current.connected) {
      socketRef.current.emit("message", cmd);
      console.log("Command sent successfully");
    } else {
      console.warn("Socket not connected, unable to emit event:", cmd);
    }
  };

  

  // eslint-disable-next-line react-hooks/exhaustive-deps
  function socketClient() {
    console.log("Initializing socket connection to ws://0.0.0.0:5555");
    const socket = io("ws://0.0.0.0:5555") as CustomSocket;

    console.log("Socket instance created:", socket);

    socket.on('connect', () => {
      console.log("Socket connected successfully with ID:", socket.id);
      
    })

    socket.on('message', async (data) => {
      try {
        await cb(data);
      } catch (error) {
        console.error("Error in message callback:", error);
      }
    })

  

    socket.on("connect_error", async (err) => {
      console.error(`Socket connect_error: ${err.message}`, err);
      console.log("Attempting to fetch from http://localhost:1234/ws");
      try {
        const res = await fetch("http://localhost:1234/ws");
        console.log("Fetch response:", res.status, res.statusText);
      } catch (fetchErr) {
        console.error("Fetch error:", fetchErr);
      }
    });

    socketRef.current = socket;
  }

  useEffect(() => {
    console.log("useRivaWs hook initialized");
    socketClient();
    return () => {
      console.log("Cleaning up socket connection");
      socketRef?.current?.disconnect();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Return the emit function so components can use it
  return { emitCommand };
};

export { useRivaWs };
