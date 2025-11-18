import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Box } from "@mui/material";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";
import { useMessageStream } from "./hooks/useMessageStream";
import { useSendCommand } from "./hooks/useSendCommand";
import { MessageList } from "./components/MessageList";
import { MessageInput } from "./components/MessageInput";
import MiniMap from "./components/MiniMap";

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
  const messages = useMessageStream();
  const sendCommand = useSendCommand();

  const [miniMapEnabled, setMiniMapEnabled] = useState(true);

  useEffect(() => {
    invoke("get_start_message");
  }, []);

  useEffect(() => {
    const unlisten = listen("toggle-minimap", () => {
      setMiniMapEnabled((prev) => !prev);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      <MiniMap enabled={miniMapEnabled} />
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          height: "100vh",
          width: "100vw",
          bgcolor: "background.default",
        }}
      >
        <MessageList messages={messages} />
        <Box
          sx={{
            height: "4px",
            background:
              "linear-gradient(180deg, #444 0%, #525252ff 50%, #313131ff 100%)",
            borderTop: "1px solid #555",
            borderBottom: "1px solid #363636ff",
            boxShadow:
              "0 2px 4px rgba(78, 78, 78, 0.5), inset 0 1px 0 rgba(255,255,255,0.1)",
          }}
        />
        <MessageInput onSendMessage={sendCommand} />
      </Box>
    </ThemeProvider>
  );
}

export default App;
