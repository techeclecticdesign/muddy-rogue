import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Box } from "@mui/material";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";
import { useMessageStream } from "./hooks/useMessageStream";
import { useSendCommand } from "./hooks/useSendCommand";
import { MessageList } from "./components/MessageList";
import { MessageInput } from "./components/MessageInput";

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

  useEffect(() => {
    invoke("get_start_message");
  }, []);

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
        <MessageList messages={messages} />
        <MessageInput onSendMessage={sendCommand} />
      </Box>
    </ThemeProvider>
  );
}

export default App;
