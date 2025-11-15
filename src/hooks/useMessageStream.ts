import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { Message } from "../types/message";

export const useMessageStream = () => {
  const [messages, setMessages] = useState<Message[]>([]);

  useEffect(() => {
    const unlisten = listen<string>("stream-message", (event) => {
      setMessages((prev) => [
        ...prev,
        { text: event.payload, timestamp: Date.now() },
      ]);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return messages;
};
