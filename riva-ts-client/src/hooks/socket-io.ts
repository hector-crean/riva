"use client";

import { CommandMessage } from "@/types/CommandMessage";
import { CommandType } from "@/types/CommandType";
import { EventType } from "@/types/EventType";
import { useEffect, useRef } from "react";
import { io, Socket } from "socket.io-client";

import { match  } from 'ts-pattern'

// Define a custom interface for your socket with proper typing
interface CustomSocket extends Socket {
  on(event: 'connect', listener: () => void): this;
  on(event: 'disconnect', listener: () => void): this;
  on(event: 'connect_error', listener: (err: Error) => void): this;
  on(event: 'message', listener: (data: EventType) => void): this;
}




export const handleEvent = (event: EventType) => {
  return match(event).with({type: 'PresentationJoined'}, (event) => {
    return event;
  }).with({type: 'PresentationLeft'}, (event) => {
    return event;
  }).with({type: 'SlideChanged'}, (event) => {
    return event;
  })
}

const useRivaWs = (
  cb: (msg: EventType) => CommandType,
) => {
  const socketRef = useRef<CustomSocket | null>(null);

  // Function to emit events to the server
  const emitCommand = (cmd: CommandMessage<CommandType>) => {
    if (socketRef.current && socketRef.current.connected) {
      socketRef.current.emit('message', cmd);
    } else {
      console.warn('Socket not connected, unable to emit event:', cmd);
    }
  };

  // eslint-disable-next-line react-hooks/exhaustive-deps
  function socketClient() {
    const socket = io("ws://0.0.0.0:5555") as CustomSocket;

    console.log(socket);

    socket.on("connect", () => {

      socket.on('message', (data) => {
        cb(data);
      });
      console.log("Connected");
    });

    socket.on("disconnect", () => {
      console.log("Disconnected");
    });

    socket.on("connect_error", async (err) => {
      console.log(`connect_error due to ${err.message}`);
      const res = await fetch("http://localhost:1234/ws");
      console.log(res);
    });

    socketRef.current = socket;
  }

  useEffect(() => {
    socketClient();
    return () => {
      socketRef?.current?.disconnect();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Return the emit function so components can use it
  return { emitCommand };
};

export { useRivaWs};
