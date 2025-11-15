import { Box } from "@mui/material";
import ReactMarkdown from "react-markdown";
import type { Message } from "../types/message";

interface MessageItemProps {
  message: Message;
  index: number;
}

export const MessageItem = ({ message, index }: MessageItemProps) => (
  <Box
    key={`${message.timestamp}-${index}`}
    sx={{
      py: 0.25,
      wordWrap: "break-word",
      color: "text.primary",
      "& p": { margin: 0, lineHeight: 1.5 },
      "& pre": { margin: "0.5rem 0", borderRadius: "4px" },
      "& code": {
        fontFamily: "'Courier New', monospace",
        fontSize: "0.9em",
      },
      "& ul, & ol": { marginLeft: "1.5rem", marginY: "0.5rem" },
      "& h1, & h2, & h3, & h4, & h5, & h6": {
        marginTop: "0.75rem",
        marginBottom: "0.25rem",
      },
    }}
  >
    <ReactMarkdown>{message.text}</ReactMarkdown>
  </Box>
);
