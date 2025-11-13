import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Box, TextField, IconButton, Paper, Typography } from "@mui/material";
import { Send as SendIcon } from "@mui/icons-material";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";

interface Message {
  text: string;
  timestamp: number;
}

const darkTheme = createTheme({
  palette: {
    mode: "dark",
    primary: {
      main: "#4a9eff",
    },
    background: {
      default: "#1a1a1a",
      paper: "#2a2a2a",
    },
  },
  typography: {
    fontFamily: "'Courier New', monospace",
  },
});

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

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

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim()) return;

    try {
      await invoke("send_command", { command: input });
      setInput("");
    } catch (error) {
      console.error("Failed to send command:", error);
    }
  };

  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          height: "100vh",
          width: "100vw",
          bgcolor: "background.default",
        }}
      >
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
            <Typography
              key={`${msg.timestamp}-${idx}`}
              sx={{
                py: 0.25,
                lineHeight: 1.5,
                wordWrap: "break-word",
                color: "text.primary",
              }}
            >
              {msg.text}
            </Typography>
          ))}
          <div ref={messagesEndRef} />
        </Box>
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
      </Box>
    </ThemeProvider>
  );
}

export default App;
