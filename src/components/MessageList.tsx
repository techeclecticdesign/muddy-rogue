import { useEffect, useRef } from "react";
import { Box } from "@mui/material";
import type { Message } from "../types/message";
import { MessageItem } from "./MessageItem";

interface MessageListProps {
  messages: Message[];
}

export const MessageList = ({ messages }: MessageListProps) => {
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  return (
    <Box
      sx={{
        flex: 1,
        overflowY: "auto",
        p: 2,
        display: "flex",
        flexDirection: "column",
        gap: 0.5,
        "&::-webkit-scrollbar": {
          width: "8px",
        },
        "&::-webkit-scrollbar-track": {
          bgcolor: "background.default",
        },
        "&::-webkit-scrollbar-thumb": {
          bgcolor: "#3a3a3a",
          borderRadius: "4px",
          "&:hover": {
            bgcolor: "#4a4a4a",
          },
        },
      }}
    >
      {messages.map((msg, idx) => (
        <MessageItem
          key={`${msg.timestamp}-${idx}`}
          message={msg}
          index={idx}
        />
      ))}
      <div ref={messagesEndRef} />
    </Box>
  );
};
