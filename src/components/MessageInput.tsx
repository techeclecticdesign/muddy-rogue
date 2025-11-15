import { useState, useCallback } from "react";
import { TextField, IconButton, Paper } from "@mui/material";
import { Send as SendIcon } from "@mui/icons-material";

interface MessageInputProps {
  onSendMessage: (message: string) => void;
}

export const MessageInput = ({ onSendMessage }: MessageInputProps) => {
  const [input, setInput] = useState("");

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      if (!input.trim()) return;

      onSendMessage(input);
      setInput("");
    },
    [input, onSendMessage],
  );

  return (
    <Paper
      component="form"
      onSubmit={handleSubmit}
      sx={{
        display: "flex",
        gap: 1,
        p: 2,
        bgcolor: "background.paper",
        borderTop: 1,
        borderColor: "#3a3a3a",
        borderRadius: 0,
      }}
    >
      <TextField
        fullWidth
        value={input}
        onChange={(e) => setInput(e.target.value)}
        placeholder="Enter command..."
        autoFocus
        variant="outlined"
        size="small"
        sx={{
          "& .MuiOutlinedInput-root": {
            fontFamily: "'Courier New', monospace",
            fontSize: "0.95rem",
          },
        }}
      />
      <IconButton type="submit" color="primary" sx={{ px: 2 }}>
        <SendIcon />
      </IconButton>
    </Paper>
  );
};
